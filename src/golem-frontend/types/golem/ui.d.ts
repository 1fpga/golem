// Type definitions for `golem/ui` module in Golem Script.

/**
 * This module provides functions for creating and managing user interfaces in Golem Script.
 * This is a very descriptive API, where the JavaScript setup the UI as a single panel,
 * and then give control to the Golem Script to manage the UI until the User interacts
 * with it. For example, the `textMenu` function will create a menu with a list of
 * textual option, and the function will return the index of the selected option.
 */
declare module "@:golem/ui" {
  /**
   * Represents the state of a text menu.
   */
  export type TextMenuState = {};

  /**
   * Represents a textual menu item.
   */
  export interface TextMenuItem<R> {
    label: string;
    marker?: string;
    select?: (
      item: TextMenuItem<R>,
    ) => undefined | void | R | Promise<undefined | void | R>;
    details?: (
      item: TextMenuItem<R>,
    ) => undefined | void | R | Promise<undefined | void | R>;
  }

  /**
   * Represents the options for the `textMenu` function.
   */
  export interface TextMenuOptions<R> {
    title?: String;
    items: (string | TextMenuItem<R>)[];
    back?: R | (() => undefined | void | R | Promise<undefined | void | R>);
    sort?: () => Partial<TextMenuOptions<R>> | void;
    sort_label?: string;
  }

  /**
   * Creates a textual menu with the given options and show it to the user,
   * waiting for the user to select an option. The function will return a
   * tuple with the action string and the `id` of the selected option.
   * @param options The options for the textual menu.
   * @returns The result of the selected option, as a Promise.
   */
  export function textMenu<R>(options: TextMenuOptions<R>): Promise<R>;

  /**
   * Show an alert to the user, with OK.
   */
  export function alert(message: string): void;
  export function alert(title: string, message: string): void;

  /**
   * Show a prompt to the user, which the user can input any text, and an OK/Cancel
   * choices.
   * @returns The user input, or `undefined` if the user canceled the operation.
   */
  export function prompt(message: string): undefined | string;
  export function prompt(title: string, message: string): undefined | string;
  export function prompt(options: {
    title?: string;
    message: string;
    default?: string;
  }): undefined | string;

  /**
   * Update the UI but don't let the user interact with it.
   */
  export function show(message: string): void;
  export function show(title: string, message: string): void;

  /**
   * Show a message to the user, with a QR Code.
   */
  export function qrCode(url: string, message: string): void;
  export function qrCode(url: string, title: string, message: string): void;

  export interface SelectFileOptions {
    allowBack?: boolean;
    dirFirst?: boolean;
    showHidden?: boolean;
    showExtensions?: boolean;
    showDirectory?: boolean;
    filterPattern?: string;
    extensions?: string[];
  }

  /**
   * Select a file from the user's device. Return the path of the selected file,
   * or `undefined` if the user canceled the operation.
   */
  export function selectFile(
    title: string,
    initialDir: string,
    options: SelectFileOptions,
  ): string | undefined;
}
