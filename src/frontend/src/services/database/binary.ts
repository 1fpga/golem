import { Row } from "1fpga:db";
import { Catalog } from "./catalog";
import { compareVersions, sql } from "$/utils";
import { RemoteBinary } from "../remote";

export interface BinaryRow extends Row {
  id: number;
  catalog_id: number;
  name: string;
  version: string;
  update_pending: boolean;
}

export class Binary {
  private static fromRow(row: BinaryRow): Binary {
    return new Binary(
      row.id,
      row.catalog_id,
      row.name,
      row.version,
      row.update_pending,
    );
  }

  public static async checkForUpdates() {
    const binaries = await Binary.listBinaries();
    let shouldUpdate = false;
    for (const binary of binaries) {
      // If an update is already pending, skip looking if we need another one.
      if (binary.updatePending) {
        continue;
      }

      const remote = await binary.fetchRemote();
      if (
        compareVersions(remote.latestVersion()?.version, binary.version) > 0
      ) {
        await sql`UPDATE catalog_binaries
                          SET update_pending = true
                          WHERE id = ${binary.id}`;
        shouldUpdate = true;
      }
    }

    return shouldUpdate;
  }

  static async listBinaries(catalog?: Catalog, sql1 = sql): Promise<Binary[]> {
    const rows = await sql1<BinaryRow>`
            SELECT *
            FROM catalog_binaries ${
              catalog &&
              sql1`WHERE catalog_id =
                    ${catalog.id}`
            }`;
    return rows.map((row) => Binary.fromRow(row));
  }

  static async create(
    remote: RemoteBinary,
    catalog: Catalog,
    sql1 = sql,
  ): Promise<Binary> {
    const version = await remote.getInstalledVersion();
    const updatePending =
      compareVersions(remote.latestVersion()?.version, version) > 0;
    const [row] = await sql1<BinaryRow>`
            INSERT INTO catalog_binaries ${sql1.insertValues({
              catalog_id: catalog.id,
              name: remote.name,
              ...(version ? { version } : {}),
              update_pending: updatePending,
            })}
                RETURNING *`;
    return row && Binary.fromRow(row);
  }

  constructor(
    public readonly id: number,
    public readonly catalogId: number,
    public readonly name: string,
    public readonly version: string,
    private updatePending_: boolean,
  ) {}

  get updatePending() {
    return this.updatePending_;
  }

  public async fetchRemote(): Promise<RemoteBinary> {
    const catalog = await this.getCatalog();
    if (!catalog) {
      throw new Error("Catalog not found");
    }
    const remote = await catalog.fetchRemote();
    const releases = await remote.fetchReleases((name) => this.name === name);
    const release = releases[this.name];
    if (!release) {
      throw new Error("Release not found");
    }

    return release;
  }

  public getCatalog(): Promise<Catalog | null> {
    return Catalog.getById(this.catalogId);
  }

  /**
   * Clean up the version by setting its updatePending to false.
   */
  async clean() {
    await sql`UPDATE catalog_binaries
                  SET update_pending = false
                  WHERE id = ${this.id}`;
    this.updatePending_ = false;
  }
}
