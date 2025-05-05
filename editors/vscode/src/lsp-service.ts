import type {
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';
import { findRamBinary, installRamBinary } from './installation';
import { logger } from './utils';

// LSP client state
let client: LanguageClient | undefined;
let clientReady = false;
let clientFailed = false;

/**
 * Initialize the LSP client
 */
export async function initLspClient(context: vscode.ExtensionContext): Promise<boolean> {
  try {
    // Try to find the RAM binary
    let serverPath = await findRamBinary(context);

    // If we couldn't find the binary and we're in production mode, ask the user if they want to install it
    if (!serverPath && context.extensionMode !== vscode.ExtensionMode.Development) {
      const selection = await vscode.window.showInformationMessage(
        'RAM Language Server not found. Would you like to install it?',
        'Install',
        'Cancel',
      );

      if (selection === 'Install') {
        const installed = await installRamBinary();
        if (installed) {
          serverPath = 'ram';
          vscode.window.showInformationMessage('RAM Language Server installed successfully.');
        }
        else {
          vscode.window.showErrorMessage('Failed to install RAM Language Server. Language features will be limited.');
          return false;
        }
      }
      else {
        // User chose not to install the binary
        vscode.window.showInformationMessage('RAM Language Server is required for language features. Other extension features will still work.');
        return false;
      }
    }

    if (!serverPath) {
      vscode.window.showErrorMessage('Could not find or install the RAM Language Server binary. Language features will be limited.');
      return false;
    }

    const logFilePath = context.asAbsolutePath(path.join('..', '..', 'logs', 'ram.log'));
    const run = {
      command: serverPath,
      args: ['lsp', '-vvv', '--mirror', logFilePath],
      options: { env: { RUST_BACKTRACE: 1 } },
    };

    vscode.window.showInformationMessage(`Starting RAM Language Server with binary: ${serverPath}`);

    const serverOptions: ServerOptions = {
      run,
      debug: run,
    };

    const clientOptions: LanguageClientOptions = {
      documentSelector: [{ scheme: 'file', language: 'ram' }],
      // Add error handling for client-side errors
      errorHandler: {
        error: (error, message, count = 0) => {
          logger.error(`LSP client error: ${message}`, error);

          if (count <= 3) {
            // For the first few errors, try to recover
            return { action: 1 }; // Continue
          }

          // After multiple errors, show a notification and stop trying
          vscode.window.showErrorMessage(`RAM Language Server encountered multiple errors. Some language features may not work correctly. Error: ${message}`);
          clientFailed = true;
          return { action: 2 }; // Shutdown
        },
        closed: () => {
          // If the connection closed unexpectedly but we were previously connected
          if (clientReady && !clientFailed) {
            vscode.window.showWarningMessage('RAM Language Server connection closed. Language features may be limited.');
            clientReady = false;
            return { action: 1 }; // Try to reconnect
          }
          return { action: 2 }; // Don't try to restart if we never connected or if we explicitly failed
        },
      },
    };

    client = new LanguageClient(
      'ramLanguageServer',
      'RAM Language Server',
      serverOptions,
      clientOptions,
    );

    // Register the client's capabilities
    client.registerProposedFeatures();

    // Add logging for diagnostics
    client.onDidChangeState((event) => {
      if (event.newState === 1) { // State.Running = 1
        client?.info('RAM Language Server is now running');
        clientReady = true;
        clientFailed = false;
      }
      else {
        client?.info('RAM Language Server has stopped');
        clientReady = false;
      }
    });

    // Start the client with proper error handling
    try {
      await client.start();
      return true;
    }
    catch (error) {
      clientFailed = true;
      logger.error('Failed to start RAM Language Server:', error);
      vscode.window.showErrorMessage(`Failed to start RAM Language Server: ${error instanceof Error ? error.message : String(error)}. Language features will be limited.`);
      return false;
    }
  }
  catch (error) {
    logger.error('Error initializing LSP client:', error);
    vscode.window.showErrorMessage(`Error initializing RAM Language Server: ${error instanceof Error ? error.message : String(error)}. Language features will be limited.`);
    return false;
  }
}

/**
 * Get the current status of the LSP client
 */
export function getLspStatus(): string {
  if (!client) {
    return 'Not initialized';
  }

  if (clientFailed) {
    return 'Failed';
  }

  if (clientReady) {
    return 'Running';
  }

  return 'Starting';
}

/**
 * Show the current status of the LSP client
 */
export function showLspStatus(): void {
  const status = getLspStatus();
  vscode.window.showInformationMessage(`RAM Language Server status: ${status}`);
}

/**
 * Restart the LSP client
 */
export async function restartLspClient(context: vscode.ExtensionContext): Promise<void> {
  // Stop the current client if it exists
  if (client) {
    await client.stop();
    client = undefined;
    clientReady = false;
    clientFailed = false;
  }

  // Try to start a new client
  const success = await initLspClient(context);

  if (success) {
    vscode.window.showInformationMessage('RAM Language Server restarted successfully.');
  }
  else {
    vscode.window.showErrorMessage('Failed to restart RAM Language Server. Language features will be limited.');
  }
}

/**
 * Dispose the LSP client
 */
export function disposeLspClient(): Promise<void> {
  if (client) {
    return client.stop();
  }
  return Promise.resolve();
}
