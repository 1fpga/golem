import ui from "@:golem/ui";
import type { Games as GamesDbSchema } from "$schemas:catalog/games";
import { RemoteSystem } from "./catalog";
import { fetchJsonAndValidate } from "../../utils/fetch_json";

/**
 *
 */
export class RemoteGamesDb {
  public static async fetch(url: string, system: RemoteSystem) {
    const u = new URL(url, system.url).toString();

    ui.show(
      "Fetching games database...",
      `Catalog "${system.catalog.name}"\nSystem "${system.name}"\nURL: ${u}`,
    );

    // Dynamic loading to allow for code splitting.
    const json = await fetchJsonAndValidate(
      u,
      (await import("$schemas:catalog/games")).validate,
    );

    return new RemoteGamesDb(u, json, system);
  }

  private constructor(
    public readonly url: string,
    public readonly schema: GamesDbSchema,
    public readonly system: RemoteSystem,
  ) {}
}
