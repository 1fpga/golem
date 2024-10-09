import { System } from "./system";
import { Catalog } from "./catalog";
import { RemoteGameSchema } from "../remote";
import { sql } from "$/utils";

interface GamesIdentificationRow {
  id: number;
  system_id: number;
  catalog_id: number;
  name: string;
  region: string | null;
  languages: string | null;
  description: string;
}

interface GamesIdentificationFileRow {
  id: number;
  game_id: number;
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

  public static async create(
    remote: RemoteGameSchema,
    system: System,
    catalog: Catalog,
  ): Promise<GamesIdentification> {
    // Insert the game identification.
    const [row] = await sql<GamesIdentificationRow>`
            INSERT INTO games_identification ${sql.insertValues({
              system_id: system.id,
              catalog_id: catalog.id,
              name: remote.name,
              region: remote.region || null,
              languages: (remote.languages ?? []).toString() || null,
              description: remote.description,
            })}
            ON CONFLICT
            DO NOTHING
            RETURNING *`;

    // Insert all its sources.
    await sql`INSERT INTO games_identification_files ${sql.insertValues(
      remote.sources.map((source) => ({
        game_id: row.id,
        extension: source.extension,
        size: source.size,
        sha256: source.sha256,
      })),
    )}`;

    return GamesIdentification.fromRow(row);
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
