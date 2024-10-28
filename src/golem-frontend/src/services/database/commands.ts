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
  validator?: (v: unknown) => v is T;
  default?: string;
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
  public static async listForCommand<C extends BaseCommand>(
    command: C,
  ): Promise<Shortcuts[]> {
    const user = User.loggedInUser(true);
    const rows = await sql<ShortcutRow>`
            SELECT *
            FROM shortcuts
            WHERE user_id = ${user.id}
              AND key = ${command.key}
        `;
    console.log(5, JSON.stringify(rows));
    return rows.map((row) => new Shortcuts(row, command));
  }

  public static async create<C extends BaseCommand>(
    user: User,
    command: C,
    shortcut: string,
    meta: any,
  ) {
    const [row] = await sql<ShortcutRow>`
            INSERT INTO shortcuts ${sql.insertValues({
              user_id: user.id,
              key: command.key,
              shortcut,
              meta: JSON.stringify(meta ?? null),
            })}
                RETURNING *
        `;
    return new Shortcuts(row, command);
  }

  private constructor(
    private readonly row_: ShortcutRow,
    public readonly command: BaseCommand,
  ) {}

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
  }
}

export abstract class BaseCommand<
  T = unknown,
  Def extends CommandDef<T> = CommandDef<T>,
> {
  protected constructor(protected readonly def_: Def) {}

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
  abstract get shortcuts(): string[];

  abstract addShortcut(shortcut: string, meta: T): Promise<void>;

  abstract deleteShortcut(shortcut: string): Promise<void>;
}

export class GeneralCommand<T> extends BaseCommand<T, GeneralCommandDef<T>> {
  public static async create<T>(
    user: User,
    def: GeneralCommandDef<T>,
    firstTime: boolean,
  ): Promise<GeneralCommand<T>> {
    const c = new GeneralCommand(def);

    let shortcuts = await Shortcuts.listForCommand(c);
    if (firstTime && shortcuts.length == 0 && def.default) {
      shortcuts.push(await Shortcuts.create(user, c, def.default, {}));
    }
    c.shortcuts_ = shortcuts;

    // Register all shortcuts.
    for (const s of shortcuts) {
      console.log(`0 Creating shortcut ${s.shortcut}`);
      commands.createShortcut(s.shortcut, (c) => def.handler(c, s.meta));
    }

    return c;
  }

  private constructor(
    def_: GeneralCommandDef<T>,
    private shortcuts_: Shortcuts[] = [],
  ) {
    console.log(4, JSON.stringify(def_), JSON.stringify(shortcuts_));
    super(def_);
    // Register all shortcuts.
    for (const s of shortcuts_) {
      console.log(`0 Creating shortcut ${s.shortcut}`);
      commands.createShortcut(s.shortcut, (c) => def_.handler(c, s.meta));
    }
  }

  public async deleteShortcut(shortcut: string) {
    const maybeShortcut = this.shortcuts_.find((s) => s.shortcut === shortcut);
    if (!maybeShortcut) {
      return;
    }
    commands.removeShortcut(shortcut);
    await maybeShortcut.delete();
    this.shortcuts_ = this.shortcuts_.filter((s) => s.shortcut !== shortcut);
  }

  public async addShortcut(shortcut: string, meta: T) {
    const user = User.loggedInUser(true);
    const maybeShortcut = this.shortcuts_.find((s) => s.shortcut === shortcut);
    if (maybeShortcut) {
      // We cannot keep a consistent state if we try to delete it from the database here
      // but not from the command instance it's from.
      throw new Error("Shortcut already exists.");
    }
    this.shortcuts_.push(await Shortcuts.create(user, this, shortcut, meta));
    console.log(`1 Creating shortcut ${shortcut}`);
    commands.createShortcut(shortcut, (c) => {
      this.def_.handler(c, meta);
    });
  }

  public get shortcuts(): string[] {
    return this.shortcuts_.map((s) => s.shortcut);
  }
}

export class CoreCommand<T> extends BaseCommand<T, CoreCommandDef<T>> {
  public static async create<T>(
    user: User,
    def: CoreCommandDef<T>,
    firstTime: boolean,
  ): Promise<CoreCommand<T>> {
    const c = new CoreCommand(def, []);
    let shortcuts = await Shortcuts.listForCommand(c);
    if (firstTime && shortcuts.length == 0 && def.default) {
      shortcuts.push(await Shortcuts.create(user, c, def.default, {}));
    }
    c.shortcuts_ = shortcuts;

    // Register all shortcuts.
    for (const s of shortcuts) {
      commands.createShortcut(s.shortcut, (c) => {
        if (c) {
          def.handler(c, s.meta);
        }
      });
    }

    return c;
  }

  private constructor(
    def_: CoreCommandDef<T>,
    private shortcuts_: Shortcuts[],
  ) {
    super(def_);

    // Register all shortcuts.
    for (const s of shortcuts_) {
      commands.createShortcut(s.shortcut, (c) => {
        if (c) {
          this.def_.handler(c, s.meta);
        }
      });
    }
  }

  public async deleteShortcut(shortcut: string) {
    const maybeShortcut = this.shortcuts_.find((s) => s.shortcut === shortcut);
    if (!maybeShortcut) {
      return;
    }
    commands.removeShortcut(shortcut);
    await maybeShortcut.delete();
    this.shortcuts_ = this.shortcuts_.filter((s) => s.shortcut !== shortcut);
  }

  public async addShortcut(shortcut: string, meta: T) {
    const user = User.loggedInUser(true);
    // If the shortcut doesn't exist, this does nothing.
    await this.deleteShortcut(shortcut);

    this.shortcuts_.push(await Shortcuts.create(user, this, shortcut, meta));
    console.log(`3 Creating shortcut ${shortcut}`);
    commands.createShortcut(shortcut, (c) => {
      if (c) {
        this.def_.handler(c, meta);
      }
    });
  }

  public get shortcuts(): string[] {
    return this.shortcuts_.map((s) => s.shortcut);
  }
}

export class Commands {
  private static def: Map<string, CommandDef<any>> = new Map();

  private static commands: Map<string, BaseCommand<any>> = new Map();
  private static isInit: boolean = false;

  private static async createCommandFromDef(
    user: User,
    key: string,
    def: CommandDef<unknown>,
    firstTime: boolean,
  ) {
    switch (def.type) {
      case "general":
        console.log(0, JSON.stringify(def));
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
        console.log(1, JSON.stringify(def));
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
  public static async init(user: User, firstTime = false) {
    console.log("Initializing commands.");
    if (Commands.isInit) {
      throw new Error("Commands already initialized.");
    }
    Commands.isInit = true;

    await (await import("$/commands")).init();
    for (const [key, def] of Commands.def) {
      console.log(`Creating command ${key}`);
      await Commands.createCommandFromDef(user, key, def, firstTime);
    }
  }

  /**
   * Clear all commands and shortcuts. Used when logging out.
   */
  public static async logout() {
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
  public static registerCommand<Json, T extends GeneralCommandDef<Json>>(
    def: T,
  ): Promise<void>;
  public static registerCommand<Json, T extends CoreCommandDef<Json>>(
    def: T,
  ): Promise<void>;
  public static async registerCommand<Json, T extends CommandDef<Json>>(
    def: T,
  ) {
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
  public static async list(): Promise<BaseCommand<unknown>[]> {
    return Array.from(Commands.commands.values());
  }
}
