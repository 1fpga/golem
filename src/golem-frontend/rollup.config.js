import * as child_process from "node:child_process";
import typescript from "@rollup/plugin-typescript";
import terser from "@rollup/plugin-terser";
import { nodeResolve } from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import json from "@rollup/plugin-json";
import del from "rollup-plugin-delete";

import codegen from "./rollup/rollup-plugin-codegen.js";
import {
  transformCommonTags,
  transformTaggedTemplate,
} from "./rollup/rollup-plugin-template-literals.js";
import dbMigrations from "./rollup/rollup-plugin-db-migrations.js";
import constants from "./rollup/rollup-plugin-consts.js";

const production =
  !("NODE_ENV" in process.env) || process.env.NODE_ENV === "production";

const gitRev = child_process
  .execSync("git describe --all --always --dirty")
  .toString()
  .trim()
  .replace(/^.*\//, "");

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
    constants({
      environment: process.env.NODE_ENV,
      production,
      revision: gitRev,
    }),
    typescript({
      exclude: ["src/**/*.spec.ts", "src/**/*.test.ts"],
    }),
    json({}),
    commonjs({
      extensions: [".js", ".ts", ".cjs"],
      transformMixedEsModules: true,
    }),
    transformTaggedTemplate({
      tagsToProcess: [
        "sql",
        "sql1",
        "sql2",
        "sql3",
        "sql4",
        "sql5",
        "sql6",
        "sql7",
      ],
      transformer: (sql) => {
        return sql.replace(/\n/g, " ").replace(/\s\s+/g, " ");
      },
    }),
    transformCommonTags("oneLine"),
    transformCommonTags("source"),
    transformCommonTags("stripIndent"),
    transformCommonTags("stripIndents"),
    [
      ...(production
        ? [
            terser({
              compress: {
                arguments: true,
                ecma: 2020,
                module: true,
                passes: 2,
                pure_new: true,
                unsafe: true,
                unsafe_arrows: true,
                unsafe_comps: true,
                unsafe_math: true,
              },
              ecma: 2020,
              mangle: true,
            }),
          ]
        : []),
    ],
  ],
  external: [
    "@:golem/commands",
    "@:golem/core",
    "@:golem/db",
    "@:golem/fs",
    "@:golem/net",
    "@:golem/patrons",
    "@:golem/schema",
    "@:golem/settings",
    "@:golem/storage",
    "@:golem/ui",
    "@:golem/upgrade",
    "@:golem/utils",
  ],
  onLog(level, log, handler) {
    if (level === "warn") {
      handler("error", log); // turn other warnings into errors
    } else {
      handler(level, log); // otherwise, just print the log
    }
  },
};
