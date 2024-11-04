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
  meta: any;
}

export type GeneralCommandHandler<T> = (
  core: core.GolemCore | undefined,
  meta: T,
) => void | Promise<void>;

export type CoreCommandHandler<T> = (
  core: core.GolemCore,
  meta: T,
) => void | Promise<void>;

export interface CommandDef<T> {
  type: "general" | "core";
  key: string;
  name: string;
  category: string;
  default?: string;
  validator?: (v: unknown) => v is T;
  labelOf?: (meta: T) => string | Promise<string>;
}

export interface GeneralCommandDef<T> extends CommandDef<T> {
  type: "general";
  handler: GeneralCommandHandler<T>;
}

export interface CoreCommandDef<T> extends CommandDef<T> {
  type: "core";
  handler: CoreCommandHandler<T>;
}

class Shortcuts {
  public static async listForCommand<
    Json,
    C extends BaseCommand<Json, CommandDef<Json>> = BaseCommand<
      Json,
      CommandDef<Json>
    >,
  >(command: C): Promise<Shortcuts[]> {
    const user = User.loggedInUser(true);
    const rows = await sql<ShortcutRow>`
            SELECT *
            FROM shortcuts
            WHERE user_id = ${user.id}
              AND key = ${command.key}
        `;
    return rows.map((row) => new Shortcuts(row, command, row.meta));
  }

  public static async create<
    Json,
    C extends BaseCommand<Json, CommandDef<Json>> = BaseCommand<
      Json,
      CommandDef<Json>
    >,
  >(user: User, command: C, shortcut: string, meta: unknown) {
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
    public readonly command: BaseCommand<any, CommandDef<any>>,
    meta: unknown,
  ) {
    commands.createShortcut(row_.shortcut, (c) => command.execute(c, meta));
  }

  public get shortcut(): string {
    return this.row_.shortcut;
  }

  public get meta(): any {
    return this.row_.meta;
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

export abstract class BaseCommand<T, Def extends CommandDef<T>> {
  protected constructor(
    protected readonly def_: Def,
    protected shortcuts_: Shortcuts[],
  ) {}

  get key(): string {
    return this.def_.key;
  }

  get name(): string {
    return this.def_.name;
  }

  get category(): string {
    return this.def_.category;
  }

  /**
   * Get a list of all shortcuts related to this command.
   */
  get shortcuts(): string[] {
    return this.shortcuts_.map((s) => s.shortcut);
  }

  public get shortcutsWithMeta(): [string, T][] {
    return this.shortcuts_.map((s) => [s.shortcut, s.meta]);
  }

  public validate(v: unknown): v is T {
    if (this.def_.validator) {
      return this.def_.validator(v);
    }
    return v === undefined || v === null;
  }

  async addShortcut(shortcut: string, meta: T): Promise<void> {
    const user = User.loggedInUser(true);
    this.shortcuts_.push(
      await Shortcuts.create<T, BaseCommand<T, CommandDef<T>>>(
        user,
        this,
        shortcut,
        meta,
      ),
    );
  }

  public async deleteShortcut(shortcut: string) {
    const maybeShortcut = this.shortcuts_.find((s) => s.shortcut === shortcut);
    if (!maybeShortcut) {
      return;
    }
    await maybeShortcut.delete();
    this.shortcuts_ = this.shortcuts_.filter((s) => s.shortcut !== shortcut);
  }

  abstract execute(core: core.GolemCore | undefined, meta: T): void;
}

export class GeneralCommand<T> extends BaseCommand<T, GeneralCommandDef<T>> {
  public static async create<T>(
    user: User,
    def: GeneralCommandDef<T>,
    firstTime: boolean,
  ): Promise<GeneralCommand<T>> {
    const c = new GeneralCommand(def, []);
    let shortcuts = await Shortcuts.listForCommand<T>(c);
    if (firstTime && shortcuts.length == 0 && def.default) {
      shortcuts.push(await Shortcuts.create<T>(user, c, def.default, {}));
    }
    c.shortcuts_ = shortcuts;

    return c;
  }

  public execute(core: core.GolemCore | undefined, meta: T) {
    this.def_.handler(core, meta);
  }
}

export class CoreCommand<T> extends BaseCommand<T, CoreCommandDef<T>> {
  public static async create<T>(
    user: User,
    def: CoreCommandDef<T>,
    firstTime: boolean,
  ): Promise<CoreCommand<T>> {
    const c = new CoreCommand(def, []);
    let shortcuts = await Shortcuts.listForCommand<T>(c);
    if (firstTime && shortcuts.length == 0 && def.default) {
      shortcuts.push(await Shortcuts.create<T>(user, c, def.default, {}));
    }
    c.shortcuts_ = shortcuts;
    return c;
  }

  public execute(core: core.GolemCore | undefined, meta: T) {
    if (core) {
      this.def_.handler(core, meta);
    }
  }
}

export class Commands {
  private static def: Map<string, CommandDef<any>> = new Map();

  private static commands: Map<
    string,
    BaseCommand<unknown, CommandDef<unknown>>
  > = new Map();
  private static isInit: boolean = false;

  private static async createCommandFromDef<T>(
    user: User,
    key: string,
    def: CommandDef<T>,
    firstTime: boolean,
  ) {
    switch (def.type) {
      case "general":
        Commands.commands.set(
          key,
          await GeneralCommand.create(
            user,
            def as GeneralCommandDef<any>,
            firstTime,
          ),
        );
        break;
      case "core":
        Commands.commands.set(
          key,
          await CoreCommand.create(user, def as CoreCommandDef<any>, firstTime),
        );
        break;
    }
  }

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
      await Commands.createCommandFromDef(user, key, def, firstTime);
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
   * @param key The key that will be used to identify the shortcut in the database. This MUST be
   *            unique.
   * @param def The definition of the command.
   */
  public static register<Json>(def: GeneralCommandDef<Json>): Promise<void>;
  public static register<Json>(def: CoreCommandDef<Json>): Promise<void>;
  public static async register<Json>(def: CommandDef<Json>) {
    if (Commands.def.has(def.key)) {
      throw new Error(
        `Command with key ${JSON.stringify(def.key)} already exists.`,
      );
    }
    Commands.def.set(def.key, def);

    if (Commands.isInit) {
      const user = User.loggedInUser(true);
      await Commands.createCommandFromDef(user, def.key, def, false);
    }
  }

  /**
   * List all commands.
   */
  public static async list<T>(): Promise<BaseCommand<T, CommandDef<T>>[]> {
    return Array.from(Commands.commands.values());
  }
}
