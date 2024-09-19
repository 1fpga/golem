import * as golemDb from "@:golem/db";
import { getDb } from "./database";

export class LocalStorage {
  static async user(id?: number): Promise<LocalStorage> {
    let db = await getDb();
    return new LocalStorage(id, db);
  }

  private constructor(
    private readonly userId: number | undefined,
    private readonly db: golemDb.Db,
  ) {}

  async get(key: string): Promise<unknown | null> {
    let row;
    if (this.userId === undefined) {
      row = await this.db.queryOne(
        "SELECT value FROM global_storage WHERE key = ?",
        [key],
      );
    } else {
      row = await this.db.queryOne(
        "SELECT value FROM user_storage WHERE key = ? AND user_id = ?",
        [key, this.userId],
      );
    }

    return row ? JSON.parse(<string>row.value) : null;
  }

  async set(key: string, value: unknown): Promise<void> {
    const valueJson = JSON.stringify(value);
    if (this.userId === undefined) {
      await this.db.execute(
        "INSERT INTO storage (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [key, valueJson],
      );
    } else {
      await this.db.execute(
        "INSERT INTO storage (key, value, user_id) VALUES (?, ?, ?) ON CONFLICT(key, user_id) DO UPDATE SET value = excluded.value",
        [key, valueJson, this.userId],
      );
    }
  }

  async remove(key: string): Promise<void> {
    if (this.userId === undefined) {
      await this.db.execute("DELETE FROM global_storage WHERE key = ?", [key]);
    } else {
      await this.db.execute(
        "DELETE FROM user_storage WHERE key = ? AND user_id = ?",
        [key, this.userId],
      );
    }
  }
}
