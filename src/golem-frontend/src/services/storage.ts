import * as golemDb from "@:golem/db";
import { sql } from "./database";

interface UserStorageRow {
  value: string;
}

export class LocalStorage {
  static async user(id: number): Promise<LocalStorage> {
    return new LocalStorage(id);
  }

  static async global(): Promise<LocalStorage> {
    return new LocalStorage(undefined);
  }

  private constructor(private readonly userId: number | undefined) {}

  async get<T>(
    key: string,
    validator?: (v: unknown) => v is T,
  ): Promise<T | null> {
    let value: string | undefined;
    if (this.userId === undefined) {
      const rows = await sql<UserStorageRow>`SELECT value
                                                   FROM global_storage
                                                   WHERE key = ${key}LIMIT 1`;
      value = rows[0]?.value;
    } else {
      let rows = await sql<UserStorageRow>`
                SELECT value
                FROM user_storage
                WHERE key = ${key}
                  AND user_id = ${this.userId}
                LIMIT 1`;
      value = rows[0]?.value;
    }

    if (value === null || value === undefined) {
      return null;
    }
    const json = JSON.parse(value);
    if (validator && !validator(json)) {
      throw new Error(`Invalid value schema: key=${JSON.stringify(key)}`);
    }
    return json as T;
  }

  async set<T>(key: string, value: T): Promise<void> {
    const valueJson = JSON.stringify(value);
    if (this.userId === undefined) {
      await sql`INSERT INTO global_storage ${sql.insertValues({
        key,
        value: valueJson,
      })}`;
    } else {
      await sql`INSERT INTO user_storage ${sql.insertValues({
        key,
        value: valueJson,
        user_id: this.userId,
      })}`;
    }
  }

  async remove(key: string): Promise<void> {
    if (this.userId === undefined) {
      await sql`DELETE
                      FROM global_storage
                      WHERE key = ${key}`;
    } else {
      await sql`DELETE
                      FROM user_storage
                      WHERE key = ${key}
                        AND user_id = ${this.userId}`;
    }
  }
}
