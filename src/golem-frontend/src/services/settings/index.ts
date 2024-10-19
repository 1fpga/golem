import { DbStorage } from "../storage";
import { User } from "../user";
import type { StartOn as StartOnSchema } from "$schemas:settings/start-on";

export type StartOnSetting = StartOnSchema;

/**
 * The possible values for the `startOn` setting.
 * If you update this setting, remember to also update the `start-on.json` schema.
 */
export const enum StartOn {
  MainMenu = "main-menu",
  GameLibrary = "game-library",
  LastGamePlayed = "last-game",
  SpecificGame = "start-game",
}

export type FontSizeSetting = "small" | "medium" | "large";
export const FONT_SIZES: FontSizeSetting[] = ["small", "medium", "large"];

const START_ON_KEY = "startOn";
const FONT_SIZE_KEY = "fontSize";

export class UserSettings {
  public static async forLoggedInUser(): Promise<UserSettings> {
    const user = User.loggedInUser(true);
    const storage = await DbStorage.user(user.id);
    return new UserSettings(storage);
  }

  private constructor(private readonly storage_: DbStorage) {}

  private async getOrFail<T>(
    key: string,
    d?: T,
    validator?: (v: unknown) => v is T,
  ): Promise<T> {
    let value = null;

    try {
      value = await this.storage_.get(key, validator);
    } catch (_) {
      // If anything happens, treat it as a missing value.
    }

    if (value === null) {
      if (d !== undefined) {
        return d;
      }
      throw new Error(`Missing value for key: ${key}`);
    }
    return value;
  }

  public async startOn(): Promise<StartOnSetting> {
    return await this.getOrFail(
      START_ON_KEY,
      {
        kind: StartOn.MainMenu,
      },
      (await import("$schemas:settings/start-on")).validate,
    );
  }

  public async setStartOn(value: StartOnSetting): Promise<void> {
    await this.storage_.set(START_ON_KEY, value);
  }

  public async getFontSize(): Promise<FontSizeSetting> {
    return (await this.getOrFail(FONT_SIZE_KEY, "medium")) as FontSizeSetting;
  }

  public async setFontSize(value: FontSizeSetting): Promise<void> {
    if (!FONT_SIZES.includes(value)) {
      throw new Error("Invalid font size");
    }
    await this.storage_.set(FONT_SIZE_KEY, value);
  }
}
