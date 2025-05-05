import { defineExtension } from 'reactive-vscode';
import * as vscode from 'vscode';

import { disposeDecorations, initDecorations } from './decorations';
import { disposeLspClient, initLspClient, restartLspClient, showLspStatus } from './lsp-service';
import { logger } from './utils';

const { activate, deactivate } = defineExtension(async (ctx) => {
  // Initialize custom decorations - this should work regardless of LSP status
  initDecorations(ctx);

  // Register commands
  ctx.subscriptions.push(
    vscode.commands.registerCommand('ram.lspStatus', () => {
      showLspStatus();
    }),
    vscode.commands.registerCommand('ram.restartServer', async () => {
      await restartLspClient(ctx);
    }),
  );

  // Try to initialize the LSP client, but continue even if it fails
  try {
    await initLspClient(ctx);
  }
  catch (error) {
    // Log the error but don't prevent the extension from activating
    logger.error('Failed to initialize LSP client:', error);
    vscode.window.showErrorMessage(`Error initializing RAM Language Server: ${error instanceof Error ? error.message : String(error)}. Language features will be limited.`);
  }

  // Return a disposable for cleanup
  return {
    dispose: () => {
      // Clean up decorations
      disposeDecorations();

      // Stop the LSP client if it's running
      disposeLspClient().catch((error) => {
        logger.error('Error disposing LSP client:', error);
      });
    },
  };
});

export { activate, deactivate };
