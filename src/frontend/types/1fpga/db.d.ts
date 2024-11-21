// Type definitions for `1fpga:db` module.

/**
 * This module provides functions for interacting with the 1FPGA SQL database.
 * It is free form, so you can execute any SQL query you want. Be careful with
 * updates and schema changes.
 */
declare module "1fpga:db" {
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
   * @returns The database object.
   */
  export function load(
    name: string,
  ): Promise<Db>;

  /**
   * Resets the database. This will delete all tables and data in the database.
   * @param name The name of the database.
   */
  export function reset(name: string): Promise<void>;

  /**
   * A queryable object that can execute SQL queries.
   */
  export interface Queryable {
    /**
     * Executes a SQL query and returns the result rows. This will not limit the number of rows
     * returned, so be careful when querying large tables.
     *
     * @param query The SQL query to execute.
     * @param bindings Optional array of values to bind to the query.
     * @returns An array of rows returned from the query.
     */
    query<T = Row>(
      query: string,
      bindings?: SqlValue[],
    ): Promise<{ rows: T[] }>;

    /**
     * Executes a SQL query and returns the first row. If no rows are returned, this will return
     * `null`.
     *
     * @param query The SQL query to execute.
     * @param bindings Optional array of values to bind to the query.
     * @returns The first row returned from the query, or `null` if no rows are returned.
     */
    queryOne<T = Row>(query: string, bindings?: SqlValue[]): Promise<T | null>;

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
     * Executes many SQL query and returns the number of rows affected. This is useful for
     * `INSERT`, `UPDATE`, and `DELETE` queries. One query per binding will be executed, as
     * fast as possible.
     * @param query
     * @param bindings
     */
    executeMany(query: string, bindings: SqlValue[][]): Promise<number>;

    /**
     * Executes a raw SQL query. This is useful for executing queries that do not return any rows,
     * such as `CREATE TABLE` or `INSERT INTO`. Be careful with this function, as it does not
     * support bindings and is susceptible to SQL injection.
     * @param query The SQL query to execute.
     */
    executeRaw(query: string): Promise<void>;
  }

  export interface Transaction extends Queryable {
    /**
     * Commits the transaction. This will unlock the database and allow other transactions to
     * modify the database.
     */
    commit(): Promise<void>;

    /**
     * Rolls back the transaction. This will unlock the database and allow other transactions to
     * modify the database.
     */
    rollback(): Promise<void>;
  }

  /**
   * The database object.
   */
  export interface Db extends Queryable {
    /**
     * Begins a transaction. This will lock the database and prevent other transactions from
     * modifying the database until the transaction is committed or rolled back.
     */
    beginTransaction(): Promise<Transaction>;
  }
}
