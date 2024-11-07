/**
 * Upgrade module, responsible for updating the golem firmware.
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
declare module "@:golem/upgrade" {
  /**
   * Perform a firmware upgrade.
   *
   * @param path The path of the firmware file to upgrade to.
   * @param signature The signature of the firmware file. If not provided,
   *                  the firmware will require the user to validate the upgrade
   *                  manually before proceeding.
   * @throws If the upgrade fails.
   */
  export function upgrade(path: string, signature?: Uint8Array): Promise<void>;
}
