// Type definitions for the `1fpga:fs` module.

/**
 * File system functions.
 */
declare module "1fpga:fs" {
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

  /**
   * Get the SHA-256 hash of a file.
   * @param path The path to the file.
   */
  export function sha256(path: string): Promise<string>;

  /**
   * Get the SHA-256 hash of multiple files.
   * @param path The paths to the files.
   */
  export function sha256(path: string[]): Promise<string[]>;

  export function fileSize(path: string): Promise<number>;
  export function fileSize(path: string[]): Promise<number[]>;
}
