import { compareVersions, RemoteCore, RemoteSystem } from "../remote";
import { Row } from "@:golem/db";
import * as net from "@:golem/net";
import * as ui from "@:golem/ui";
import { Catalog } from "./catalog";
import type { System } from "./system";
import { sql } from "../database";

export interface CoreRow extends Row {
  id: number;
  system_id: number;
  catalog_id: number;
  name: string;
  unique_name: string;
  rbf_path: string | null;
  description: string;
  version: string | null;
  icon_path: string | null;
  image_path: string | null;
}

/**
 * A Core is a game engine that can be used to run games.
 * This strictly deals with cores from the database. Downloading, parsing
 * and validating cores is done by {@link RemoteCore}.
 */
export class Core {
  private static fromRow(row: CoreRow | null): Core {
    if (row === null) {
      throw new Error("Core not found");
    }

    return new Core(row);
  }

  public static pathForAsset(core: RemoteCore, version: string) {
    return `/media/fat/golem/cores/${core.uniqueName}/${version}`;
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

  static async upgrade(core: RemoteCore, system: System, catalog: Catalog) {
    throw new Error("Method not implemented.");
  }

  static async install(
    core: RemoteCore,
    system: System,
    catalog: Catalog,
    version?: string,
  ) {
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
        return await Core.upgrade(core, system, catalog);
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
    await sql`INSERT INTO cores ${sql.insertValues({
      system_id: system.id,
      catalog_id: catalog.id,
      name: core.name,
      unique_name: core.uniqueName,
      rbf_path: rbf,
      description: core.description,
      version: release.version,
      icon_path: core.iconPath,
      image_path: core.imagePath,
    })}`;
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
}
