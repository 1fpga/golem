// Type definitions for the `@:fs` module in Golem Script.

/**
 * File system functions for Golem Script.
 */
declare module "@:fs" {
  export function writeFile(path: string, data: string): Promise<void>;
  export function writeFile(path: string, data: Uint8Array): Promise<void>;
  export function writeFile(path: string, data: ArrayBuffer): Promise<void>;

  export function readFile(path: string): Promise<ArrayBuffer>;
}
