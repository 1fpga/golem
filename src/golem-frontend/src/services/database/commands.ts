import * as core from "@:golem/core";
import * as commands from "@:golem/commands";
import { sql } from "$/utils";
import { User } from "$/services";

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

export interface GeneralCommandDef<T> {
  key: string;
  name: string;
  validator?: (v: unknown) => v is T;
  handler: GeneralCommandHandler<T>;
  default?: string;
}

export interface CoreCommandDef<T> {
  key: string;
  name: string;
  validator?: (v: unknown) => v is T;
  handler: CoreCommandHandler<T>;
  default?: string;
}

class Shortcuts {
  public static async list(): Promise<Shortcuts[]> {
    const user = User.loggedInUser(true);
    const rows = await sql<ShortcutRow>`
            SELECT *
            FROM shortcuts
            WHERE user_id = ${user.id}
        `;
    return rows.map((row) => new Shortcuts(row));
  }

  public static async listForName(key: string): Promise<Shortcuts[]> {
    const user = User.loggedInUser(true);
    const rows = await sql<ShortcutRow>`
            SELECT *
            FROM shortcuts
            WHERE user_id = ${user.id}
              AND key = ${key}
        `;
    return rows.map((row) => new Shortcuts(row));
  }

  private constructor(private readonly row_: ShortcutRow) {}

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

export abstract class BaseCommand {}

export class GeneralCommand<T> {
  public static async create<T>(
    def: GeneralCommandDef<T>,
  ): Promise<GeneralCommand<T>> {
    const shortcuts = await Shortcuts.listForName(def.key);
    return new GeneralCommand(def, shortcuts);
  }

  private constructor(
    private readonly def_: GeneralCommandDef<T>,
    private readonly shortcuts_: Shortcuts[],
  ) {
    // Register all shortcuts.
    for (const s of shortcuts_) {
      commands.createGeneralCommand(s.shortcut, (c) => {
        this.def_.handler(c, s.meta);
      });
    }
  }

  public async deleteShortcut(shortcut: string) {
    const maybeShortcut = this.shortcuts_.find((s) => s.shortcut === shortcut);
    if (!maybeShortcut) {
      return;
    }
    commands.removeCommand(shortcut);
    await maybeShortcut.delete();
  }

  public async addShortcut(shortcut: string, meta: T) {
    const user = User.loggedInUser(true);
    const maybeShortcut = this.shortcuts_.find((s) => s.shortcut === shortcut);
    if (maybeShortcut) {
      maybeShortcut.delete();
    }
    await sql`
            INSERT INTO shortcuts (user_id, key, shortcut, meta)
            VALUES (${user.id}, ${this.def_.key}, ${shortcut}, ${meta})
        `;
    commands.createGeneralCommand(shortcut, (c) => {
      this.def_.handler(c, meta);
    });
  }
}

export class Commands {
  private static general: Map<string, GeneralCommand<any>> = new Map();
  private static core: Map<string, CoreCommandDef<any>> = new Map();

  /**
   * Register a new general command. Add the shortcuts to the system, if any.
   * @param key The key that will be used to identify the shortcut in the database. This MUST be
   *            unique.
   * @param def The definition of the command.
   */
  public static async registerGeneral<Json>(
    def: GeneralCommandDef<Json>,
  ): Promise<void> {
    if (Commands.general.has(def.key)) {
      throw new Error(
        `General command with key ${JSON.stringify(def.key)} already exists.`,
      );
    }
    Commands.general.set(def.key, new GeneralCommand(def));
  }

  /**
   * Register a new core command.
   * @param key The key that will be used to identify the shortcut in the database. This MUST be
   *            unique.
   * @param def The definition of the command.
   */
  public static async registerCoreCommand<Json>(
    def: CoreCommandDef<Json>,
  ): Promise<void> {
    if (Commands.core.has(def.key)) {
      throw new Error(
        `Core command with key ${JSON.stringify(def.key)} already exists.`,
      );
    }
    Commands.core.set(def.key, new CoreCommand(def));
  }
}

// Basic commands.
Commands.registerCoreCommand({
  key: "resetCore",
  name: "Reset Core",
  async handler(core) {
    core.reset();
  },
});
