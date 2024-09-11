// Storage type definitions.

/**
 * This module provides functions for interacting with the storage.
 * It allows scripts to store and retrieve JS values from the storage.
 */
declare module "@:golem/storage" {
  export type StorageItem =
    | number
    | string
    | boolean
    | { [key: string]: StorageItem }
    | Array<StorageItem>
    | null;

  /**
   * Whether a value is in the storage.
   *
   * @param key The key to check.
   * @param user The user to check the key for. If not provided, the key is
   *             global for all users.
   * @returns `true` if the key is in the storage, `false` otherwise.
   */
  export function has(key: string, user?: string): boolean;

  /**
   * Retrieves a value from the storage.
   *
   * @param key The key to retrieve.
   * @param user The user to check the key for. If not provided, the key is
   *             global for all users.
   * @returns The value stored at the key, or `undefined` if no value is stored.
   */
  export function get(key: string, user?: string): StorageItem;

  /**
   * Stores a value in the storage.
   *
   * @param key The key to store the value at.
   * @param user The user to check the key for. If not provided, the key is
   *             global for all users.
   * @param value The value to store.
   */
  export function set(key: string, value: StorageItem, user?: string): void;

  /**
   * Removes a value from the storage.
   *
   * @param key The key to remove.
   * @param user The user to check the key for. If not provided, the key is
   *             global for all users.
   */
  export function remove(key: string, user?: string): void;
}
