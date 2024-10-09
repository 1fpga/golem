// Type definitions for general types and interfaces.

/**
 * Represents an image that can be saved or loaded.
 */
export declare class Image {
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

export declare class JsonSchema<T> {
  /**
   * Create a new JSON schema.
   * @param schema The schema to use.
   */
  constructor(schema: object);

  /**
   * Validate the given JSON object.
   * @param json The JSON object to validate.
   * @returns Whether the JSON object is valid.
   */
  validate(json: unknown): json is T;

  /**
   * Get the errors from the last validation.
   * @returns The errors from the last validation.
   */
  get errors(): string[] | null;
}
