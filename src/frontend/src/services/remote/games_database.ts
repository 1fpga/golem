import * as oneFpgaSchema from "1fpga:schema";
import * as osd from "1fpga:osd";
import { fetchJsonAndValidate } from "$/utils";
import type {
  Games as GamesSchema,
  GamesDb as GamesDbSchema,
} from "$schemas:catalog/games_db";
import { RemoteSystem } from "./catalog";

export type RemoteGameSchema = GamesSchema[0];

/**
 * The Game identification database downloaded from a catalog.
 */
export class RemoteGamesDb {
  public static async fetch(url: string, system: RemoteSystem) {
    const u = new URL(url, system.url).toString();

    osd.show(
      "Fetching games database...",
      `Catalog "${system.catalog.name}"\nSystem "${system.name}"\nURL: ${u}`,
    );

    // Dynamic loading to allow for code splitting.
    const schema = (await import("$schemas-json:catalog/games_db.json"))
      .default;
    const json = await fetchJsonAndValidate<GamesDbSchema>(
      u,
      (value: unknown) => oneFpgaSchema.validate<GamesDbSchema>(value, schema),
    );

    return new RemoteGamesDb(u, json, system);
  }

  private constructor(
    public readonly url: string,
    private readonly schema: GamesDbSchema,
    public readonly system: RemoteSystem,
  ) {}

  get games(): GamesSchema {
    return this.schema.games ?? [];
  }
}
