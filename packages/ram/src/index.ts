import { execa, type Options as ExecaOptions, type ResultPromise } from "execa";
import { fileURLToPath } from "node:url";
import { getExePath } from "./get-exe-path.js";


/**
 * Runs the `@ramlang/cli` with the provided arguments.
 *
 * @param args - The arguments to pass to `@ramlang/cli`.
 * These should be in an array of string format.
 * Every option and their value should be its own entry in the array.
 *
 * @param execaOptions - Options to pass to {@link execa}.
 *
 * @returns A promise that resolves when the `@ramlang/cli` has finished running.
 */
async function run(argsOrOptions: string[], execaOptions?: ExecaOptions): Promise<ResultPromise> {
  const exePath = await getExePath();
  const args = argsOrOptions;

  return execa(fileURLToPath(exePath), args, {
    stdio: "inherit",
    ...execaOptions,
  });
}

export const ram = {
  run,
};
