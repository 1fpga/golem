// Type definitions for general types and interfaces.

declare interface SendToBackgroundOptions {
  /**
   * The position of the image on the screen. Position is only used
   * when the stretch mode is set to None.
   * @default "center"
   */
  position?: { x: number; y: number } | "center" | "top-left";

  /**
   * Whether to clear the background or not. Default to false.
   */
  clear?: boolean;
}

declare global {
  /**
   * Represents an image that can be saved or loaded.
   */
  class Image {
    readonly width: number;
    readonly height: number;

    /**
     * Load an image from the given path.
     * @param path The path to the image file.
     * @returns The loaded image.
     */
    static load(path: string): Promise<Image>;

    /**
     * Load an embedded (included in the release) image.
     * @param name The name of the image.
     * @returns The loaded image.
     */
    static embedded(name: string): Promise<Image>;

    /**
     * The default background image.
     */
    static embedded(name: "background"): Promise<Image>;

    /**
     * Save the image to the given path.
     * @param path The path to save the image to.
     */
    save(path: string): Promise<void>;

    /**
     * Send the image to the background (only in the menu core).
     */
    sendToBackground(options?: SendToBackgroundOptions): void;

    /**
     * Resize the image, creating a new image.
     * @param width The new width.
     * @param height The new height.
     * @param keepAspectRatio Whether to keep the aspect ratio or not. Defaults to true.
     */
    resize(width: number, height: number, keepAspectRatio?: boolean): Image;
  }

  class JsonSchema<T> {
    /**
     * Validate the given JSON object, or throw an error if it is invalid.
     * @param schema The schema to use.
     * @param json The JSON object to validate.
     * @returns Whether the JSON object is valid.
     */
    static validateOrThrow<T>(schema: object, json: unknown): json is T;
  }

  interface OneFpgaGlobal {
    name: "OneFPGA";
    version: {
      major: number;
      minor: number;
      patch: number;
    };
  }

  const ONE_FPGA: OneFpgaGlobal;
}

export {};
