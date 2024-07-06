import typescript from "@rollup/plugin-typescript";
import terser from "@rollup/plugin-terser";
import { nodeResolve } from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import json from "@rollup/plugin-json";
import del from "rollup-plugin-delete";
import { unpluginAjvTools } from "ajv-build-tools";

import * as fs from "fs";
import * as path from "path";
import { glob } from "glob";
import { Ajv } from "ajv";
import * as j from "ajv/codegen";
import * as schema_to_ts from "json-schema-to-typescript";

// Create the JSON schema files.
fs.mkdirSync("codegen", { recursive: true });
glob("schemas/**/*.json", (err, files) => {
  if (err) {
    console.error(err);
    return;
  }

  for (const file of files) {
    const schema = JSON.parse(fs.readFileSync(file, "utf8"));
    const outputDir = path.join("codegen", path.dirname(file));
    const outputFile = path.basename(file, ".json");
    fs.mkdirSync(outputDir, { recursive: true });
    fs.writeFileSync(
      `${outputDir}/${name}.ts`,
      `export default ${JSON.stringify(schema, null, 2)};`,
    );
    fs.writeFileSync(`${outputDir}/${name}.ts`);
  }
});

export default {
  input: "src/main.ts",
  output: {
    dir: "dist/",
    format: "es",
  },
  plugins: [
    del({ targets: "dist/*" }),
    // unpluginAjvTools.rollup({
    //   include: "src/schemas/**",
    // }),
    commonjs(),
    json(),
    typescript(),
    nodeResolve(),
    terser({}),
  ],
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
