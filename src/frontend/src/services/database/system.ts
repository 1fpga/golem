import * as ui from "1fpga:ui";
import { type Row } from "1fpga:db";
import { sql } from "$/utils";
import { RemoteCatalog, RemoteSystem } from "../remote";
import type { Catalog } from "./catalog";
import { GamesIdentification } from "$/services/database/games_database";

export interface SystemRow extends Row {
  id: number;
  catalog_id: number;
  name: string;
  unique_name: string;
  description: string;
}

export class System {
  private static fromRow(row: SystemRow | null): System {
    if (row === null) {
      throw new Error("System not found");
    }

    return new System(
      row.id,
      row.catalog_id,
      row.name,
      row.unique_name,
      row.description,
    );
  }

  public static async listForCatalogId(catalogId: number): Promise<System[]> {
    const rows = await sql<SystemRow>`SELECT *
                                          FROM systems
                                          WHERE catalog_id = ${catalogId}`;
    return rows.map(System.fromRow);
  }

  public static async getByUniqueName(
    uniqueName: string,
  ): Promise<System | null> {
    const [row] = await sql<SystemRow>`SELECT *
                                           FROM systems
                                           WHERE unique_name = ${uniqueName}
                                           LIMIT 1`;
    return System.fromRow(row);
  }

  public static async getById(id: number): Promise<System | null> {
    const [row] = await sql<SystemRow>`SELECT *
                                           FROM systems
                                           WHERE id = ${id}
                                           LIMIT 1`;
    return System.fromRow(row);
  }

  public static async create(system: RemoteSystem, catalogId: number) {
    const id = await sql<{ id: number }>`SELECT id
                                             FROM systems
                                             WHERE unique_name = ${system.uniqueName}`;
    if (id.length > 0) {
      throw new Error("System already exists");
    }

    await sql`INSERT INTO systems ${sql.insertValues({
      catalog_id: catalogId,
      name: system.name,
      unique_name: system.uniqueName,
      description: system.description || null,
    })}`;
  }

  private constructor(
    public readonly id: number,
    public readonly catalogId: number,
    public readonly name: string,
    public readonly uniqueName: string,
    public readonly description: string,
  ) {}

  public async fetchRemote(): Promise<RemoteSystem | null> {
    const remoteCatalog = await RemoteCatalog.fetch(
      (await this.getCatalog()).url,
    );

    let systems = await remoteCatalog.fetchSystems(
      (uniqueName) => this.uniqueName === uniqueName,
    );
    return systems[this.uniqueName] ?? null;
  }

  public async getCatalog(): Promise<Catalog> {
    const { Catalog } = await import("./catalog");
    const catalog = await Catalog.getById(this.catalogId);
    if (catalog === null) {
      throw new Error("Catalog not found");
    }
    return catalog;
  }

  /**
   * Install the system's games database and other system related data.
   */
  public async install(catalog: Catalog) {
    const remoteSystem = await this.fetchRemote();
    if (remoteSystem === null) {
      throw new Error("(0) System not found");
    }

    // Game database.
    const db = await remoteSystem.downloadGameDatabase();
    if (db) {
      ui.show("Installing game database...", `System "${this.name}"`);
      await GamesIdentification.createBatch(db.games, this, catalog);
    }
  }
}
