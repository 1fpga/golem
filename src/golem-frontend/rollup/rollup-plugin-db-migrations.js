import * as fs from "fs";
import * as path from "path";
import { globSync } from "glob";

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
    name: "golem-codegen",
    async load(id) {
      if (id.startsWith("@:migrations")) {
        const files = globSync("migrations/**/up.sql");
        const output = Object.create(null);
        for (const file of files) {
          const content = fs.readFileSync(file, "utf8");
          const version = path.basename(path.dirname(file));
          output[version] = simplifySql(content);
        }

        return `
          export const up = ${JSON.stringify(output)};
        `;
      }
      return null;
    },
    resolveId(source) {
      if (source.startsWith("@:migrations")) {
        return source;
      }
      return null;
    },
  };
}
