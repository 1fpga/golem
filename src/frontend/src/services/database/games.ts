import * as oneFpgaCore from "1fpga:core";
import * as fs from "1fpga:fs";
import { sql } from "$/utils";
import { oneLine } from "common-tags";
import { User } from "../user";
import { PickGameOptions } from "$/ui/games";
import { Core } from "$/services/database/core";

interface GamesCoreRow {
  id: number;
  name: string;
  rom_path: string | null;
  rbf_path: string;
  system_name: string;
  cores_id: number;
  favorite: boolean | null;
  last_played_at: Date | null;
}

export enum GameSortOrder {
  NameAsc = "name ASC",
  NameDesc = "name DESC",
  SystemAsc = "systems.unique_name ASC",
  LastPlayed = "user_games.last_played_at DESC",
  Favorites = "user_games.favorite DESC, user_games.last_played_at DESC",
}

export interface GamesListOptions {
  /**
   * The sort order.
   */
  sort?: GameSortOrder;

  limit?: number;

  index?: number;

  /**
   * Merge games with the same game identification.
   */
  mergeByGameId?: boolean;
  includeUnplayed?: boolean;
  includeUnfavorites?: boolean;
  system?: string;
}

const GAMES_FIELDS = oneLine`
  games.id AS id,
  games.path AS rom_path,
  IFNULL(cores_2.rbf_path, cores.rbf_path) AS rbf_path,
  IFNULL(games_identification.name, games.name) AS name,
  systems.unique_name AS system_name,
  user_games.favorite,
  user_games.last_played_at,
  IFNULL(user_games.cores_id, cores_systems.core_id) AS cores_id
`;

const GAMES_FROM_JOIN = oneLine`
  games
    LEFT JOIN games_identification ON games.games_id = games_identification.id
    LEFT JOIN systems AS systems_2 ON games_identification.system_id = systems_2.id
    LEFT JOIN cores AS cores_2 ON games.core_id = cores_2.id
    LEFT JOIN cores_systems ON cores_2.id = cores_systems.core_id OR games.core_id = cores_systems.core_id OR systems_2.id = cores_systems.system_id
    LEFT JOIN cores ON games.core_id = cores.id OR cores_systems.core_id = cores.id
    LEFT JOIN systems ON games_identification.system_id = systems.id OR cores_systems.system_id = systems.id
    LEFT JOIN user_games ON user_games.games_id = games.id
`;

const GROUP_BY_GAME_ID = oneLine`
    GROUP BY IFNULL(games_identification.id, cores.rbf_path)
`;

function buildSqlQuery(options: GamesListOptions) {
  return sql`
      SELECT ${sql.raw(GAMES_FIELDS)}
      FROM ${sql.raw(GAMES_FROM_JOIN)}
      WHERE ${sql.and(
        true,
        options.system
          ? sql`systems.unique_name =
                      ${options.system}`
          : undefined,
        (options.includeUnplayed ?? true)
          ? undefined
          : sql`user_games.last_played_at IS NOT NULL`,
        (options.includeUnfavorites ?? true)
          ? undefined
          : sql`user_games.favorite = true`,
      )}
      ORDER BY ${sql.raw(options.sort ?? GameSortOrder.NameAsc)}
      LIMIT ${options.limit ?? 100} OFFSET ${options.index ?? 0}
  `;
}

interface SaveStateRow {
  id: number;
  core_id: number;
  game_id: number;
  user_id: number;
  state_path: string;
  screenshot_path: string;
  created_at: Date;
}

export class SaveState {
  private static fromRow(row: SaveStateRow): SaveState {
    return new SaveState(row);
  }

  public static async create(
    game: Games,
    savestate: Uint8Array,
    screenshot: Image,
  ): Promise<SaveState> {
    const user = User.loggedInUser(true);
    const state_path = `/media/fat/1fpga/savestates/${user.id}/${game.systemName}/${game.name} ${Date.now()}.ss`;
    const screenshot_path = `/media/fat/1fpga/savestates/${user.id}/${game.systemName}/${game.name} ${Date.now()}.png`;

    await fs.writeFile(state_path, savestate);
    await screenshot.save(screenshot_path);

    const [row] =
      await sql<SaveStateRow>`INSERT INTO savestates ${sql.insertValues({
        core_id: game.coreId,
        game_id: game.id,
        user_id: 0,
        state_path,
        screenshot_path,
      })}
                                  RETURNING *`;
    return SaveState.fromRow(row);
  }

  constructor(private readonly row_: SaveStateRow) {}
}

export class Games {
  private static fromGamesCoreRow(row: GamesCoreRow): Games {
    return new Games(row);
  }

  public static async select(options?: PickGameOptions): Promise<Games | null> {
    return await (await import("$/ui/games")).pickGame(options);
  }

  public static async count(options: GamesListOptions): Promise<number> {
    const [{ count }] = await sql<{ count: number }>`
        SELECT COUNT(*) as count
        FROM (${buildSqlQuery(options)})
    `;
    return count;
  }

  public static async byId(id: number): Promise<Games> {
    const [row] = await sql<GamesCoreRow>`
        SELECT ${sql.raw(GAMES_FIELDS)}
        FROM ${sql.raw(GAMES_FROM_JOIN)}
        WHERE games.id = ${id}
    `;
    return Games.fromGamesCoreRow(row);
  }

  public static async lastPlayed(): Promise<Games | null> {
    const [row] = await sql<GamesCoreRow>`
        SELECT ${sql.raw(GAMES_FIELDS)}
        FROM ${sql.raw(GAMES_FROM_JOIN)}
        WHERE user_games.last_played_at IS NOT NULL
        ORDER BY user_games.last_played_at DESC
        LIMIT 1
    `;
    return row ? Games.fromGamesCoreRow(row) : null;
  }

  public static async first() {
    const [row] = await sql<GamesCoreRow>`
        SELECT ${sql.raw(GAMES_FIELDS)}
        FROM ${sql.raw(GAMES_FROM_JOIN)}
        LIMIT 1
    `;
    return row ? Games.fromGamesCoreRow(row) : null;
  }

  public static async list(
    options: GamesListOptions = {},
  ): Promise<{ total: number; games: Games[] }> {
    const games = await sql<GamesCoreRow>`${buildSqlQuery(options)}`;
    const total = await Games.count(options);

    return { total, games: games.map(Games.fromGamesCoreRow) };
  }

  constructor(private readonly row_: GamesCoreRow) {}

  get id(): number {
    return this.row_.id;
  }

  get name(): string {
    return this.row_.name;
  }

  get systemName(): string {
    return this.row_.system_name;
  }

  get coreId(): number {
    return this.row_.cores_id;
  }

  get romPath(): string | null {
    return this.row_.rom_path;
  }

  get favorite(): boolean | null {
    return this.row_.favorite;
  }

  async setFavorite(favorite: boolean) {
    if (this.row_.favorite !== favorite) {
      await sql`INSERT INTO user_games
                    ${sql.insertValues({
                      user_id: User.loggedInUser(true).id,
                      games_id: this.id,
                      favorite,
                    })}
                ON CONFLICT
      DO
      UPDATE SET favorite = excluded.favorite`;
    }
    this.row_.favorite = favorite;
  }

  get lastPlayedAt(): Date | null {
    return this.row_.last_played_at;
  }

  async launch() {
    console.log("Launching game: ", JSON.stringify(this.row_));

    // Insert last played time at.
    await sql`INSERT INTO user_games
                  ${sql.insertValues({
                    user_id: User.loggedInUser(true).id,
                    games_id: this.id,
                    last_played_at: "" + new Date(),
                  })}
              ON CONFLICT
    DO
    UPDATE SET last_played_at = excluded.last_played_at`;

    const settings = await (
      await import("$/services/settings/user")
    ).UserSettings.forLoggedInUser();

    try {
      Core.setRunning(await Core.getById(this.row_.cores_id));
      const core = oneFpgaCore.load({
        core: { type: "Path", path: this.row_.rbf_path },
        ...(this.row_.rom_path
          ? { game: { type: "RomPath", path: this.row_.rom_path } }
          : {}),
      });

      if (core) {
        console.log("Starting core: " + core.name);
        core.volume = await settings.defaultVolume();
        core.on(
          "saveState",
          async (savestate: Uint8Array, screenshot: Image) => {
            const ss = SaveState.create(this, savestate, screenshot);
            console.log("Saved state: ", JSON.stringify(ss));
          },
        );
        core.loop();
      }
    } finally {
      Core.setRunning(null);
    }
  }
}
