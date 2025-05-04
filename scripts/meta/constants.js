import * as selfExports from './constants.js';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';
import { existsSync } from 'fs';

// Get the directory name of the current module
const __dirname = dirname(fileURLToPath(import.meta.url));

/**
 * Find the repository root by traversing up the directory tree
 * looking for specific repository marker files
 */
function findRepoRoot(startDir) {
  let currentDir = startDir;
  const rootMarkers = ['package.json', 'Cargo.toml', '.git'];
  
  while (currentDir !== '/') {
    // Check if any of the root markers exist in the current directory
    if (rootMarkers.some(marker => existsSync(resolve(currentDir, marker)))) {
      return currentDir;
    }
    // Move up one directory
    currentDir = dirname(currentDir);
  }
  
  // If we reach the filesystem root without finding markers, return the starting directory
  return startDir;
}

// Find repository root starting from the directory of this file
export const REPO_ROOT = findRepoRoot(resolve(__dirname, '../..'));

if (process.argv[1] === import.meta.filename) {
  // If this file is executed directly, print the exports
  console.log(selfExports);
}