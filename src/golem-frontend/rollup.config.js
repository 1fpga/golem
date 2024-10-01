import typescript from "@rollup/plugin-typescript";
import terser from "@rollup/plugin-terser";
import { nodeResolve } from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import json from "@rollup/plugin-json";
import del from "rollup-plugin-delete";

import codegen from "./rollup/rollup-plugin-codegen.js";
import {
  transformTaggedTemplate,
  transformCommonTags,
} from "./rollup/rollup-plugin-template-literals.js";
import dbMigrations from "./rollup/rollup-plugin-db-migrations.js";

const production =
  !("NODE_ENV" in process.env) || process.env.NODE_ENV === "production";

export default {
  input: "src/main.ts",
  output: {
    dir: "dist/",
    format: "es",
  },
  plugins: [
    del({ targets: "dist/*" }),
    codegen(),
    dbMigrations(),
    nodeResolve({
      preferBuiltins: false,
    }),
    commonjs({
      extensions: [".js", ".ts", ".cjs"],
      transformMixedEsModules: true,
    }),
    transformTaggedTemplate({
      tagsToProcess: ["sql"],
      transformer: (sql) => {
        return sql.replace(/\n/g, " ").replace(/\s\s+/g, " ");
      },
    }),
    transformCommonTags("oneLine"),
    transformCommonTags("stripIndent"),
    transformCommonTags("stripIndents"),
    typescript({
      exclude: ["src/**/*.spec.ts", "src/**/*.test.ts"],
    }),
    json(),
    [
      ...(production
        ? [
            terser({
              compress: true,
              ecma: 2020,
              mangle: true,
            }),
          ]
        : []),
    ],
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
