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
    args: ['lsp', '-vvv', '--mirror', logFilePath, '--no-stdout-log'],
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

  client.start();
  client.outputChannel.show();
});

export { activate, deactivate };
