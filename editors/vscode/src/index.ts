import type {
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';
import * as path from 'node:path';
import { defineExtension } from 'reactive-vscode';
import {
  LanguageClient,
} from 'vscode-languageclient/node';

let client: LanguageClient;

const { activate, deactivate } = defineExtension((ctx) => {
  const serverPath = ctx.asAbsolutePath(path.join('..', '..', 'target', 'debug', 'ram'));
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
});

export { activate, deactivate };
