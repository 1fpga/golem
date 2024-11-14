import { SqlTag } from "@sqltags/core";

export interface MigrationDetails {
  /**
   * SQL to be executed.
   */
  sql: string;
  /**
   * Function to be executed
   * after the SQL is executed.
   */
  apply: (db: SqlTag<unknown, unknown>) => Promise<void>;
}

export interface Migration {
  up?: MigrationDetails;
}

/**
 * List of up migrations to be applied.
 */
export const migrations: {
  [key: string]: Migration;
};
