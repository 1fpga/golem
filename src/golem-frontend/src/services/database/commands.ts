/**
 * This file contains the command system for Golem. It allows for the registration of commands
 * that can be executed by the user. Commands can be of two types:
 * - General: These are commands that can be executed by the user. They can have shortcuts
 *            associated with them.
 *            Example: "Show the core menu"
 * - Core: These are commands that are executed when a core is running.
 *         Example: "Quit to the main menu"
 *
 * `GeneralCommand` and `CoreCommand` are holders for the type of command they represent.
 * They do not hold any value. The value is stored in the `Shortcut` class, which isn't
 * public.
 */
import * as core from "@:golem/core";
import * as commands from "@:golem/commands";
import { sql } from "$/utils";
import { User } from "../user";

interface ShortcutRow {
  id: number;
  user_id: number;
  key: string;
  shortcut: string;
  meta: string;
}

export type GeneralCommandHandler<T> = (
  core: core.GolemCore | undefined,
  meta: T,
) => void | Promise<void>;

export type CoreCommandHandler<T> = (
  core: core.GolemCore,
  meta: T,
) => void | Promise<void>;

/**
 * The implementation of a command handler. This will only be created
 * once per command (not per shortcut), and should not contain any
 * internal state.
 */
export abstract class CommandImpl<T> {
  /**
   * The default shortcut for this command. This will be used when the user
   * first logs in and the command is created for the first time.
   */
  default: string | undefined = undefined;

  /**
   * The key that identifies this command in the database.
   * This MUST be unique.
   */
  abstract get key(): string;

  /**
   * A human-readable label for this command, generally.
   */
  abstract get label(): string;

  /**
   * The category that this command belongs to. This will be used to group
   * commands in the UI. It is case-sensitive and free form.
   */
  abstract get category(): string;

  validate(v: unknown): v is T {
    return v === undefined || v === null;
  }

  async labelOf(v: T): Promise<string> {
    if (this.label === undefined) {
      throw new Error("Label not set for command.");
    }
    return this.label;
  }

  /**
   * Inner version of execute.
   * @param core
   * @param v
   */
  abstract execute_(
    core: core.GolemCore | undefined,
    v: T,
  ): void | Promise<void>;
}

export abstract class CoreCommandImpl<T = undefined> extends CommandImpl<T> {
  execute_(core: core.GolemCore | undefined, v: T) {
    if (core) {
      return this.execute(core, v);
    }
    return Promise.resolve();
  }

  abstract execute(core: core.GolemCore, v: T): void | Promise<void>;
}

export abstract class GeneralCommandImpl<T = undefined> extends CommandImpl<T> {
  execute_(core: core.GolemCore | undefined, v: T) {
    return this.execute(core, v);
  }

  abstract execute(
    core: core.GolemCore | undefined,
    v: T,
  ): void | Promise<void>;
}

class Shortcuts {
  public static async listForCommand<Json>(
    command: CommandImpl<Json>,
  ): Promise<Shortcuts[]> {
    const user = User.loggedInUser(true);
    const rows = await sql<ShortcutRow>`
            SELECT *
            FROM shortcuts
            WHERE user_id = ${user.id}
              AND key = ${command.key}
        `;
    return rows.map((row) => new Shortcuts(row, command, JSON.parse(row.meta)));
  }

  public static async create<Json>(
    user: User,
    command: CommandImpl<Json>,
    shortcut: string,
    meta: unknown,
  ) {
    // First, verify that meta fits the validator.
    if (!command.validate(meta)) {
      throw new Error(
        `${JSON.stringify(meta)} does not match the command schema.`,
      );
    }

    const [row] = await sql<ShortcutRow>`
            INSERT INTO shortcuts ${sql.insertValues({
              user_id: user.id,
              key: command.key,
              shortcut,
              meta: JSON.stringify(meta ?? null),
            })}
                RETURNING *
        `;
    return new Shortcuts(row, command, meta);
  }

  private constructor(
    private readonly row_: ShortcutRow,
    public readonly command: CommandImpl<unknown>,
    public readonly meta: unknown,
  ) {
    commands.createShortcut(row_.shortcut, (c) => command.execute_(c, meta));
  }

  public get shortcut(): string {
    return this.row_.shortcut;
  }

  async delete() {
    await sql`
            DELETE
            FROM shortcuts
            WHERE id = ${this.row_.id}
        `;
    commands.removeShortcut(this.row_.shortcut);
  }
}

/**
 * A command that links shortcuts to a command implementation.
 */
export class Command<T = unknown> {
  public static async create<T>(
    def: CommandImpl<T>,
    firstTime = false,
  ): Promise<Command<T>> {
    const user = User.loggedInUser(true);
    const c = new Command(def, []);
    const shortcuts = await Shortcuts.listForCommand<T>(def);
    if (firstTime && shortcuts.length == 0 && def.default) {
      shortcuts.push(
        await Shortcuts.create<T>(user, def, def.default, undefined),
      );
    }
    c.shortcuts_.push(...shortcuts);

    return c;
  }

  private constructor(
    private readonly def_: CommandImpl<T>,
    private readonly shortcuts_: Shortcuts[],
  ) {}

  public is(Class: { new (): CommandImpl<T> }): boolean {
    return this.def_ instanceof Class;
  }

  get key(): string {
    return this.def_.key;
  }

  get label(): string {
    return this.def_.label;
  }

  get category(): string {
    return this.def_.category;
  }

  async labels() {
    return await Promise.all(
      this.shortcuts_.map((s) => s.command.labelOf(s.meta)),
    );
  }

  get shortcuts(): string[] {
    return this.shortcuts_.map((s) => s.shortcut);
  }

  get shortcutsWithMeta(): { shortcut: string; meta: T }[] {
    return this.shortcuts_.map((s) => ({
      shortcut: s.shortcut,
      // This has been already validated when the shortcut was created.
      meta: s.meta as T,
    }));
  }

  labelOf(meta: T) {
    return this.def_.labelOf(meta);
  }

  public async addShortcut(shortcut: string, meta?: T): Promise<void> {
    if (!this.def_.validate(meta)) {
      throw new Error("Meta does not match the command schema.");
    }

    this.shortcuts_.push(
      await Shortcuts.create(
        User.loggedInUser(true),
        this.def_,
        shortcut,
        meta,
      ),
    );
  }

  public async deleteShortcut(shortcut: string) {
    const i = this.shortcuts_.findIndex((s) => s.shortcut === shortcut);
    const maybeShortcut = this.shortcuts_[i];
    if (!maybeShortcut) {
      return;
    }
    await maybeShortcut.delete();
    this.shortcuts_.splice(i, 1);
  }

  public async execute(core: core.GolemCore | undefined, meta: T) {
    await this.def_.execute_(core, meta);
  }
}

export class Commands {
  private static def: Map<string, CommandImpl<unknown>> = new Map();
  private static commands: Map<string, Command<unknown>> = new Map();
  private static isInit: boolean = false;

  /**
   * Initialize the command subsystem;
   *  - Register shortcuts that are in the database.
   *  - Create the commands for both general and core.
   * @param user The logged-in user to use for registering commands.
   * @param firstTime Whether this is the first time the commands are being initialized.
   *                  This will set the default shortcuts for commands that have one.
   */
  public static async login(user: User, firstTime = false) {
    if (Commands.isInit) {
      throw new Error("Commands already initialized.");
    }

    await (await import("$/commands")).init();
    Commands.isInit = true;
    for (const [key, def] of Commands.def) {
      Commands.commands.set(key, await Command.create(def, firstTime));
    }
  }

  /**
   * Clear all commands and shortcuts. Used when logging out.
   */
  public static async logout() {
    if (!Commands.isInit) {
      throw new Error("Commands not initialized.");
    }

    for (const [_, command] of Commands.commands) {
      for (const shortcut of command.shortcuts) {
        commands.removeShortcut(shortcut);
      }
    }
    Commands.commands = new Map();
    Commands.def = new Map();
    Commands.isInit = false;
  }

  /**
   * Register a new command. Add the shortcuts to the system, if any.
   * @param Class The class of command to register.
   */
  public static async register<Json>(Class: { new (): CommandImpl<Json> }) {
    const def = new Class();
    if (Commands.def.has(def.key)) {
      throw new Error(
        `Command with key ${JSON.stringify(def.key)} already exists.`,
      );
    }
    Commands.def.set(def.key, def);

    if (Commands.isInit) {
      const user = User.loggedInUser(true);
      Commands.commands.set(def.key, await Command.create(def, false));
    }
  }

  /**
   * List all commands.
   */
  public static async list<T>(): Promise<Command<T>[]> {
    return Array.from(Commands.commands.values()) as Command<T>[];
  }

  public static get<T>(Class: {
    new (): CommandImpl<T>;
  }): Command<T> | undefined {
    return Array.from(Commands.commands.values()).find((c) => c.is(Class)) as
      | Command<T>
      | undefined;
  }

  public static async find<T>(
    shortcut: string,
  ): Promise<Command<T> | undefined> {
    return Array.from(Commands.commands.values()).find((c) =>
      c.shortcuts.includes(shortcut),
    ) as Command<T> | undefined;
  }

  public static async shortcutExists(shortcut: string): Promise<boolean> {
    return Array.from(Commands.commands.values()).some((c) =>
      c.shortcuts.includes(shortcut),
    );
  }
}
