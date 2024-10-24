import { DbStorage } from "../storage";
import { User } from "../user";
import type { StartOn as StartOnSchema } from "$schemas:settings/start-on";
import * as settings from "@:golem/settings";

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

export type DatetimeFormatSetting = "default" | "short" | "timeOnly" | "hidden";
export const DATETIME_FORMATS: settings.DateTimeFormat[] = [
  "default",
  "short",
  "timeOnly",
  "hidden",
];

export const enum DatetimeUpdate {
  Automatic = "auto",
  Manual = "manual",
}

const START_ON_KEY = "startOn";
const FONT_SIZE_KEY = "fontSize";
const DATETIME_FORMAT_KEY = "datetimeFormat";
const DEV_TOOLS_KEY = "devTools";
const SHOW_FPS_KEY = "showFps";
const INVERT_TOOLBAR_KEY = "invertToolbar";
const TIMEZONE_KEY = "timezone";
const DATETIME_UPDATE_KEY = "datetimeUpdate";

export class UserSettings {
  public static async forLoggedInUser(): Promise<UserSettings> {
    const user = User.loggedInUser(true);
    const storage = await DbStorage.user(user.id);
    return new UserSettings(storage);
  }

  public static async init(user: User) {
    const storage = await DbStorage.user(user.id);
    const result = new UserSettings(storage);
    settings.setFontSize(await result.getFontSize());
    settings.setDatetimeFormat(await result.getDatetimeFormat());
    settings.setShowFps(await result.getShowFps());
    settings.setInvertToolbar(await result.getInvertToolbar());

    return result;
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

  public async toggleFontSize() {
    const current = await this.getFontSize();
    const currentIndex = FONT_SIZES.indexOf(current);
    const nextIndex = (currentIndex + 1) % FONT_SIZES.length;
    await this.setFontSize(FONT_SIZES[nextIndex]);
  }

  public async setFontSize(value: FontSizeSetting): Promise<void> {
    if (!FONT_SIZES.includes(value)) {
      throw new Error("Invalid font size");
    }
    await this.storage_.set(FONT_SIZE_KEY, value);
    settings.setFontSize(value);
  }

  public async getDatetimeFormat(): Promise<DatetimeFormatSetting> {
    return (await this.getOrFail(
      DATETIME_FORMAT_KEY,
      "default",
    )) as DatetimeFormatSetting;
  }

  public async toggleDatetimeFormat() {
    const current = await this.getDatetimeFormat();
    const currentIndex = DATETIME_FORMATS.indexOf(current);
    const nextIndex = (currentIndex + 1) % DATETIME_FORMATS.length;
    await this.setDatetimeFormat(DATETIME_FORMATS[nextIndex]);
  }

  public async setDatetimeFormat(value: DatetimeFormatSetting): Promise<void> {
    if (!DATETIME_FORMATS.includes(value)) {
      throw new Error("Invalid datetime format");
    }
    await this.storage_.set(DATETIME_FORMAT_KEY, value);
    settings.setDatetimeFormat(value);
  }

  public async getDevTools(): Promise<boolean> {
    return await this.getOrFail(DEV_TOOLS_KEY, false);
  }

  public async setDevTools(value: boolean): Promise<void> {
    await this.storage_.set(DEV_TOOLS_KEY, value);
  }

  public async toggleDevTools(): Promise<void> {
    await this.setDevTools(!(await this.getDevTools()));
  }

  public async getShowFps(): Promise<boolean> {
    return await this.getOrFail(SHOW_FPS_KEY, false);
  }

  public async setShowFps(value: boolean): Promise<void> {
    await this.storage_.set(SHOW_FPS_KEY, value);
    settings.setShowFps(value);
  }

  public async toggleShowFps(): Promise<void> {
    await this.setShowFps(!(await this.getShowFps()));
  }

  public async getInvertToolbar(): Promise<boolean> {
    return await this.getOrFail(INVERT_TOOLBAR_KEY, false);
  }

  public async setInvertToolbar(value: boolean): Promise<void> {
    await this.storage_.set(INVERT_TOOLBAR_KEY, value);
    settings.setInvertToolbar(value);
  }

  public async toggleInvertToolbar(): Promise<void> {
    await this.setInvertToolbar(!(await this.getInvertToolbar()));
  }

  public async getTimeZone(d?: string) {
    return await this.getOrFail(TIMEZONE_KEY, d);
  }

  public async setTimeZone(tz: string) {
    await this.storage_.set(TIMEZONE_KEY, tz);
  }

  public async setDateTimeUpdate(value: DatetimeUpdate) {
    await this.storage_.set(DATETIME_UPDATE_KEY, value);
  }

  public async getDateTimeUpdate() {
    return await this.getOrFail(DATETIME_UPDATE_KEY, DatetimeUpdate.Manual);
  }
}
