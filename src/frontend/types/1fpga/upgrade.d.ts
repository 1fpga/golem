/**
 * Upgrade module, responsible for updating the firmware and other protected
 * binaries.
 *
 * TypeScript is responsible for deciding which file to use based on
 * any user input (settings, etc), and when to try to perform the
 * upgrade.
 *
 * The upgrade module performs a check with a public key that is
 * shared in the source code. If the signature is valid, the upgrade
 * is performed. If it is invalid, and the TypeScript API has
 * permission to do so, the user will be prompted by the firmware
 * to accept the upgrade.
 *
 * This is to prevent plugins from trying to perform upgrades without
 * user consent (or knowledge).
 */

/**
 * This module provides functions for upgrading the firmware.
 */
declare module "1fpga:upgrade" {
  /**
   * Perform a firmware upgrade.
   *
   * @param name The name of the binary to upgrade. For now this is always "1fpga".
   * @param path The path of the firmware file to upgrade to.
   * @param signature The signature of the firmware file. If not provided,
   *                  the firmware will require the user to validate the upgrade
   *                  manually before proceeding.
   * @throws If the upgrade fails.
   */
  export function upgrade(
    name: string,
    path: string,
    signature?: Uint8Array,
  ): Promise<void>;

  /**
   * Verify a firmware file. This is a convenience to check if the firmware
   * file is valid before attempting to upgrade.
   *
   * @param path The path of the firmware file to verify.
   * @param signature The signature of the firmware file. This must be provided.
   * @throws string If the file path is wrong or the signature is the invalid
   *                format. This will not throw if the signature is invalid.
   */
  export function verifySignature(
    path: string,
    signature: Uint8Array,
  ): Promise<boolean>;
}
