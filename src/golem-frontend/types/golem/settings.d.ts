// Type definitions for the `golem/settings` module in Golem Script.

/**
 * This module provides functions to interact with the settings of the Golem frontend.
 */
declare module "@:golem/settings" {
  /*
   * A command shortcut is an array of strings that represent the key sequence
   * to trigger the command. Any string can trigger a command. The string format
   * is a list of keys, buttons or axis separated by pluses, e.g. `'B' + A`
   * would trigger the command when the A button is pressed and the B key are
   * pressed at the same time.
   */
  export type CommandShortcut = string | string[];

  /**
   * Global list of commands supported by all cores.
   */
  export interface GlobalCommands {
    showMenu?: CommandShortcut;
    resetCore?: CommandShortcut;
    quitCore?: CommandShortcut;
    takeScreenshot?: CommandShortcut;
  }

  /**
   * Font size options for the UI.
   */
  export type FontSize = "small" | "medium" | "large";

  /**
   * Date and time format options for the toolbar.
   */
  export type DateTimeFormat = "default" | "short" | "timeOnly" | "hidden";

  /**
   * The settings object that can be retrieved and updated. This object also
   * matches the structure of the `settings.json` file.
   */
  export interface Settings {
    ui?: {
      menuFontSize?: FontSize;
      invertToolbar?: boolean;
      toolbarDatetimeFormat?: DateTimeFormat;
      showFps?: boolean;
      language?: string;
    };

    retronomiconBackend?: string[];

    commands?: {
      global?: GlobalCommands;
      perCore?: {
        [slug: string]: GlobalCommands | { [command: string]: CommandShortcut };
      };
    };
  }

  export function getSettings(): Settings;

  export function updateSettings(settings: Partial<Settings>): void;
}
