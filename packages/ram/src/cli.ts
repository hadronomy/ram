#!/usr/bin/env node

import { ram } from "./index.js";

async function run() {
  const args = process.argv.slice(2);

  try {
    const processResult = await ram.run(args);
    process.exit(processResult.exitCode ?? 0);
  } catch (error) {
    // Check if this is a binary resolution error
    if (error instanceof Error && error.message.includes("Couldn't find @ramlang/cli binary")) {
      console.error("\x1b[31mError: Failed to find the RAM binary executable\x1b[0m");
      console.error(error.message);
      console.error("\nPlease ensure the appropriate binary package is installed for your platform.");
      console.error("You may need to run: npm install @ramlang/cli-<platform>-<arch>");
      process.exit(1);
    }

    // If the command fails with an exit code, use that
    if (error && typeof error === 'object' && 'exitCode' in error &&
        (typeof error.exitCode === 'number' || error.exitCode === undefined)) {
      process.exit(error.exitCode ?? 1);
    }

    // For any other unexpected errors
    console.error("An unexpected error occurred:", error);
    process.exit(1);
  }
}

void run();
