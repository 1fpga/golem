import * as osd from "1fpga:osd";
import type { Catalog as CatalogSchema } from "$schemas:catalog/catalog";
import type { System as SystemSchema } from "$schemas:catalog/system";
import type {
  Core as CoresCoreSchema,
  Cores as CoresSchema,
} from "$schemas:catalog/cores";
import type {
  System as SystemsSystemSchema,
  Systems as SystemsSchema,
} from "$schemas:catalog/systems";
import type { Core as CoreSchema } from "$schemas:catalog/core";
import { RemoteGamesDb } from "$/services/remote/games_database";
import { fetchJsonAndValidate, ValidationError } from "$/utils";
import { RemoteBinary, RemoteReleases } from "$/services/remote/release";
import { compareVersions } from "$/utils/versions";

export enum WellKnownCatalogs {
  // The basic stable 1FPGA catalog.
  OneFpga = "https://catalog.1fpga.cloud/",

  // The BETA 1FPGA catalog (not yet available).
  OneFpgaBeta = "https://catalog.1fpga.cloud/beta.json",

  // Only exists in development mode.
  LocalTest = "http://catalog.local:8081/catalog.json",
}

const CATALOG_CACHE: { [url: string]: RemoteCatalog } = {};

/**
 * A remote system array. This is a `systems.json` that is fetched from the internet,
 * parsed and validated. It contains the list of all systems.
 */
export class RemoteSystems {
  /**
   * The system cache.
   * @private
   */
  private systems_: { [k: string]: RemoteSystem } = Object.create(null);

  public static async fetch(
    url: string,
    catalog: RemoteCatalog,
    _deep = false,
  ) {
    const systemsUrl = new URL(url, catalog.url).toString();
    const systems = await fetchJsonAndValidate(
      systemsUrl,
      (await import("$schemas:catalog/systems")).validate,
    );
    return new RemoteSystems(systemsUrl, systems, catalog);
  }

  constructor(
    public readonly url: string,
    public readonly schema: SystemsSchema,
    public readonly catalog: RemoteCatalog,
  ) {}

  public async fetchSystem(key: string, deep = false) {
    if (!this.systems_[key]) {
      this.systems_[key] = await RemoteSystem.fetch(
        key,
        new URL(this.schema[key].url, this.url).toString(),
        this,
        deep,
      );
    }
    return this.systems_[key];
  }
}

/**
 * A remote core array. This is a `cores.json` that is fetched from the internet,
 * parsed and validated. It contains the list of all cores. Each core is a `RemoteCore`.
 */
export class RemoteCores {
  /**
   * The core cache.
   * @private
   */
  private cores_: { [k: string]: RemoteCore } = Object.create(null);

  public static async fetch(
    url: string,
    catalog: RemoteCatalog,
    _deep = false,
  ) {
    const coresUrl = new URL(url, catalog.url).toString();
    const cores = await fetchJsonAndValidate(
      coresUrl,
      (await import("$schemas:catalog/cores")).validate,
    );
    return new RemoteCores(coresUrl, cores, catalog);
  }

  constructor(
    public readonly url: string,
    public readonly schema: CoresSchema,
    public readonly catalog: RemoteCatalog,
  ) {}

  public async fetchCore(key: string, _deep = false) {
    if (!this.cores_[key]) {
      this.cores_[key] = await RemoteCore.fetch(
        key,
        new URL(this.schema[key].url, this.url).toString(),
        this,
      );
    }
    return this.cores_[key];
  }
}

/**
 * A remote core is a `core.json` that is fetched from the internet,
 * parsed and validated.
 */
export class RemoteCore {
  public static async fetch(key: string, url: string, cores: RemoteCores) {
    const u = new URL(url, cores.url).toString();

    osd.show(
      "Fetching core...",
      `Catalog "${cores.catalog.name}"\nCore: ${key}\nURL: ${u}`,
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

    return new RemoteCore(u, json, cores);
  }

  constructor(
    public readonly url: string,
    public readonly schema: CoreSchema,
    public readonly cores: RemoteCores,
  ) {}

  get catalog() {
    return this.cores.catalog;
  }

  get name(): string {
    return this.schema.name || this.schema.uniqueName;
  }

  get systems(): string[] {
    if (typeof this.schema.systems === "string") {
      return [this.schema.systems];
    } else {
      return this.schema.systems || [];
    }
  }

  get uniqueName() {
    return this.schema.uniqueName;
  }

  get tags() {
    return this.schema.tags || [];
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
}

/**
 * A remote system array is a `system.json` that is fetched from the internet,
 * parsed and validated.
 */
export class RemoteSystem {
  public static pathForAsset(system: RemoteSystem, assetType: string) {
    return `/media/fat/1fpga/systems/${system.uniqueName}/${assetType}`;
  }

  public static async fetch(
    key: string,
    url: string,
    systems: RemoteSystems,
    _deep = false,
  ): Promise<RemoteSystem> {
    const u = new URL(url, systems.url).toString();

    osd.show(
      "Fetching system...",
      `Catalog ${systems.catalog.name}\nURL: ${u}`,
    );

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
    return new RemoteSystem(u, json, systems);
  }

  constructor(
    public readonly url: string,
    public readonly schema: SystemSchema,
    private systems_: RemoteSystems,
  ) {}

  get catalog(): RemoteCatalog {
    return this.systems_.catalog;
  }

  get name(): string {
    return this.schema.name || this.schema.uniqueName;
  }

  get uniqueName(): string {
    return this.schema.uniqueName;
  }

  get description(): string | null {
    return this.schema.description || null;
  }

  get size(): number {
    return this.schema.gamesDb?.size || 0;
  }

  get tags(): string[] {
    return this.schema.tags || [];
  }

  async downloadGameDatabase() {
    if (!this.schema.gamesDb) {
      return;
    }

    return await RemoteGamesDb.fetch(this.schema.gamesDb.url, this);
  }
}

/**
 * A remote catalog is the `catalog.json` that is fetched from the internet,
 * parsed and validated.
 */
export class RemoteCatalog {
  private systems_: RemoteSystems | undefined;
  private cores_: RemoteCores | undefined;
  private releases_: RemoteReleases | undefined;

  public static async clearCache() {
    for (const key of Object.keys(CATALOG_CACHE)) {
      delete CATALOG_CACHE[key];
    }
  }

  public static async fetchWellKnown(
    wellKnown: WellKnownCatalogs,
  ): Promise<RemoteCatalog> {
    return RemoteCatalog.fetch(wellKnown);
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

    osd.show("Fetching catalog...", "URL: " + url);

    // Add protocol to the URL.
    if (!url.startsWith("https://") && !url.startsWith("http://")) {
      url = "https://" + url;
    }

    try {
      // Dynamic loading to allow for code splitting. And also don't allow for retries
      // since we're already in a retry loop.
      const json = await fetchJsonAndValidate(
        url,
        (await import("$schemas:catalog/catalog")).validate,
        { allowRetry: false },
      );
      const catalog = new RemoteCatalog(url, json);
      if (all) {
        await catalog.fetchDeep();
      }

      CATALOG_CACHE[url] = catalog;
      return catalog;
    } catch (e) {
      if (e instanceof ValidationError) {
        throw e;
      }

      console.error("Error fetching catalog:", (e as any)?.message || e);

      // If this is http, try with https.
      // If this doesn't end with `catalog.json`, try adding it.
      if (!url.endsWith("/catalog.json")) {
        return RemoteCatalog.fetch(new URL("catalog.json", url).toString());
      } else if (url.startsWith("http://")) {
        return RemoteCatalog.fetch(url.replace(/^http:\/\//, "https://"));
      } else {
        throw e;
      }
    }
  }

  private constructor(
    public readonly url: string,
    public readonly schema: CatalogSchema,
  ) {}

  public get name(): string {
    return this.schema.name;
  }

  public get version(): number | string {
    return this.schema.version;
  }

  public get lastUpdated(): string | null {
    return this.schema.lastUpdated || null;
  }

  public async fetchSystems(
    predicate: (
      uniqueName: string,
      system: SystemsSystemSchema,
    ) => boolean = () => true,
    deep = false,
  ): Promise<{ [k: string]: RemoteSystem }> {
    // If there's no cores, return an empty array.
    if (this.schema.systems === undefined) {
      return {};
    }

    if (this.systems_ === undefined) {
      this.systems_ = await RemoteSystems.fetch(this.schema.systems.url, this);
    }

    return Object.fromEntries(
      (
        await Promise.all(
          Object.entries(this.systems_.schema)
            .filter(([name, system]) => predicate(name, system))
            .map(async ([name, _system]) => [
              name,
              await this.systems_?.fetchSystem(name, deep),
            ]),
        )
      ).filter(([_, s]) => s !== undefined),
    );
  }

  public async fetchCores(
    predicate: (uniqueName: string, core: CoresCoreSchema) => boolean = () =>
      true,
    deep = false,
  ): Promise<{ [k: string]: RemoteCore }> {
    // If there's no cores, return an empty array.
    if (this.schema.cores === undefined) {
      return {};
    }

    if (this.cores_ === undefined) {
      this.cores_ = await RemoteCores.fetch(this.schema.cores.url, this);
    }

    return Object.fromEntries(
      (
        await Promise.all(
          Object.entries(this.cores_.schema)
            .filter(([name, core]) => predicate(name, core))
            .map(async ([name, _core]) => [
              name,
              await this.cores_?.fetchCore(name, deep),
            ]),
        )
      ).filter(([_, c]) => c !== undefined),
    );
  }

  public async fetchReleases(
    predicate: (name: string) => boolean = () => true,
  ): Promise<{
    [name: string]: RemoteBinary;
  }> {
    if (this.schema.releases === undefined) {
      return {};
    }
    if (this.releases_ === undefined) {
      this.releases_ = await RemoteReleases.fetch(
        this.schema.releases.url,
        this,
      );
    }

    return this.releases_.asObject(predicate);
  }

  public async fetchDeep() {
    await Promise.all([
      this.fetchReleases(() => true),
      this.fetchSystems(() => true, true),
      this.fetchCores(() => true, true),
    ]);
  }
}
