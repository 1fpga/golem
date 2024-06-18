// Type definitions for the `golem/settings` module in Golem Script.

/**
 * This module provides functions to interact with the settings of the Golem frontend.
 */
declare module "@/golem/settings" {
  export type CommandShortcut = string;

  export interface GlobalCommands {
    showMenu: CommandShortcut;
    resetCore: CommandShortcut;
    takeScreenshot: CommandShortcut;
  }

  export interface Settings {
    ui?: {
      menuFontSize?: "small" | "medium" | "large";
      invertToolbar?: boolean;
      toolbarDatetimeFormat?:
        | "hidden"
        | "default"
        | "short"
        | "timeOnly"
        | string;
      showFps?: boolean;
      language?: string;
    };

    retronomiconBackend?: string[];

    commands?: {
      global?: GlobalCommands;
      perCore?: GlobalCommands;
    };
  }

  export function getSettings(): Settings;

  export function updateSettings(settings: Partial<Settings>): void;
}
