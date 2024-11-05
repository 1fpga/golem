// Type definitions for the `@:golem/fs` module in Golem Script.

/**
 * File system functions for Golem Script.
 */
declare module "@:golem/fs" {
  export function writeFile(path: string, data: string): Promise<void>;
  export function writeFile(path: string, data: Uint8Array): Promise<void>;
  export function writeFile(path: string, data: ArrayBuffer): Promise<void>;

  export function readFile(path: string): Promise<ArrayBuffer>;

  export function readTextFile(path: string): Promise<string>;

  export function deleteFile(path: string): Promise<void>;

  export function readDir(path: string): Promise<string[]>;

  export function isFile(path: string): Promise<boolean>;

  export function isDir(path: string): Promise<boolean>;

  export function findAllFiles(
    root: string,
    options?: { extensions?: string[] },
  ): Promise<string[]>;

  export function sha256(path: string): Promise<string>;
  export function sha256(path: string[]): Promise<string[]>;

  export function fileSize(path: string): Promise<number>;
  export function fileSize(path: string[]): Promise<number[]>;
}
