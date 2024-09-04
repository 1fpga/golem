import * as fs from "fs";
import * as path from "path";
import { globSync } from "glob";
import { _, Ajv } from "ajv";
import * as schema_to_ts from "json-schema-to-typescript";
import standaloneCode from "ajv/dist/standalone/index.js";
import { capitalCase } from "change-case";

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
            source: true,
          },
          schemas: [
            ...files.map((file) => JSON.parse(fs.readFileSync(file, "utf8"))),
          ],
        });

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
              return path.basename(schema["$id"], "json");
            } else {
              return "Unknown";
            }
          },
        });

        let content = `
                    import type { ValidateFunction } from "ajv";
                    ${ts}
                    export const validate: ValidateFunction<${capitalCase(id)}>; 
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
    },
  };
}
