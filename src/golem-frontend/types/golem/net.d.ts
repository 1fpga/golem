// Network access type definitions.

/**
 * This module provides functions for accessing the internet. It can download
 * JSON data or files from URLs.
 */
declare module "@:golem/net" {
  /**
   * Download a JSON file from a URL. Returns the parsed JSON.
   * Will throw an error if the request fails or the response is not JSON.
   *
   * @param url The URL to download.
   */
  export function fetchJson<R extends object = any>(url: string): R;

  /**
   * Download a file from a URL. Returns the path to the downloaded file.
   *
   * @param url The URL to download.
   */
  export function downloadFile(url: string): string;
}
