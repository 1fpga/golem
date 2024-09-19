import * as golemDb from "@:golem/db";

let db: golemDb.Db | null = null;

export async function getDb(): Promise<golemDb.Db> {
    if (db === null) {
        db = await golemDb.load("1fpga");
    }

    return db;
}
