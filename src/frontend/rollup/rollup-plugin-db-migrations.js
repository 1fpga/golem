import * as fs from "fs";
import * as path from "path";
import { globSync } from "glob";
import { stripIndent } from "common-tags";

/**
 * Simplify the SQL content by removing comments and extra whitespace.
 * @param sql {string} The SQL content to simplify.
 * @returns {*}
 */
function simplifySql(sql) {
  return sql
    .split("\n")
    .map((x) => x.replace(/--.*?$/, "").trim())
    .filter((x) => x.length > 0)
    .join(" ")
    .replace(/\s\s+/g, " ");
}

export default function (baseDir = process.cwd()) {
  const root = path.resolve(`${baseDir}/codegen`);
  return {
    name: "1fpga-codegen",
    async load(id) {
      if (id.startsWith("1fpga:migrations")) {
        const files = globSync("migrations/**/up.sql");
        let output = "{";
        for (const file of files) {
          const content = fs.readFileSync(file, "utf8");
          const version = path.basename(path.dirname(file));
          let applyUpPath = path.join(path.dirname(file), "apply.ts");
          if (!fs.existsSync(applyUpPath)) {
            applyUpPath = undefined;
          }

          output += stripIndent`
            ${JSON.stringify(version)}: {
              up: {
                sql: ${JSON.stringify(simplifySql(content))},
                ${
                  applyUpPath
                    ? `apply: async (a, b, c, d, e) => {
                  await (await import(${JSON.stringify(applyUpPath)})).up(a, b, c, d, e);
                },`
                    : ""
                }
              },
            },
          `;
        }
        output += "}";

        return `
          export const migrations = ${output};
        `;
      }
      return null;
    },
    resolveId(source) {
      if (source.startsWith("1fpga:migrations")) {
        return source;
      }
      return null;
    },
  };
}
