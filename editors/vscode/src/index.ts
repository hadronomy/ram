import type {
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';
import * as path from 'node:path';
import { defineExtension } from 'reactive-vscode';
import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

import { disposeDecorations, initDecorations } from './decorations';
import { findRamBinary, installRamBinary } from './installation';

let client: LanguageClient;

const { activate, deactivate } = defineExtension(async (ctx) => {
  // Try to find the RAM binary
  let serverPath = await findRamBinary(ctx);

  // If we couldn't find the binary and we're in production mode, ask the user if they want to install it
  if (!serverPath && ctx.extensionMode !== vscode.ExtensionMode.Development) {
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
        vscode.window.showErrorMessage('Failed to install RAM Language Server. Please install it manually.');
        return {
          dispose: () => {},
        };
      }
    }
    else {
      // User chose not to install the binary
      vscode.window.showInformationMessage('RAM Language Server is required for language features.');
      return {
        dispose: () => {},
      };
    }
  }

  if (!serverPath) {
    vscode.window.showErrorMessage('Could not find or install the RAM Language Server binary.');
    return {
      dispose: () => {},
    };
  }

  const logFilePath = ctx.asAbsolutePath(path.join('..', '..', 'logs', 'ram.log'));
  const run = {
    command: serverPath,
    args: ['lsp', '-vvv', '--mirror', logFilePath],
    options: { env: { RUST_BACKTRACE: 1 } },
  };

  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'ram' }],
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
    if (event.newState === 1) {
      client.info('RAM Language Server is now running');
    }
    else {
      client.info('RAM Language Server has stopped');
    }
  });

  // Log errors that might prevent proper file change detection
  client.start().catch((error) => {
    client.error(`Failed to start RAM Language Server: ${error}`);
    console.error(error);
  });

  client.outputChannel.show();

  // Initialize custom decorations
  initDecorations(ctx);

  // Return a disposable for cleanup
  return {
    dispose: () => {
      disposeDecorations();
    },
  };
});

export { activate, deactivate };
