import type {
  LanguageClientOptions,
  ServerOptions,
  StreamInfo,
} from 'vscode-languageclient/node';
import { Readable, Writable } from 'node:stream';
import { defineExtension } from 'reactive-vscode';
import { workspace } from 'vscode';
import {
  LanguageClient,
} from 'vscode-languageclient/node';
import { WebSocket } from 'ws';

let client: LanguageClient;

const { activate, deactivate } = defineExtension(() => {
  const serverOptions: ServerOptions = () => {
    return new Promise<StreamInfo>((resolve, reject) => {
      const socket = new WebSocket('ws://localhost:9257');

      socket.addEventListener('open', () => {
        // Create Node.js streams to communicate with the language server
        const reader = new Readable({
          read() {}, // Implement empty read as we'll push data manually
        });

        const writer = new Writable({
          write(chunk, encoding, callback) {
            // Send the data to the WebSocket
            socket.send(chunk);
            callback();
          },
        });

        // Handle WebSocket events
        socket.addEventListener('message', (event) => {
          // eslint-disable-next-line node/prefer-global/buffer
          const data = event.data instanceof Buffer ? event.data : Buffer.from(String(event.data));
          reader.push(data);
        });

        socket.addEventListener('close', () => {
          reader.push(null); // Signal end of the stream
        });

        socket.addEventListener('error', (err) => {
          reader.emit('error', err);
          writer.emit('error', err);
        });

        resolve({
          reader,
          writer,
        });
      });

      socket.addEventListener('error', (event) => {
        reject(event);
      });
    });
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'ram' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.ram'),
    },
  };

  client = new LanguageClient(
    'ramLanguageServer',
    'RAM Language Server',
    serverOptions,
    clientOptions,
  );

  client.start();
});

export { activate, deactivate };
