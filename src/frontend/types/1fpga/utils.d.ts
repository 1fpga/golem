// Type definitions for `1fpga:utils` module.

/**
 * Various utility functions.
 */
declare module "1fpga:utils" {
  export interface IpsPatchOptions {
    /**
     * Whether to skip the checksum verification or not.
     * @default false
     */
    skipChecksum?: boolean;
  }

  /**
   * Apply an IPS patch to a ROM. The patch will be applied in-place.
   * @param rom The ROM to patch.
   * @param ips The IPS patch to apply.
   * @param options The options for patching.
   * @throws If the IPS patch is invalid (e.g. the checksum doesn't match).
   */
  export function ipsPatch(
    rom: ArrayBuffer,
    ips: ArrayBuffer,
    options?: IpsPatchOptions,
  ): Promise<void>;
}
