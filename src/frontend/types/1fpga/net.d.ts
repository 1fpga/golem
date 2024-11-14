// Network access type definitions.

/**
 * This module provides functions for accessing the internet. It can download
 * JSON data or files from URLs.
 */
declare module "1fpga:net" {
  export interface NetworkInterface {
    status: "Loopback" | "Up" | "Down";
    flags: string[];
    name: string;
    address?: string;
    netmask?: string;
    family?: "IPv4" | "IPv6";
  }

  /**
   * Returns true if an Internet connection is available.
   */
  export function isOnline(): Promise<boolean>;

  /**
   * Returns a list of network interfaces and their details.
   */
  export function interfaces(): Promise<NetworkInterface[]>;

  /**
   * Download a JSON file from a URL. Returns the parsed JSON.
   * Will throw an error if the request fails or the response is not JSON.
   *
   * @param url The URL to download.
   */
  export function fetchJson<R extends object = any>(url: string): Promise<R>;

  /**
   * Download a file from a URL. Returns the file location.
   * Will throw an error if the request fails.
   *
   * @param url The URL to download.
   * @param destination The destination directory. If not provided, a temporary file will be created.
   * @returns The path to the downloaded file.
   */
  export function download(url: string, destination?: string): Promise<string>;
}
