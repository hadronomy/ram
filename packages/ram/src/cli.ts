#!/usr/bin/env node

import { ram } from "./index.js";

async function run() {
  const args = process.argv.slice(2);
  const processResult = await ram.run(args);

  process.exit(processResult.exitCode ?? 0);
}

void run();
