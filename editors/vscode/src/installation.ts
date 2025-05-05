import { exec } from 'node:child_process';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { promisify } from 'node:util';
import * as vscode from 'vscode';

const execAsync = promisify(exec);

/**
 * Check if a command exists in the PATH
 */
export async function commandExists(command: string): Promise<boolean> {
  try {
    const platform = os.platform();
    const cmd = platform === 'win32' ? 'where' : 'which';
    await execAsync(`${cmd} ${command}`);
    return true;
  }
  catch {
    return false;
  }
}

/**
 * Find the path to the RAM binary
 */
export async function findRamBinary(context: vscode.ExtensionContext): Promise<string | null> {
  // First check if we're in development mode
  if (context.extensionMode === vscode.ExtensionMode.Development) {
    // In development mode, use the local binary from target/debug
    const localDebugBinary = context.asAbsolutePath(path.join('..', '..', 'target', 'debug', 'ram'));
    if (fs.existsSync(localDebugBinary)) {
      return localDebugBinary;
    }

    // If the debug binary doesn't exist, try the release binary
    const localReleaseBinary = context.asAbsolutePath(path.join('..', '..', 'target', 'release', 'ram'));
    if (fs.existsSync(localReleaseBinary)) {
      return localReleaseBinary;
    }
  }

  // Check if 'ram' is in the PATH
  if (await commandExists('ram')) {
    return 'ram';
  }

  return null;
}

/**
 * Install the RAM binary via npm
 */
export async function installRamBinary(): Promise<boolean> {
  try {
    const terminal = vscode.window.createTerminal('Install RAM Language Server');
    terminal.show();
    terminal.sendText('npm install -g @ramlang/cli');

    // Give some time for the installation to complete
    await new Promise(resolve => setTimeout(resolve, 5000));

    // Check if installation succeeded
    return await commandExists('ram');
  }
  catch (error) {
    console.error('Failed to install RAM Language Server:', error);
    return false;
  }
}
