import { compareVersions, sql } from "$/utils";
import { Row } from "@:golem/db";
import { RemoteCatalog, WellKnownCatalogs } from "../remote";
import { System } from "./system";
import { Core } from "$/services/database/core";

export interface CatalogRow extends Row {
  id: number;
  name: string;
  url: string;
  latest_update_at: string | null;
  last_updated: string;
  version: string;
  priority: number;
  update_pending: boolean;
}

export interface ListCatalogsOptions {
  updatePending?: boolean;
  url?: string;
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
      row.latest_update_at ? new Date(row.latest_update_at) : null,
      row.last_updated,
      row.version,
      row.priority,
      row.update_pending,
    );
  }

  public static async hasCatalog(
    url: WellKnownCatalogs | string,
  ): Promise<boolean> {
    const catalogs = await Catalog.listCatalogs({ url });
    return catalogs.length > 0;
  }

  /**
   * Perform the update for all catalogs that have updates pending.
   */
  public static async updateAll(): Promise<boolean> {
    const catalogs = await Catalog.listCatalogs({ updatePending: true });
    let any = false;
    for (const catalog of catalogs) {
      any = any || (await catalog.update());
    }
    return any;
  }

  /**
   * Check for updates in catalogs that do not have update pendings, updating
   * the `update_pending` field in the database for those who have new updates.
   */
  public static async checkForUpdates(): Promise<boolean> {
    await RemoteCatalog.clearCache();

    const catalogs = await Catalog.listCatalogs({ updatePending: false });
    const shouldUpdate: Catalog[] = (
      await Promise.all(
        catalogs.map(async (c) =>
          (await c.checkForUpdates()) !== null ? c : undefined,
        ),
      )
    ).filter((c) => c !== undefined);

    await sql`UPDATE catalogs
                  SET update_pending = true
                  WHERE ${sql.in(
                    "id",
                    shouldUpdate.map((c) => c.id),
                  )}`;
    return shouldUpdate.length > 0;
  }

  public static async listCatalogs(
    options: ListCatalogsOptions = {},
  ): Promise<Catalog[]> {
    const rows = await sql<CatalogRow>`SELECT *
                                           FROM catalogs
                                           WHERE ${sql.and(
                                             true,
                                             options.url
                                               ? sql`url =
                                                           ${options.url}`
                                               : undefined,
                                             options.updatePending
                                               ? sql`update_pending =
                                                           ${options.updatePending}`
                                               : undefined,
                                           )}
        `;
    return rows.map(Catalog.fromRow).sort((a, b) => a.priority - b.priority);
  }

  public static async count(updatePending = false): Promise<number> {
    const [{ count }] = await sql<{ count: number }>`SELECT COUNT(*) as count
                                                       FROM catalogs ${updatePending ? sql`WHERE update_pending = true` : ""}`;
    return count;
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
    public readonly latestUpdateAt: Date | null,
    public readonly lastUpdated: string,
    public readonly version: string,
    public readonly priority: number,
    public readonly updatePending: boolean,
  ) {}

  /**
   * Check for updates in the catalog.
   * @returns Return the remote catalog if there's an update, otherwise null.
   */
  private async checkForUpdates(): Promise<RemoteCatalog | null> {
    const remote = await RemoteCatalog.fetch(this.url);
    const shouldUpdate = compareVersions(remote.version, this.version) > 0;
    return shouldUpdate ? remote : null;
  }

  /**
   * Get the list of systems in this catalog.
   */
  public async listSystems(): Promise<System[]> {
    return System.listForCatalogId(this.id);
  }

  /**
   * Get a list of cores in this catalog.
   */
  public async listCores(): Promise<Core[]> {
    return Core.listForCatalogId(this.id);
  }

  /**
   * Perform an update in the catalog.
   * @returns Whether the catalog was updated or not.
   */
  public async update(): Promise<boolean> {
    const remote = await RemoteCatalog.fetch(this.url);
    const shouldUpdate = compareVersions(remote.version, this.version) > 0;
    if (!shouldUpdate) {
      // Just a last check since this can be expensive.
      return false;
    }
    console.debug(
      "Updating catalog...",
      this.name,
      JSON.stringify({
        current: this.version,
        remote: remote.version,
      }),
    );

    const systems = await remote.fetchSystems(() => true);

    return true;
  }

  /**
   * Fetch the remote catalog for this catalog.
   */
  public async fetchRemote(): Promise<RemoteCatalog> {
    return await RemoteCatalog.fetch(this.url);
  }
}
