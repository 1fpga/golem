import * as net from "@:golem/net";
import * as ui from "@:golem/ui";

import type {
  Catalog as CatalogSchema,
  System as CatalogSystemSchema,
} from "$schemas:catalog/catalog";
import type { System as SystemSchema } from "$schemas:catalog/system";
import type { Core as CoreSchema } from "$schemas:catalog/core";
import { RemoteGamesDb } from "./games_database";
import { fetchJsonAndValidate } from "../../utils/fetch_json";

export const CATALOG_1FPGA_URL = "https://catalog.1fpga.cloud/";

const CATALOG_CACHE: { [url: string]: RemoteCatalog } = {};

/**
 * Compare two versions in the catalog JSONs.
 * @param a The first version.
 * @param b The second version.
 * @returns -1 if a < b, 0 if a == b, 1 if a > b.
 */
export function compareVersions(
  a: string | number | null,
  b: string | number | null,
): number {
  if (a === null) {
    return b === null ? 0 : 1;
  } else if (b === null) {
    return -1;
  } else if (typeof a === "number") {
    if (typeof b === "number") {
      return a - b;
    } else {
      a = a.toString();
    }
  } else {
    b = b.toString();
  }

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
 * A remote core is a `core.json` that is fetched from the internet,
 * parsed and validated.
 */
export class RemoteCore {
  public static async fetch(key: string, url: string, system: RemoteSystem) {
    const u = new URL(url, system.url).toString();

    ui.show(
      "Fetching core...",
      `Catalog "${system.catalog.name}"\nSystem "${system.name}"\nURL: ${u}`,
    );

    // Dynamic loading to allow for code splitting.
    const json = await fetchJsonAndValidate(
      u,
      (await import("$schemas:catalog/core")).validate,
    );

    if (key !== json.uniqueName) {
      throw new Error(
        `Core name mismatch: ${JSON.stringify(key)} != ${JSON.stringify(json.uniqueName)}`,
      );
    }
    return new RemoteCore(u, json, system);
  }

  constructor(
    public readonly url: string,
    public readonly schema: CoreSchema,
    public readonly system: RemoteSystem,
  ) {}

  get name() {
    return this.schema.name || this.schema.uniqueName;
  }

  get uniqueName() {
    return this.schema.uniqueName;
  }

  get latestRelease() {
    // If a release has the tag `latest`, use that.
    const latest = this.schema.releases.find((r) => r.tags?.includes("latest"));
    if (latest) {
      return latest;
    }

    // Sort by version number, descending, skipping `alpha` or `beta` tags.
    return this.schema.releases
      .filter((x) => !(x.tags?.includes("alpha") || x.tags?.includes("beta")))
      .sort((a, b) => -compareVersions(a.version, b.version))[0];
  }

  get releases() {
    return this.schema.releases;
  }

  get description() {
    return this.schema.description || null;
  }

  get iconPath() {
    return this.schema.icon || null;
  }

  get imagePath() {
    return this.schema.image || null;
  }
}

/**
 * A remote system array is a `system.json` that is fetched from the internet,
 * parsed and validated.
 */
export class RemoteSystem {
  private cores_: { [k: string]: RemoteCore } | undefined;
  private iconPath_: string | null = null;

  public static pathForAsset(system: RemoteSystem, assetType: string) {
    return `/media/fat/golem/systems/${system.uniqueName}/${assetType}`;
  }

  public static async fetch(
    key: string,
    schema: CatalogSystemSchema,
    catalog: RemoteCatalog,
    deep = false,
  ): Promise<RemoteSystem> {
    const u = new URL(schema.url, catalog.url).toString();

    ui.show("Fetching system...", `Catalog ${catalog.name}\nURL: ${u}`);

    // Dynamic loading to allow for code splitting.
    const json = await fetchJsonAndValidate(
      u,
      (await import("$schemas:catalog/system")).validate,
    );

    if (key !== json.uniqueName) {
      throw new Error(
        `System name mismatch: ${JSON.stringify(key)} != ${JSON.stringify(json.uniqueName)}`,
      );
    }
    const system = new RemoteSystem(u, json, catalog);

    if (deep) {
      await system.fetchCores(deep);

      if (json.icon) {
        const destination = RemoteSystem.pathForAsset(system, "icons");
        system.iconPath_ = await net.download(
          new URL(json.icon, u).toString(),
          destination,
        );
      }
    }

    return system;
  }

  constructor(
    public readonly url: string,
    private readonly system_: SystemSchema,
    private catalog_: RemoteCatalog,
  ) {}

  get name(): string {
    return this.system_.name || this.system_.uniqueName;
  }

  get uniqueName(): string {
    return this.system_.uniqueName;
  }

  get description(): string | null {
    return this.system_.description || null;
  }

  get catalog(): RemoteCatalog {
    return this.catalog_;
  }

  get iconPath(): string | null {
    return this.iconPath_;
  }

  get imagePath(): string | null {
    return null;
  }

  get size(): number {
    return this.system_.gamesDb?.size || 0;
  }

  async fetchCores(_deep = false) {
    if (this.cores_ === undefined) {
      if (this.system_.cores) {
        const entries = Object.entries(this.system_.cores);
        const cores = await Promise.all(
          entries.map(async ([key, core]) => {
            return [key, await RemoteCore.fetch(key, core.url, this)] as [
              string,
              RemoteCore,
            ];
          }),
        );
        this.cores_ = Object.fromEntries(cores);
      } else {
        this.cores_ = {};
      }
    }
    return this.cores_;
  }

  async downloadGameDatabase() {
    if (!this.system_.gamesDb) {
      return;
    }

    return await RemoteGamesDb.fetch(this.system_.gamesDb.url, this);
  }
}

/**
 * A remote catalog is the `catalog.json` that is fetched from the internet,
 * parsed and validated.
 */
export class RemoteCatalog {
  private systems_: { [key: string]: RemoteSystem } | undefined;

  public static async fetch1Fpga(): Promise<RemoteCatalog> {
    return RemoteCatalog.fetch(CATALOG_1FPGA_URL);
  }

  public static async fetch(url: string, all = false): Promise<RemoteCatalog> {
    if (CATALOG_CACHE[url]) {
      // TODO: When fetching, check if the cache is outdated.
      let catalog = CATALOG_CACHE[url];
      if (all) {
        await catalog.fetchDeep();
      }
      return catalog;
    }

    url = new URL(url).toString();

    ui.show("Fetching catalog...", "URL: " + url);

    // Dynamic loading to allow for code splitting.
    let validateCatalog = (await import("$schemas:catalog/catalog")).validate;

    // Add protocol to the URL.
    if (!url.startsWith("https://") && !url.startsWith("http://")) {
      url = "https://" + url;
    }

    try {
      const json = await fetchJsonAndValidate(url, validateCatalog);
      const catalog = new RemoteCatalog(url, json);
      if (all) {
        await catalog.fetchDeep();
      }

      CATALOG_CACHE[url] = catalog;
      return catalog;
    } catch (e) {
      // If this is http, try with https.
      if (url.startsWith("http://")) {
        return RemoteCatalog.fetch(url.replace(/^http:\/\//, "https://"));
      } else {
        // If this doesn't end with `catalog.json`, try adding it.
        if (!url.endsWith("/catalog.json")) {
          return RemoteCatalog.fetch(
            url + (url.endsWith("/") ? "" : "/") + "catalog.json",
          );
        } else {
          throw e;
        }
      }
    }
  }

  private constructor(
    public readonly url: string,
    private readonly catalog: CatalogSchema,
  ) {}

  public get name(): string {
    return this.catalog.name;
  }

  public get version(): number | string {
    return this.catalog.version;
  }

  public get lastUpdated(): string | null {
    return this.catalog.lastUpdated || null;
  }

  public async fetchSystems(
    predicate: (
      uniqueName: string,
      system: CatalogSystemSchema,
    ) => boolean = () => true,
    deep = false,
  ): Promise<{ [name: string]: RemoteSystem }> {
    if (this.systems_ === undefined) {
      if (this.catalog.systems) {
        const entries = Object.entries(this.catalog.systems);

        const systems = await Promise.all(
          entries
            .filter(([key, system]) => predicate(key, system))
            .map(async ([key, system]) => {
              return [
                key,
                await RemoteSystem.fetch(key, system, this, deep),
              ] as [string, RemoteSystem];
            }),
        );
        this.systems_ = Object.fromEntries(systems);
      } else {
        this.systems_ = {};
      }
    }

    return this.systems_;
  }

  public async fetchDeep() {
    await this.fetchSystems(() => true, true);
  }
}
