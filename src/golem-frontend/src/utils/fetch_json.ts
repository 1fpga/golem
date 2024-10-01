import type { ValidateFunction } from "ajv";
import net from "@:golem/net";

/**
 * Fetch a JSON file from the internet and validate it.
 * @param url The URL to fetch.
 * @param validate The validation function to use.
 * @returns The parsed JSON.
 */
export async function fetchJsonAndValidate<T>(
  url: string,
  validate: ValidateFunction<T>,
): Promise<T> {
  const response = await net.fetchJson(url);
  if (validate(response)) {
    return response;
  } else {
    throw new Error(
      `Validation error on URL ${url}:\n` +
        (validate.errors || []).map((e) => e.message || "").join("\n"),
    );
  }
}
