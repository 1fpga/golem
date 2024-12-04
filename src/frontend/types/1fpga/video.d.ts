// Video management.

declare module "1fpga:video" {
  /**
   * Set the video mode.
   * @param mode A string representing the video mode to set.
   */
  export function setMode(mode: string): void;

  /**
   * Get the current video resolution. When not in the menu core, this
   * will return undefined.
   */
  export function getResolution():
    | { width: number; height: number }
    | undefined;
}
