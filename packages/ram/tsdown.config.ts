import { defineConfig, type Options } from "tsdown";
import packageJson from "./package.json" with { type: "json" };

const baseOptions: Options = {
  clean: true,
  dts: true,
  entry: ["src/index.ts"],
  minify: false,
  external: Object.keys(packageJson.dependencies),
  sourcemap: true,
  target: "es2020",
  tsconfig: "tsconfig.json",
  treeshake: true,
};

export default [
  defineConfig({
    ...baseOptions,
    outDir: "lib/cjs",
    format: "cjs",
  }),
  defineConfig({
    ...baseOptions,
    outDir: "lib/esm",
    format: "esm",
  }),
  defineConfig({
    ...baseOptions,
    outDir: "lib/cli",
    entry: ["src/cli.ts"],
    dts: false,
    sourcemap: false,
    format: "esm",
  }),
];
