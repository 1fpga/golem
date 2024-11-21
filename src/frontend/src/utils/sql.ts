import production from "consts:production";
import * as oneFpgaDb from "1fpga:db";
import { SqlTag, type SqlTagDriver } from "@sqltags/core";
import { MigrationDetails } from "1fpga:migrations";

async function applyMigrations(
  db: oneFpgaDb.Db,
  _name: string,
  latest: string
) {
  const migrations = (await import("1fpga:migrations")).migrations;
  const allMigrations: [string, MigrationDetails][] =
    Object.getOwnPropertyNames(migrations)
      .filter((m) => m.localeCompare(latest) > 0)
      .sort()
      .map(
        (m) =>
          migrations[m].up &&
          ([m, migrations[m].up] as [string, MigrationDetails])
      )
      .filter((m) => m !== undefined);

  if (allMigrations.length === 0) {
    console.debug(`Latest migration: ${latest}, no migrations to apply.`);
    return;
  }

  console.log(
    `Latest migration: ${latest}, applying ${allMigrations.length} migrations...`
  );

  const sql1 = await transaction();
  for (const [name, up] of allMigrations) {
    console.debug(`Applying ${name}...`);
    const { sql, apply } = up;

    // Start a transaction so everything is in a single transaction.
    try {
      await sql1.db.executeRaw(sql);
      if (apply) {
        await apply(sql1);
      }
    } catch (e) {
      await sql1.rollback();
      console.error(`Error applying migration ${name}: ${e}`);
      throw e;
    }

    await sql1`INSERT INTO __1fpga_settings ${sql1.insertValues({
        key: "latest_migration",
        value: name
    })}
               ON CONFLICT (key)
    DO UPDATE SET value = excluded.value`;
  }
  await sql1.commit();

  console.log(
    "Migrations applied. Latest migration: ",
    (
      await sql<{ value: string }>`SELECT value
                                   FROM __1fpga_settings
                                   WHERE key = 'latest_migration'`
    )[0]?.value
  );
}

async function createMigrationTable(
  db: oneFpgaDb.Db,
  name: string
): Promise<void> {
  await sql`CREATE TABLE __1fpga_settings
            (
                id    INTEGER PRIMARY KEY,
                key   TEXT NOT NULL UNIQUE,
                value TEXT NOT NULL
            )`;

  await applyMigrations(db, name, "");
}

async function initDb(db: oneFpgaDb.Db, name: string): Promise<void> {
  // Check if the migrations table exists.
  let migrationsTableExists = await sql`SELECT 1
                                        FROM sqlite_schema
                                        WHERE type = 'table'
                                          AND name = '__1fpga_settings'`;

  if (migrationsTableExists.length === 0) {
    await createMigrationTable(db, name);
  } else {
    const [{ value: latestMigration }] = await sql<{
      value: string;
    }>`SELECT value
       FROM __1fpga_settings
       WHERE key = 'latest_migration'`;

    await applyMigrations(db, name, latestMigration || "");
  }
}

let db: oneFpgaDb.Db | null = null;

export async function resetDb(): Promise<void> {
  console.warn("Clearing the database. Be careful!");
  await oneFpgaDb.reset("1fpga");
  db = null;
}

async function getDb(): Promise<oneFpgaDb.Db> {
  if (db === null) {
    db = await oneFpgaDb.load("1fpga");
    await initDb(db, "1fpga");
  }

  return db;
}

const driver: SqlTagDriver<undefined, never> = {
  cursor(
    sql: string,
    params: any[],
    options: {} | undefined
  ): AsyncIterable<any> {
    throw new Error("Method not implemented.");
  },
  escapeIdentifier(identifier: string): string {
    return `"${identifier.replace(/"/g, "\"\"")}"`;
  },
  parameterizeValue(value: any, paramIndex: number): string {
    return "?";
  },
  async query(sql: string, params: any[]): Promise<[any[], undefined]> {
    if (!production) {
      console.log(sql, "|", JSON.stringify(params));
    }

    let db = await getDb();
    let result = await db.query(sql, params);
    return [result.rows, undefined];
  }
};

export const sql = new SqlTag(driver);

export interface SqlTransactionTag extends SqlTag<undefined, never> {
  commit(): Promise<void>;

  rollback(): Promise<void>;

  db: oneFpgaDb.Queryable;
}

export async function transaction(): Promise<SqlTransactionTag> {
  const db = await (await getDb()).beginTransaction();

  const tag = new SqlTag({
    ...driver,
    async query(sql: string, params: any[]): Promise<[any[], undefined]> {
      if (!production) {
        console.log("tx", sql, "|", JSON.stringify(params));
      }
      let { rows } = await db.query(sql, params);
      return [rows, undefined];
    }
  }) as SqlTransactionTag;

  tag.commit = async () => {
    await db.commit();
  };
  tag.rollback = async () => {
    await db.rollback();
  };
  tag.db = db;
  return tag;
}
