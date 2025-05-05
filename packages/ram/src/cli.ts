#!/usr/bin/env node

import { ram } from "./index.js";

async function run() {
  const args = process.argv.slice(2);
  const processResult = await ram.run(args).catch((error) => {
    // If the command fails, we'll just exit with the error's exit code
    // without printing the error stack trace
    if (error && typeof error === 'object' && 'exitCode' in error) {
      process.exit(error.exitCode);
    }
    // For any other unexpected errors
    process.exit(1);
  });
  process.exit(processResult.exitCode ?? 0);
}

void run();
