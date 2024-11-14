// Type definitions for the `1fpga:commands` module.

/**
 * This module provides functions to interact with commands in 1FPGA.
 * Commands can be shortcuts or scripted actions that can be executed by the user.
 *
 * Commands come in 3 variations:
 *
 * 1. a `GeneralCommand` is a command that is executed in any context, whether
 *    the user is in the main menu or in a game.
 * 2. a `CoreCommand` is a command that is executed only when the user is
 *    running a core.
 */
declare module "1fpga:commands" {
  import { OneFpgaCore } from "1fpga:core";

  /**
   * A general command action. A function that handles the command itself.
   *
   * @param core The core that is currently loaded, if any.
   */
  export type GeneralCommandAction = (
    core?: OneFpgaCore,
  ) => void | Promise<void>;

  /**
   * Create a new shortcut.
   *
   * @param shortcut The shortcut that would trigger this command.
   * @param action The action to execute when the command is called.
   * @throws If the shortcut is already in use by another command.
   */
  export function createShortcut(
    shortcut: string,
    action: GeneralCommandAction,
  ): void;

  /**
   * Remove a shortcut from the list of commands.
   * @param shortcut
   */
  export function removeShortcut(shortcut: string): void;
}
