import { sql } from "$/utils";

interface UserStorageRow {
  value: string;
}

export class DbStorage {
  static async user(id: number): Promise<DbStorage> {
    return new DbStorage(id);
  }

  static async global(): Promise<DbStorage> {
    return new DbStorage(undefined);
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
                                                   WHERE key = ${key}
                                                   LIMIT 1`;
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
      })}
                      ON CONFLICT (key)
            DO UPDATE SET value = excluded.value`;
    } else {
      await sql`INSERT INTO user_storage ${sql.insertValues({
        key,
        value: valueJson,
        user_id: this.userId,
      })}
                      ON CONFLICT (key, user_id)
            DO UPDATE SET value = excluded.value`;
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
