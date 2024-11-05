import * as core from "@:golem/core";
import { compareVersions, RemoteCore } from "../remote";
import { Row } from "@:golem/db";
import * as net from "@:golem/net";
import * as ui from "@:golem/ui";
import { Catalog } from "./catalog";
import { System } from "./system";
import { sql } from "$/utils";
import { coreOsdMenu } from "$/ui/menus/core_osd";

export interface CoreRow extends Row {
  id: number;
  system_id: number;
  catalog_id: number;
  name: string;
  unique_name: string;
  rbf_path: string | null;
  description: string;
  version: string | null;
}

/**
 * A Core is a game engine that can be used to run games.
 * This strictly deals with cores from the database. Downloading, parsing
 * and validating cores is done by {@link RemoteCore}.
 */
export class Core {
  private static runningCore: Core | null = null;

  private static fromRow(row: CoreRow | null): Core {
    if (row === null) {
      throw new Error("Core not found");
    }

    return new Core(row);
  }

  public static pathForAsset(core: RemoteCore, version: string) {
    return `/media/fat/golem/cores/${core.uniqueName}/${version}`;
  }

  public static running() {
    return Core.runningCore;
  }

  public static setRunning(core: Core | null) {
    Core.runningCore = core;
  }

  public static async getById(id: number): Promise<Core | null> {
    const [row] = await sql<CoreRow>`SELECT *
                                         FROM cores
                                         WHERE id = ${id}
                                         LIMIT 1`;
    return Core.fromRow(row);
  }

  public static async count(system?: System): Promise<number> {
    const [{ count }] = await sql<{ count: number }>`SELECT COUNT(*) as count
                                                       FROM cores ${
                                                         system
                                                           ? sql`WHERE system_id =
                                                                       ${system.id}`
                                                           : undefined
                                                       }`;

    return count;
  }

  public static async listForCatalogId(catalogId: number): Promise<Core[]> {
    const rows = await sql<CoreRow>`SELECT *
                                        FROM cores
                                        WHERE catalog_id = ${catalogId}`;

    return rows.map(Core.fromRow);
  }

  public static async list(system?: System): Promise<Core[]> {
    const rows = await sql<CoreRow>`SELECT *
                                        FROM cores ${
                                          system
                                            ? sql`WHERE system_id =
                                                        ${system.id}`
                                            : undefined
                                        }`;

    return rows.map(Core.fromRow);
  }

  static async upgrade(core: RemoteCore, catalog: Catalog): Promise<Core> {
    throw new Error("Method not implemented.");
  }

  static async install(core: RemoteCore, catalog: Catalog, version?: string) {
    let [maybeCore] = await sql<CoreRow>`SELECT *
                                             FROM cores
                                             WHERE unique_name = ${core.uniqueName}
                                             LIMIT 1`;

    let release = core.latestRelease;
    if (version !== undefined) {
      const maybeRelease = core.releases.find((r) => r.version === version);
      if (maybeRelease) {
        release = maybeRelease;
      }
    }

    if (maybeCore) {
      // Maybe we need to upgrade it?
      const existingCore = Core.fromRow(maybeCore);
      if (compareVersions(release.version, existingCore.version) > 0) {
        return await Core.upgrade(core, catalog);
      }
    }

    console.log("Installing core:", core.uniqueName);

    // First download the core files
    const root = Core.pathForAsset(core, `${release.version}`);
    const files = release.files;
    ui.show(
      "Downloading core...",
      `Core "${core.name}"\nVersion "${release.version}"`,
    );

    let rbf = null;
    for (const file of files) {
      const path = await net.download(
        new URL(file.url, core.url).toString(),
        root,
      );
      if (file.type === "mister.core.rbf") {
        rbf = path;
      }
    }

    console.debug("Adding core to database");
    let [row] = await sql<CoreRow>`INSERT INTO cores ${sql.insertValues({
      catalog_id: catalog.id,
      name: core.name,
      unique_name: core.uniqueName,
      rbf_path: rbf,
      description: core.description,
      version: release.version,
    })} RETURNING *`;

    console.debug("Adding systems to core");
    for (const system of core.systems) {
      const s = await System.getByUniqueName(system);
      if (s === null) {
        throw new Error("System not found");
      }

      await sql`INSERT INTO cores_systems ${sql.insertValues({
        core_id: row.id,
        system_id: s.id,
      })}`;
    }

    console.debug("Adding core to games database");
    if (core.tags.includes("no-roms")) {
      await sql`INSERT INTO games ${sql.insertValues({
        name: core.name,
        core_id: row.id,
      })}`;
    }

    return Core.fromRow(row);
  }

  private constructor(private readonly row: CoreRow) {}

  public async getSystem(): Promise<System> {
    const { System } = await import("./system");
    const s = await System.getById(this.row.system_id);
    if (!s) {
      throw new Error("System not found");
    }
    return s;
  }

  public get id() {
    return this.row.id;
  }

  public get uniqueName() {
    return this.row.unique_name;
  }

  public get name() {
    return this.row.name;
  }

  public get description() {
    return this.row.description;
  }

  public get version() {
    return this.row.version;
  }

  public get rbfPath() {
    return this.row.rbf_path;
  }

  public async launch() {
    if (!this.rbfPath) {
      throw new Error("Core does not have an RBF path");
    }

    try {
      console.log(`Starting core: ${JSON.stringify(this)}`);
      Core.setRunning(this);
      let c = core.load({
        core: { type: "Path", path: this.rbfPath },
      });

      let error = undefined;
      c.showOsd(async () => {
        try {
          return await coreOsdMenu(c, this);
        } catch (e) {
          error = e;
          return true;
        }
      });
      if (error) {
        throw error;
      }

      c.loop();
    } finally {
      Core.setRunning(null);
    }
  }
}
