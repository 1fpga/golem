// Type definitions for `golem/db` module in Golem Script.

/**
 * This module provides functions for interacting with the Golem SQL database.
 * It is free form, so you can execute any SQL query you want. Be careful with
 * updates and schema changes.
 */
declare module "@:golem/db" {
  /**
   * A value that can be bound to or returned from a SQL query. This can be a string,
   * number, boolean, or NULL.
   */
  export type SqlValue = string | number | boolean | null;

  /**
   * A row returned from a SQL query. This is an object where the keys are the column
   * names and the values are the column values.
   */
  export type Row = { [field: string]: SqlValue };

  /**
   * Gets a database object for the given database name. This will create the database if it
   * does not exist. Applies migrations if specified.
   * @param name The name of the database.
   * @param options Optional options for the database.
   * @returns The database object.
   */
  export function load(
    name: string,
    options?: { migrations?: string },
  ): Promise<Db>;

  /**
   * The database object.
   */
  export interface Db {
    /**
     * Executes a SQL query and returns the result rows. This will not limit the number of rows
     * returned, so be careful when querying large tables.
     *
     * @param query The SQL query to execute.
     * @param bindings Optional array of values to bind to the query.
     * @returns An array of rows returned from the query.
     */
    query(query: string, bindings?: SqlValue[]): Promise<{ rows: Row[] }>;

    /**
     * Executes a SQL query and returns the first row. If no rows are returned, this will return
     * `null`.
     *
     * @param query The SQL query to execute.
     * @param bindings Optional array of values to bind to the query.
     * @returns The first row returned from the query, or `null` if no rows are returned.
     */
    queryOne(query: string, bindings?: SqlValue[]): Promise<Row | null>;

    /**
     * Executes a SQL query and returns the number of rows affected. This is useful for `INSERT`,
     * `UPDATE`, and `DELETE` queries.
     *
     * @param query The SQL query to execute.
     * @param bindings Optional array of values to bind to the query.
     * @returns The first column of the first row returned from the query, or `null` if no rows are returned.
     */
    execute(query: string, bindings?: SqlValue[]): Promise<number>;

    /**
     * Executes a raw SQL query. This is useful for executing queries that do not return any rows,
     * such as `CREATE TABLE` or `INSERT INTO`. Be careful with this function, as it does not
     * support bindings and is susceptible to SQL injection.
     * @param query The SQL query to execute.
     */
    executeRaw(query: string): Promise<void>;
  }
}
