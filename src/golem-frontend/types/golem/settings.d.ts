// Type definitions for the `golem/settings` module in Golem Script.

/**
 * This module provides functions to interact with the settings of the Golem frontend.
 */
declare module "@:golem/settings" {
  /**
   * Font size options for the UI.
   */
  export type FontSize = "small" | "medium" | "large";

  /**
   * Date and time format options for the toolbar.
   */
  export type DateTimeFormat = "default" | "short" | "timeOnly" | "hidden";

  export function fontSize(): FontSize;

  export function setFontSize(fontSize: FontSize): void;

  export function datetimeFormat(): DateTimeFormat;

  export function setDatetimeFormat(format: DateTimeFormat): void;

  export function showFps(): boolean;

  export function setShowFps(show: boolean): void;

  export function invertToolbar(): boolean;

  export function setInvertToolbar(invert: boolean): void;

  /**
   * Ping the NTP server and update the current time.
   */
  export function updateDateTime(): void;

  /**
   * Get a list of all available timezones.
   */
  export function listTimeZones(): string[];

  /**
   * Get the timezone to use.
   */
  export function getTimeZone(): string | null;

  /**
   * Set the timezone to use.
   * @param timeZone The timezone to use, or null to not change the system time zone.
   */
  export function setTimeZone(timeZone: string): void;

  /**
   * Manually set the date and time.
   * @param dateTime The date and time to set.
   */
  export function setDateTime(dateTime: Date): void;
}
