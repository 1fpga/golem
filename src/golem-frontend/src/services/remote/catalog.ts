import * as net from "@:golem/net";
import * as ui from "@:golem/ui";

import type {
  Catalog as CatalogSchema,
  System as CatalogSystemSchema,
} from "$schemas:catalog/catalog";
import type { System as SystemSchema } from "$schemas:catalog/system";
import type { Core as CoreSchema } from "$schemas:catalog/core";
import type { ValidateFunction } from "ajv";

/**
 * Fetch a JSON file from the internet and validate it.
 * @param url The URL to fetch.
 * @param validate The validation function to use.
 * @returns The parsed JSON.
 */
async function fetchJson<T>(
  url: string,
  validate: ValidateFunction<T>,
): Promise<T> {
  const response = await net.fetchJson(url);
  if (validate(response)) {
    return response;
  } else {
    throw new Error(
      (validate.errors || []).map((e) => e.message || "").join("\n"),
    );
  }
}

/**
 * A remote core is a `core.json` that is fetched from the internet,
 * parsed and validated.
 */
export class RemoteCore {
  public static async fetch(key: string, url: string, system: RemoteSystem) {
    const u = new URL(url, system.url).toString();

    ui.show(
      `Catalog "${system.catalog.name}"\nSystem "${system.name}"\nFetching core...\n${u}`,
    );

    // Dynamic loading to allow for code splitting.
    const validate = (await import("$schemas:catalog/core")).validate;
    const json = await fetchJson(u, validate);

    return new RemoteCore(u, json, system);
  }

  constructor(
    public readonly url: string,
    public readonly core: CoreSchema,
    public readonly system: RemoteSystem,
  ) {}
}

/**
 * A remote system array is a `system.json` that is fetched from the internet,
 * parsed and validated.
 */
export class RemoteSystem {
  private cores_: { [k: string]: RemoteCore } | undefined;

  public static async fetch(
    key: string,
    schema: CatalogSystemSchema,
    catalog: RemoteCatalog,
    deep = false,
  ): Promise<RemoteSystem> {
    const u = new URL(schema.url, catalog.url).toString();

    ui.show(`Catalog ${catalog.name}\nFetching system...\n${u}`);

    // Dynamic loading to allow for code splitting.
    const validate = (await import("$schemas:catalog/system")).validate;
    const json = await fetchJson(u, validate);

    if (key !== json.uniqueName) {
      throw new Error(
        `System name mismatch: ${JSON.stringify(key)} != ${JSON.stringify(json.uniqueName)}`,
      );
    }
    const system = new RemoteSystem(u, json, catalog);

    if (deep) {
      await system.fetchCores(deep);
    }

    return system;
  }

  constructor(
    public readonly url: string,
    public readonly system: SystemSchema,
    private catalog_: RemoteCatalog,
  ) {}

  get name(): string {
    return this.system.name || this.system.uniqueName;
  }

  get catalog(): RemoteCatalog {
    return this.catalog_;
  }

  async fetchCores(deep = false) {
    if (this.cores_ === undefined) {
      if (this.system.cores) {
        const entries = Object.entries(this.system.cores);
        const cores = await Promise.all(
          entries.map(async ([key, core]) => {
            return [key, await RemoteCore.fetch(key, core.url, this)] as [
              string,
              RemoteCore,
            ];
          }),
        );
      } else {
        this.cores_ = {};
      }
    }
    return this.cores_;
  }
}

/**
 * A remote catalog is the `catalog.json` that is fetched from the internet,
 * parsed and validated.
 */
export class RemoteCatalog {
  private systems_: { [key: string]: RemoteSystem } | undefined;

  public static async fetch1Fpga(): Promise<RemoteCatalog> {
    return RemoteCatalog.fetch("https://golem.1fpga.cloud");
  }

  public static async fetch(url: string, all = false): Promise<RemoteCatalog> {
    ui.show("Fetching catalog...\n" + url);

    // Dynamic loading to allow for code splitting.
    let validateCatalog = (await import("$schemas:catalog/catalog")).validate;

    // Add protocol to the URL.
    if (!url.startsWith("https://") && !url.startsWith("http://")) {
      url = "https://" + url;
    }

    try {
      const json = await fetchJson(url, validateCatalog);
      const catalog = new RemoteCatalog(url, json);
      if (all) {
        await catalog.fetchDeep();
      }
      return catalog;
    } catch (e) {
      // If this is http, try with https.
      if (url.startsWith("http://")) {
        return RemoteCatalog.fetch(url.replace(/^http:\/\//, "https://"));
      } else {
        throw e;
      }
    }
  }

  private constructor(
    public readonly url: string,
    public readonly catalog: CatalogSchema,
  ) {}

  public get name(): string {
    return this.catalog.name;
  }

  public async fetchSystems(
    predicate: (name: string, system: CatalogSystemSchema) => boolean = () =>
      true,
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
