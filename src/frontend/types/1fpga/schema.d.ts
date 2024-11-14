// Schema functions.

declare module "1fpga:schema" {
  /**
   * Add a schema to the schema registry.
   * @param schema The schema to add.
   * @returns The schema ID.
   * @throws If the schema is invalid, an error is thrown.
   */
  export function addSchema(schema: object): string;

  /**
   * Validate a value with a schema.
   * @param value The value to validate.
   * @param schema The schema to validate the value against. It will be added to the database.
   * @returns Whether the value is valid.
   * @throws If the value is invalid, an error is thrown with all validation errors.
   */
  export function validate<T>(value: unknown, schema: object): value is T;

  /**
   * Validate a value with a schema.
   * @param value The value to validate.
   * @param schemaId The schema ID to validate the value against.
   * @returns Whether the value is valid.
   * @throws If the value is invalid, an error is thrown with all validation errors.
   * @throws If the schema ID is not found, an error is thrown.
   */
  export function validateWithId<T>(
    value: unknown,
    schemaId: string,
  ): value is T;
}
