import { sql } from "../database";
import { Row } from "@:golem/db";
import { compareVersions, RemoteCatalog } from "../remote";
import { System, SystemRow } from "./system";

export interface CatalogRow extends Row {
  id: number;
  name: string;
  url: string;
  latest_check_at: string | null;
  latest_update_at: string | null;
  last_updated: string;
  version: string;
  priority: number;
}

/**
 * A Catalog is a place where games/cores/systems/etc can be downloaded from.
 * This strictly deals with catalogs from the database. Downloading, parsing
 * and validating catalogs is done by {@link RemoteCatalog}.
 */
export class Catalog {
  private static fromRow(row: CatalogRow | null): Catalog {
    if (row === null) {
      throw new Error("Catalog not found");
    }

    return new Catalog(
      row.id,
      row.name,
      row.url,
      row.latest_check_at ? new Date(row.latest_check_at) : null,
      row.latest_update_at ? new Date(row.latest_update_at) : null,
      row.last_updated,
      row.version,
      row.priority,
    );
  }

  public static async listCatalogs(): Promise<Catalog[]> {
    const rows = await sql<CatalogRow>`SELECT *
                                           FROM catalogs`;
    return rows.map(Catalog.fromRow).sort((a, b) => a.priority - b.priority);
  }

  public static async getByUrl(url: string): Promise<Catalog | null> {
    const row = await sql<CatalogRow>`SELECT *
                                          FROM catalogs
                                          WHERE url = ${url}
                                          LIMIT 1`;
    return row.length == 1 ? Catalog.fromRow(row[0]) : null;
  }

  public static async getById(id: number): Promise<Catalog | null> {
    const row = await sql<CatalogRow>`SELECT *
                                          FROM catalogs
                                          WHERE id = ${id}
                                          LIMIT 1`;
    return row.length == 1 ? Catalog.fromRow(row[0]) : null;
  }

  /**
   * Create a new catalog in the Database.
   * @param remote The {@link RemoteCatalog} to create.
   * @param priority
   */
  public static async create(
    remote: RemoteCatalog,
    priority = 0,
  ): Promise<Catalog> {
    // Check if it exists already.
    if ((await Catalog.getByUrl(remote.url)) !== null) {
      throw new Error("Catalog already exists");
    }

    let systems = await remote.fetchSystems(() => true, true);

    await sql`INSERT INTO catalogs ${sql.insertValues({
      name: remote.name,
      url: remote.url,
      last_updated: remote.lastUpdated || null,
      version: remote.version,
      priority,
    })}`;
    const catalog = await Catalog.getByUrl(remote.url);
    if (catalog === null) {
      throw new Error("Catalog not found after creation");
    }

    for (let system of Object.values(systems)) {
      await System.create(system, catalog.id);
    }

    return catalog;
  }

  private constructor(
    public readonly id: number,
    public readonly name: string,
    public readonly url: string,
    public readonly latestCheckAt: Date | null,
    public readonly latestUpdateAt: Date | null,
    public readonly lastUpdated: string,
    public readonly version: string,
    public readonly priority: number,
  ) {}

  /**
   * Check for updates in the catalog.
   * @returns Return the remote catalog if there's an update, otherwise null.
   */
  public async checkForUpdates(): Promise<RemoteCatalog | null> {
    if (
      this.latestCheckAt !== null &&
      this.latestCheckAt.getTime() > Date.now()
    ) {
      return null;
    }

    let remote = await RemoteCatalog.fetch(this.url);
    if (compareVersions(remote.version, this.version) > 0) {
      return remote;
    } else {
      return null;
    }
  }

  /**
   * Get the list of systems in this catalog.
   */
  public async listSystems(): Promise<System[]> {
    const rows = await sql<SystemRow>`SELECT *
                                          FROM catalog_systems
                                          WHERE catalog_id = ${this.id}`;
    return rows.map(System.fromRow);
  }
}
