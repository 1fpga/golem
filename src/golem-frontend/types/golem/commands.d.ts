// Type definitions for the `@/golem/commands` module in Golem Script.

/**
 * This module provides functions to interact with commands in Golem Script.
 * Commands can be shortcuts or scripted actions that can be executed by the user.
 *
 * Commands come in 3 variations:
 *
 * 1. a `GeneralCommand` is a command that is executed in any context, whether
 *    the user is in the main menu or in a game.
 * 2. a `CoreCommand` is a command that is executed only when the user is
 *    running a core.
 * 3. a `CoreSpecificCommand` is a command that is executed only when the user
 *    is running a specific core.
 */
declare module "@:golem/commands" {
  import { GolemCore } from "@:golem/core";

  /**
   * A command type containing all information about a command.
   */
  export type Command = GeneralCommand | CoreCommand | CoreSpecificCommand;

  /**
   * A command definition. Contains all information about a command.
   */
  export type CommandDef = (
    | (Omit<GeneralCommand, "shortcuts" | "execute"> & {
        action: GeneralCommandAction;
      })
    | (Omit<CoreCommand, "shortcuts" | "execute"> & {
        action: CoreCommandAction;
      })
    | (Omit<CoreSpecificCommand, "shortcuts" | "execute"> & {
        action: CoreSpecificCommandAction;
      })
  ) & { defaultShortcuts?: string[] };

  /**
   * A base interface for all commands.
   */
  export interface CommandBase {
    shortName: string;
    name: string;
    description: string;

    /**
     * Execute the command. Will throw if the command is not valid in the
     * current context (e.g. trying to execute a core command when not running
     * a core).
     * @throws If the command is not valid in the current context.
     */
    execute(): void;

    get shortcuts(): string[];

    set shortcuts(shortcuts: string[]);
  }

  export interface GeneralCommand extends CommandBase {
    type: "general";
  }

  export interface CoreCommand extends CommandBase {
    type: "core";
  }

  export interface CoreSpecificCommand extends CommandBase {
    type: "coreSpecific";
    core: string;
  }

  /**
   * A general command action. A function that handles the command itself.
   */
  export type GeneralCommandAction = (core?: GolemCore) => void | Promise<void>;

  /**
   * A core command action. A function that handles the command itself.
   */
  export type CoreCommandAction = (core: GolemCore) => void | Promise<void>;

  /**
   * A core specific command action. A function that handles the command itself.
   */
  export type CoreSpecificCommandAction = (
    core: GolemCore,
  ) => void | Promise<void>;

  /**
   * List all commands available.
   */
  export function listCommands(): Command[];

  /**
   * Find a command by its short name.
   *
   * @param shortName The short name of the command.
   * @returns The command, or `undefined` if not found.
   */
  export function findCommand(shortName: string): Command | undefined;

  /**
   * Create a new general command.
   *
   * @param shortName The short name of the command. MUST BE UNIQUE.
   * @param name The name of the command.
   * @param description The description of the command.
   * @param action The action to execute when the command is called.
   * @returns The created general command.
   * @throws If the short name is already in use.
   */
  export function createGeneralCommand(
    shortName: string,
    name: string,
    description: string,
    action: GeneralCommandAction,
  ): GeneralCommand;

  /**
   * Create a new core command.
   *
   * @param shortName The short name of the command. MUST BE UNIQUE.
   * @param name The name of the command.
   * @param description The description of the command.
   * @param action The action to execute when the command is called.
   * @returns The created core command.
   * @throws If the short name is already in use.
   */
  export function createCoreCommand(
    shortName: string,
    name: string,
    description: string,
    action: CoreCommandAction,
  ): CoreCommand;

  /**
   * Create a new core specific command.
   *
   * @param shortName The short name of the command. MUST BE UNIQUE.
   * @param name The name of the command.
   * @param description The description of the command.
   * @param core The core to bind the command to.
   * @param action The action to execute when the command is called.
   * @returns The created core specific command.
   * @throws If the short name is already in use.
   */
  export function createCoreSpecificCommand(
    shortName: string,
    name: string,
    description: string,
    core: string,
    action: CoreSpecificCommandAction,
  ): CoreSpecificCommand;
}
