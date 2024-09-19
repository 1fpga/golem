import * as net from "@:golem/net";
import * as ui from "@:golem/ui";
import { getDb } from "./database";
import { Row } from "@:golem/db";
import type { Catalog as CatalogSchema } from "$schemas:catalog/catalog";

export class RemoteCatalog {
  public static async fetch1Fpga(): Promise<RemoteCatalog> {
    return RemoteCatalog.fetch("https://golem.1fpga.cloud");
  }

  public static async fetch(url: string): Promise<RemoteCatalog> {
    ui.show("Fetching catalog...\n" + url);

    // Dynamic loading to allow for code splitting.
    let validateCatalog = (await import("$schemas:catalog/catalog")).validate;

    function inner(url: string) {
      try {
        return net.fetchJson(url + "/catalog.json");
      } catch (e) {
        if (url.startsWith("http://")) {
          return inner(url.replace(/^http:\/\//, "https://"));
        } else {
          // Propagate the exception.
          throw e;
        }
      }
    }

    // Normalize the URL.
    if (!url.startsWith("https://") && !url.startsWith("http://")) {
      url = "https://" + url;
    }

    const maybeJson = inner(url);

    if (validateCatalog(maybeJson)) {
      return new RemoteCatalog(url, maybeJson);
    }

    throw new Error(
      (validateCatalog.errors || []).map((e) => e.message || "").join("\n"),
    );
  }

  private constructor(
    public readonly url: string,
    public readonly catalog: CatalogSchema,
  ) {}
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
      new Date("" + row.last_updated_at),
      "" + row.latest_downloaded,
      "" + row.latest_release,
      row.priority === null ? 0 : +row.priority,
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

    const lastUpdatedAt = new Date().toISOString();
    await db.execute(
      "INSERT INTO catalogs (name, url, last_updated_at, priority) VALUES (?, ?, ?, ?)",
      [remote.catalog.name, remote.url, lastUpdatedAt, priority],
    );

    return Catalog.getByUrl(remote.url);
  }

  private constructor(
    public readonly id: number,
    public readonly name: string,
    public readonly url: string,
    public readonly lastUpdatedAt: Date,
    public readonly latestDownloaded: string | undefined,
    public readonly latestRelease: string | undefined,
    public readonly priority: number,
  ) {}

  public async checkForUpdates(): Promise<void> {
    let remote = await RemoteCatalog.fetch(this.url);
  }
}
