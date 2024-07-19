import * as fs from "fs";
import * as path from "path";
import { globSync } from "glob";
import { Ajv } from "ajv";
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
      const files = globSync("schemas/**.json");

      for (const file of files) {
        const schema = JSON.parse(fs.readFileSync(file, "utf8"));
        const outputDir = path.join("codegen", path.dirname(file));
        const outputFile = path.basename(file, ".json");
        fs.mkdirSync(outputDir, { recursive: true });

        const ajv = new Ajv({
          code: { esm: true, source: true },
        });
        const validate = ajv.compile(schema);
        let moduleCode = standaloneCode(ajv, validate);

        fs.writeFileSync(`${outputDir}/${outputFile}.js`, moduleCode);

        let ts = await schema_to_ts.compile(schema, outputFile);
        let content = `
                    ${ts}
                    import { ValidateFunction } from "ajv";
                    export const validate: ValidateFunction<${capitalCase(outputFile)}>; 
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
      return null;
    },
  };
}
