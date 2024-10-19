import * as fs from "fs";
import * as path from "path";
import { globSync } from "glob";
import { Ajv } from "ajv";
import addFormats from "ajv-formats";
import * as schema_to_ts from "json-schema-to-typescript";
import standaloneCode from "ajv/dist/standalone/index.js";
import { capitalCase } from "change-case";
import { source } from "common-tags";

export default function (baseDir = process.cwd()) {
  const root = path.resolve(`${baseDir}/codegen`);
  return {
    name: "golem-codegen",
    async options() {
      if (fs.existsSync(root)) {
        fs.rmSync(path.resolve(baseDir, "codegen"), { recursive: true });
      }
      fs.mkdirSync(path.resolve(baseDir, "codegen"), { recursive: true });

      // Create the JSON schema files.
      const files = globSync("schemas/**/*.json");

      for (const file of files) {
        const schema = JSON.parse(fs.readFileSync(file, "utf8"));

        // Remove definitions only files.
        if (schema["type"] === undefined) {
          continue;
        }

        const outputDir = path.join("codegen", path.dirname(file));
        const outputFile = path.basename(file, ".json");
        fs.mkdirSync(outputDir, { recursive: true });

        const ajv = new Ajv({
          useDefaults: true,
          code: {
            esm: true,
            lines: true,
            source: true,
          },
          schemas: [
            ...files.map((file) => JSON.parse(fs.readFileSync(file, "utf8"))),
          ],
        });
        addFormats(ajv);

        const validate = ajv.getSchema(schema["$id"]);
        let moduleCode = standaloneCode(ajv, validate);

        fs.writeFileSync(`${outputDir}/${outputFile}.js`, moduleCode);

        let id = path.basename(schema["$id"], "json");
        let ts = await schema_to_ts.compile(schema, outputFile, {
          cwd: path.dirname(file),
          customName(schema, name) {
            if (name !== undefined) {
              return name;
            } else if (schema["$id"] !== undefined) {
              if (URL.canParse(schema["$id"])) {
                return path.basename(schema["$id"], "json");
              } else {
                return schema["$id"];
              }
            } else {
              return "Unknown";
            }
          },
        });

        let content = source`
          import type { ValidateFunction } from "ajv";
          
          ${ts}  
          
          export const validate: ValidateFunction<${capitalCase(id).replace(/ /g, "")}>;
          export default validate;
        `;

        fs.writeFileSync(`${outputDir}/${outputFile}.d.ts`, content);
      }
    },
    resolveId(source) {
      if (source.startsWith("$schemas:")) {
        return path.resolve(
          `${baseDir}/codegen/schemas/${source.substring(9)}.js`,
        );
      }
      if (source.startsWith("$schemas-json:") && source.endsWith(".json")) {
        return this.resolve(`${baseDir}/schemas/${source.substring(14)}`);
      }
      if (source.startsWith("node:")) {
        // return path.resolve(`${baseDir}/node_modules/${source.substring(5)}`);
        return this.resolve(source.substring(5));
      }
      return null;
    },
  };
}
