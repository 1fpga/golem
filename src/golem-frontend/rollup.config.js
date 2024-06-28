import typescript from "@rollup/plugin-typescript";
import terser from "@rollup/plugin-terser";
import { nodeResolve } from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import json from "@rollup/plugin-json";

export default {
  input: "src/main.ts",
  output: {
    file: "dist/main.js",
    format: "es",
  },
  plugins: [commonjs(), json(), typescript(), nodeResolve(), terser()],
  external: [
    "@/golem/core",
    "@/golem/db",
    "@/golem/net",
    "@/golem/patrons",
    "@/golem/settings",
    "@/golem/storage",
    "@/golem/ui",
  ],
};
