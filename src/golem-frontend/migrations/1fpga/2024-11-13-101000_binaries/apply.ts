import { SqlTag } from "@sqltags/core";
import { Binary, Catalog, RemoteCatalog } from "$/services";

export async function up(sql: SqlTag<unknown, unknown>) {
  // We need to get the catalogs, download their remote versions and update the database
  // with their binaries.
  const catalogs = await Catalog.listCatalogs();

  for (const c of catalogs) {
    const remote = await RemoteCatalog.fetch(c.url);
    const releases = await remote.fetchReleases();

    for (const name of Object.getOwnPropertyNames(releases)) {
      const binary = releases[name];
      if (binary) {
        await Binary.create(binary, c);
      }
    }
  }
}
