import { DbStorage } from "../storage";
import { User } from "../user";
import type { StartOn as StartOnSchema } from "$schemas:settings/start-on";
import { getOrFail } from "$/services/settings/utils";
import { GameSortOrder } from "$/services/database/games";

export type StartOnSetting = StartOnSchema;

/**
 * The possible values for the `startOn` setting.
 * If you update this setting, remember to also update the `start-on.json` schema.
 */
export enum StartOnKind {
  MainMenu = "main-menu",
  GameLibrary = "game-library",
  LastGamePlayed = "last-game",
  SpecificGame = "start-game",
}

const START_ON_KEY = "startOn";
const DEV_TOOLS_KEY = "devTools";
const GAME_SORT_KEY = "gameSort";
const DEFAULT_VOLUME_KEY = "defaultVolume";

export class UserSettings {
  public static async forLoggedInUser(): Promise<UserSettings> {
    const user = User.loggedInUser(true);
    const storage = await DbStorage.user(user.id);
    return new UserSettings(storage);
  }

  public static async init(user: User) {
    const storage = await DbStorage.user(user.id);
    return new UserSettings(storage);
  }

  private constructor(private readonly storage_: DbStorage) {}

  public async startOn(): Promise<StartOnSetting> {
    return await getOrFail(
      this.storage_,
      START_ON_KEY,
      {
        kind: StartOnKind.MainMenu,
      },
      (await import("$schemas:settings/start-on")).validate,
    );
  }

  public async setStartOn(value: StartOnSetting): Promise<void> {
    await this.storage_.set(START_ON_KEY, value);
  }

  public async getDevTools(): Promise<boolean> {
    return await getOrFail(this.storage_, DEV_TOOLS_KEY, false);
  }

  public async setDevTools(value: boolean): Promise<void> {
    await this.storage_.set(DEV_TOOLS_KEY, value);
  }

  public async toggleDevTools(): Promise<void> {
    await this.setDevTools(!(await this.getDevTools()));
  }

  public async getGameSort(): Promise<GameSortOrder> {
    return await getOrFail(this.storage_, GAME_SORT_KEY, GameSortOrder.NameAsc);
  }

  public async setGameSort(value: GameSortOrder): Promise<void> {
    await this.storage_.set(GAME_SORT_KEY, value);
  }

  public async defaultVolume(): Promise<number> {
    return await getOrFail(this.storage_, DEFAULT_VOLUME_KEY, 255.0);
  }

  public async setDefaultVolume(value: number): Promise<void> {
    await this.storage_.set(DEFAULT_VOLUME_KEY, value);
  }
}
