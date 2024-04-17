// Storage type definitions.

/**
 * This module provides functions for interacting with the storage.
 * It allows scripts to store and retrieve JS values from the storage.
 */
declare module "golem/storage" {
    /**
     * Retrieves a value from the storage.
     *
     * @param key The key to retrieve.
     * @returns The value stored at the key, or `undefined` if no value is stored.
     */
    export function get(key: string): any;

    /**
     * Stores a value in the storage.
     *
     * @param key The key to store the value at.
     * @param value The value to store.
     */
    export function set(key: string, value: any): void;

    /**
     * Removes a value from the storage.
     *
     * @param key The key to remove.
     */
    export function remove(key: string): void;
}

