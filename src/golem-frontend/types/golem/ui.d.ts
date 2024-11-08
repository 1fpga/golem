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
   * Represents a textual menu item.
   */
  export interface TextMenuItem<R> {
    label: string;
    marker?: string;
    select?: (
      item: TextMenuItem<R>,
      index: number,
    ) => undefined | void | R | Promise<undefined | void | R>;
    details?: (
      item: TextMenuItem<R>,
      index: number,
    ) => undefined | void | R | Promise<undefined | void | R>;
  }

  /**
   * Represents the options for the `textMenu` function.
   */
  export interface TextMenuOptions<R> {
    /**
     * The title to show at the top of the menu.
     */
    title?: String;

    /**
     * All items.
     */
    items: (string | TextMenuItem<R>)[];

    /**
     * The value to return if the user presses the back button (or function to execute).
     */
    back?: R | (() => undefined | void | R | Promise<undefined | void | R>);

    /**
     * The value to return if the user presses the cancel button (or function to execute).
     */
    sort?: () =>
      | Partial<TextMenuOptions<R>>
      | void
      | Promise<Partial<TextMenuOptions<R>> | void>;

    /**
     * The label to show for the sort button.
     */
    sort_label?: string;

    /**
     * The label to show for the detail button. If missing, it will not be shown.
     */
    details?: string;

    /**
     * The index of the item to highlight when presenting the menu to the
     * user. By default, the first item is highlighted. If a number is
     * provided but the index is out of bounds, the last item is highlighted.
     * If an unselectable item is highlighted, the next selectable item will
     * be highlighted instead.
     */
    highlighted?: number;

    /**
     * The value of an item to select. This will execute the `select` function
     * of the item with the given value. If multiple items have the same label,
     * the first one will be selected. Provide a number for an index instead.
     */
    selected?: string | number;
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
  export function alert(message: string): Promise<void>;
  export function alert(title: string, message: string): Promise<void>;
  export function alert(options: {
    title?: string;
    message: string;
    choices?: string[];
  }): Promise<null | number>;

  /**
   * Show a prompt to the user, which the user can input any text, and an OK/Cancel
   * choices.
   * @returns The user input, or `undefined` if the user canceled the operation.
   */
  export function prompt(message: string): Promise<undefined | string>;
  export function prompt(
    title: string,
    message: string,
  ): Promise<undefined | string>;
  export function prompt(options: {
    title?: string;
    message: string;
    default?: string;
  }): Promise<undefined | string>;

  /**
   * Show a prompt to the user for a button/key password.
   * @param title The title of the prompt.
   * @param message The message (title) to show to the user.
   * @param length The length of the password.
   * @returns The user's password (as a string), or `null` if the user canceled the operation.
   */
  export function promptPassword(
    title: string,
    message: string,
    length: number,
  ): Promise<string[] | null>;

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
    directory?: boolean;
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
  ): Promise<string | undefined>;

  /**
   * Show the input tester panel.
   */
  export function inputTester(): Promise<void>;

  /**
   * Prompt the user for a shortcut.
   */
  export function promptShortcut(
    title?: string,
    message?: string,
  ): Promise<string | undefined>;
}
