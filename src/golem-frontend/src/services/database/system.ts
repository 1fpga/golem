import { type Row } from "@:golem/db";
import { sql } from "../database";
import { RemoteCatalog, RemoteGamesDb, RemoteSystem } from "../remote";
import type { Catalog } from "./catalog";
import { Core } from "./core";

export interface SystemRow extends Row {
  id: number;
  catalog_id: number;
  name: string;
  unique_name: string;
  icon_path: string | null;
  image_path: string | null;
}

export class System {
  public static fromRow(row: SystemRow | null): System {
    if (row === null) {
      throw new Error("System not found");
    }

    return new System(
      row.id,
      row.catalog_id,
      row.name,
      row.unique_name,
      row.icon_path,
      row.image_path,
    );
  }

  public static async getById(id: number): Promise<System | null> {
    const [row] = await sql<SystemRow>`SELECT *
                                           FROM catalog_systems
                                           WHERE id = ${id}
                                           LIMIT 1`;
    return System.fromRow(row);
  }

  public static async create(system: RemoteSystem, catalogId: number) {
    const id = await sql<{ id: number }>`SELECT id
                                             FROM catalog_systems
                                             WHERE unique_name = ${system.uniqueName}`;
    if (id.length > 0) {
      throw new Error("System already exists");
    }

    await sql`INSERT INTO catalog_systems ${sql.insertValues({
      catalog_id: catalogId,
      name: system.name,
      unique_name: system.uniqueName,
      description: system.description || null,
      icon_path: system.iconPath,
      image_path: system.imagePath,
    })}`;
  }

  private constructor(
    public readonly id: number,
    public readonly catalogId: number,
    public readonly name: string,
    public readonly uniqueName: string,
    public readonly iconPath: string | null,
    public readonly imagePath: string | null,
  ) {}

  public async fetchRemote(): Promise<RemoteSystem> {
    const remoteCatalog = await RemoteCatalog.fetch(
      (await this.getCatalog()).url,
    );
    let systems = await remoteCatalog.fetchSystems(
      (uniqueName) => this.uniqueName === uniqueName,
    );
    return systems[this.uniqueName];
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
   * Install the system's cores (all of them), and its game database.
   */
  public async install(catalog: Catalog) {
    const remoteSystem = await this.fetchRemote();
    const core = await remoteSystem.fetchCores(true);
    const coreNames = Object.keys(core);
    if (coreNames.length === 0) {
      throw new Error("No cores found for system.");
    }

    // All cores.
    await Promise.all(
      coreNames.map((name) => Core.install(core[name], this, catalog)),
    );

    // Game database.
    const db = await remoteSystem.downloadGameDatabase();
    if (db) {
      console.log(`Downloaded ${db.schema.game.length} games`);
    }
  }
}
