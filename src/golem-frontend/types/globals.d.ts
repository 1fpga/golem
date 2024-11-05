// Type definitions for general types and interfaces.

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
     * Save the image to the given path.
     * @param path The path to save the image to.
     */
    save(path: string): Promise<void>;
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
    };
  }

  const ONE_FPGA: OneFpgaGlobal;
}

export {};
