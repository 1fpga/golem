import typescript from "@rollup/plugin-typescript";
import terser from "@rollup/plugin-terser";
import { nodeResolve } from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import json from "@rollup/plugin-json";
import del from "rollup-plugin-delete";
import codegen from "./rollup/rollup-plugin-codegen.js";

export default {
  input: "src/main.ts",
  output: {
    dir: "dist/",
    format: "es",
  },
  plugins: [
    del({ targets: "dist/*" }),
    codegen(),
    typescript({}),
    nodeResolve(),
    commonjs({
      extensions: [".js", ".ts", ".cjs"],
      transformMixedEsModules: true,
    }),
    json(),
    terser({ compress: true }),
  ],
  external: [
    "@:fs",
    "@:golem/commands",
    "@:golem/core",
    "@:golem/db",
    "@:golem/net",
    "@:golem/patrons",
    "@:golem/settings",
    "@:golem/storage",
    "@:golem/ui",
  ],
};
