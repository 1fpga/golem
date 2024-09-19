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
