import * as settings from "@:golem/settings";
import { DbStorage } from "$/services/storage";
import { getOrFail } from "$/services/settings/utils";

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
  AutoWithTz = "auto-tz",
}

const FONT_SIZE_KEY = "fontSize";
const DATETIME_FORMAT_KEY = "datetimeFormat";
const SHOW_FPS_KEY = "showFps";
const INVERT_TOOLBAR_KEY = "invertToolbar";
const TIMEZONE_KEY = "timezone";
const DATETIME_UPDATE_KEY = "datetimeUpdate";

export class GlobalSettings {
  public static async create() {
    return new GlobalSettings(await DbStorage.global());
  }

  public static async init() {
    const global = new GlobalSettings(await DbStorage.global());
    settings.setFontSize(await global.getFontSize());
    settings.setDatetimeFormat(await global.getDatetimeFormat());
    settings.setShowFps(await global.getShowFps());
    settings.setInvertToolbar(await global.getInvertToolbar());

    await global.updateDateTimeIfNecessary();
    return global;
  }

  private constructor(private readonly storage_: DbStorage) {}

  public async getFontSize(): Promise<FontSizeSetting> {
    return (await getOrFail(
      this.storage_,
      FONT_SIZE_KEY,
      "medium",
    )) as FontSizeSetting;
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
    return (await getOrFail(
      this.storage_,
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

  public async getShowFps(): Promise<boolean> {
    return await getOrFail(this.storage_, SHOW_FPS_KEY, false);
  }

  public async setShowFps(value: boolean): Promise<void> {
    await this.storage_.set(SHOW_FPS_KEY, value);
    settings.setShowFps(value);
  }

  public async toggleShowFps(): Promise<void> {
    await this.setShowFps(!(await this.getShowFps()));
  }

  public async getInvertToolbar(): Promise<boolean> {
    return await getOrFail(this.storage_, INVERT_TOOLBAR_KEY, false);
  }

  public async setInvertToolbar(value: boolean): Promise<void> {
    await this.storage_.set(INVERT_TOOLBAR_KEY, value);
    settings.setInvertToolbar(value);
  }

  public async toggleInvertToolbar(): Promise<void> {
    await this.setInvertToolbar(!(await this.getInvertToolbar()));
  }

  public async getTimeZone(d?: string) {
    return await getOrFail(this.storage_, TIMEZONE_KEY, d);
  }

  public async setTimeZone(tz: string) {
    // This will throw if the timezone is invalid.
    settings.setTimeZone(tz);
    await this.storage_.set(TIMEZONE_KEY, tz);
  }

  public async setDateTimeUpdate(value: DatetimeUpdate) {
    await this.storage_.set(DATETIME_UPDATE_KEY, value);
  }

  public async getDateTimeUpdate() {
    return await getOrFail(
      this.storage_,
      DATETIME_UPDATE_KEY,
      DatetimeUpdate.Manual,
    );
  }

  public async updateDateTimeIfNecessary() {
    const update = await this.getDateTimeUpdate();
    if (update != DatetimeUpdate.Manual) {
      let tz = undefined;
      if (update === DatetimeUpdate.AutoWithTz) {
        tz = await this.getTimeZone("UTC");
      }
      settings.updateDateTime(tz, update === DatetimeUpdate.Automatic);
    }
  }
}
