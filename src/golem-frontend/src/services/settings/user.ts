import { DbStorage } from "../storage";
import { User } from "../user";
import type { StartOn as StartOnSchema } from "$schemas:settings/start-on";
import { getOrFail } from "$/services/settings/utils";

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
}
