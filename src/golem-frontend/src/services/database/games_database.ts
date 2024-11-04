import { System } from "./system";
import { Catalog } from "./catalog";
import { RemoteGameSchema } from "../remote";
import { sql, transaction } from "$/utils";
import { partitionAndProgress } from "$/ui/progress";
import { oneLine } from "common-tags";
import * as fs from "@:golem/fs";

interface GamesIdentificationRow {
  id: number;
  system_id: number;
  catalog_id: number;
  name: string;
  shortname: string | null;
  region: string | null;
  languages: string | null;
  description: string;
}

interface GamesIdentificationFileRow {
  id: number;
  games_id: number;
  extension: string;
  size: number;
  sha256: string;
}

/**
 * A game identification database entry, including its dependencies (the files
 * identifications).
 */
export class GamesIdentification {
  public static fromRow(
    row: GamesIdentificationRow | null,
  ): GamesIdentification {
    if (row === null) {
      throw new Error("Game not found");
    }

    return new GamesIdentification(
      row.id,
      row.system_id,
      row.catalog_id,
      row.name,
      row.region,
      row.languages,
      row.description,
    );
  }

  /**
   * Add all game files found under a root directory.
   * @param root The root directory to search for games.
   */
  public static async addGamesFromRoot(root: string): Promise<void> {
    interface Row {
      extension: string;
    }

    // Get the extension list from the database.
    const extensions = await sql<Row>`
            SELECT DISTINCT extension
            FROM games_identification_files
        `;

    const allFiles = await fs.findAllFiles(root, {
      extensions: extensions.map((e) => e.extension),
    });
    const missing = await GamesIdentification.createGamesFromFiles(allFiles);

    console.log(missing);
  }

  public static async createGamesFromFiles(files: string[]): Promise<string[]> {
    const missing: string[] = [];

    await partitionAndProgress(
      files,
      100,
      "Adding games",
      (current, total) => `Adding games: ${current}/${total}`,
      async (partition) => {
        const shas = await fs.sha256(partition);
        const sizes = await fs.fileSize(partition);

        for (let i = 0; i < partition.length; i++) {
          const path = partition[i];
          const sha256 = shas[i];
          const size = sizes[i];

          const [row] = await sql<{ games_id: number }>`
                        SELECT games_id
                        FROM games_identification_files
                        WHERE sha256 = ${sha256}
                          AND (size = ${size} OR size = 0)
                    `;

          if (row === undefined) {
            missing.push(path);
          } else {
            await sql`INSERT INTO games ${sql.insertValues({
              games_id: row.games_id,
              path: path,
              size,
              sha256,
            })}`;
          }
        }
      },
    );

    return missing;
  }

  public static async createBatch(
    remotes: RemoteGameSchema[],
    system: System,
    catalog: Catalog,
  ) {
    // Instead of using this method directly, we use transaction strings that are precomputed.
    // Boa is very slow at string concatenation, and we need to do it for every record. Doing
    // it this way can lead to issues when we change the schema, but it's a tradeoff since
    // this reduces the time to insert all records by over a factor of 10.
    const sql1 = await transaction();

    const insertGamesIdStatement = oneLine`
            INSERT INTO games_identification
            (system_id, catalog_id, name, shortname, region, languages, description)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT DO NOTHING
            RETURNING id
        `;
    const sourceGamesIdStatement = oneLine`
            INSERT INTO games_identification_files
                (games_id, catalog_id, extension, size, sha256)
            VALUES (?, ?, ?, ?, ?)
        `;

    await partitionAndProgress(
      remotes,
      100,
      "Installing game database...",
      (current, total) =>
        `System "${system.uniqueName}"\n\n${current}/${total} games processed`,
      async (partition) => {
        for (const remote of partition) {
          if (typeof remote.languages === "string") {
            remote.languages = remote.languages.toLocaleLowerCase();
          } else if (Array.isArray(remote.languages)) {
            remote.languages = remote.languages.join(",").toLocaleLowerCase();
          }
          const languages: string | null = remote.languages ?? null;

          let games_id: number;
          // Insert the game identification.
          const r = await sql1.db.queryOne(insertGamesIdStatement, [
            system.id,
            catalog.id,
            remote.name,
            remote.shortname ?? null,
            remote.region ?? null,
            languages,
            remote.description ?? null,
          ]);

          if (r === undefined || r === null) {
            const [row] = await sql1<{ id: number }>`
                            SELECT id
                            FROM games_identification
                            WHERE name = ${remote.name}
                              AND system_id = ${system.id}`;
            if (row === undefined) {
              throw new Error("Game not found: " + remote.name);
            } else {
              games_id = row.id;
            }
          } else {
            games_id = +(r.id || 0);
          }

          // Insert all its sources. Partition them into multiple runs.
          const sources = remote.sources.flatMap((source) => [
            ...(source.files?.map((file) => [
              games_id,
              catalog.id,
              file.extension,
              file.size,
              file.sha256 ?? null,
            ]) ?? []),
          ]);

          if (sources.length > 0) {
            await Promise.all([
              ...sources.map((source) =>
                sql1.db.execute(sourceGamesIdStatement, source),
              ),
            ]);
          }
        }
      },
    ).then(
      () => sql1.commit(),
      (e) => {
        sql1.rollback();
        throw e;
      },
    );
  }

  public static async fromId(id: number): Promise<GamesIdentification> {
    const [row] = await sql<GamesIdentificationRow>`SELECT *
                                                        FROM games_identification
                                                        WHERE id = ${id}
                                                        LIMIT 1`;

    return GamesIdentification.fromRow(row);
  }

  private constructor(
    public readonly id: number,
    public readonly system_id: number,
    public readonly catalog_id: number,
    public readonly name: string,
    public readonly region: string | null,
    public readonly languages: string | null,
    public readonly description: string,
  ) {}
}
