import { getDb } from "./database";
import { Row } from "@:golem/db";
import { RemoteCatalog } from "./remote/catalog";

/**
 * Compare two versions in the catalog JSONs.
 * @param a The first version.
 * @param b The second version.
 * @returns -1 if a < b, 0 if a == b, 1 if a > b.
 */
export function compareVersions(a: string, b: string): number {
  const aParts = a.split(".");
  const bParts = b.split(".");

  for (let i = 0; i < aParts.length || i < bParts.length; i++) {
    const aPart = aParts[i];
    const bPart = bParts[i];
    if (aPart === undefined) {
      return bPart === undefined ? 0 : -1;
    }

    const maybeResult = aPart.localeCompare(bPart);
    if (maybeResult !== 0) {
      return maybeResult;
    }
  }

  return 0;
}

/**
 * A Catalog is a place where games/cores/systems/etc can be downloaded from.
 * This strictly deals with catalogs from the database. Downloading, parsing
 * and validating catalogs is done by {@link RemoteCatalog}.
 */
export class Catalog {
  private static fromRow(row: Row | null): Catalog {
    if (row === null || row.id === null) {
      throw new Error("Catalog not found");
    }

    return new Catalog(
      +row.id,
      "" + row.name,
      "" + row.url,
      row.latest_check_at ? new Date("" + row.latest_check_at) : null,
      row.latest_update_at ? new Date("" + row.latest_update_at) : null,
      "" + row.last_updated,
      "" + row.version,
      +(row.priority || 0),
    );
  }

  public static async listCatalogs(): Promise<Catalog[]> {
    const db = await getDb();
    const names = await db.query("SELECT * FROM catalogs");
    return names.map(Catalog.fromRow).sort((a, b) => a.priority - b.priority);
  }

  public static async getByUrl(url: string): Promise<Catalog> {
    const db = await getDb();
    const row = await db.queryOne("SELECT * FROM catalogs WHERE url = ?", [
      url,
    ]);

    return Catalog.fromRow(row);
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
    const db = await getDb();
    // Check if it exists already.
    const id = await db.queryOne("SELECT id FROM catalogs WHERE url = ?", [
      remote.url,
    ]);
    if (id !== null) {
      throw new Error("Catalog already exists");
    }

    await db.execute(
      "INSERT INTO catalogs (name, url, latest_check_at, latest_update_at, last_updated, version, priority) VALUES (?, ?, ?, ?, ?)",
      [
        remote.catalog.name,
        remote.url,
        null, // latest_check_at. This hasn't been checked yet.
        remote.catalog.lastUpdated || null,
        remote.catalog.version,
        priority,
      ],
    );

    return Catalog.getByUrl(remote.url);
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
      this.latestCheckAt === null ||
      this.latestCheckAt.getTime() > Date.now()
    ) {
      return null;
    }

    let remote = await RemoteCatalog.fetch(this.url);
    if (compareVersions(remote.catalog.version, this.version) > 0) {
      return remote;
    } else {
      return null;
    }
  }
}
