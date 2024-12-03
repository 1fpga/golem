import type { ErrorObject, ValidateFunction } from "ajv";
import * as net from "1fpga:net";
import * as osd from "1fpga:osd";

export class ValidationError extends Error {
  constructor(public readonly errors: ErrorObject[]) {
    const message =
      `Validation error:\n  ` +
      errors.map((e) => JSON.stringify(e)).join("\n  ");
    super(message);
  }
}

/**
 * Fetch a JSON file from the internet and validate it.
 * @param url The URL to fetch.
 * @param validate The validation function to use.
 * @param options Options for fetching the url.
 * @returns The parsed JSON.
 */
export async function fetchJsonAndValidate<T>(
  url: string,
  validate: ValidateFunction<T> | ((json: unknown) => json is T),
  options?: {
    allowRetry?: boolean;
    onPreValidate?: (json: any) => Promise<void>;
  },
): Promise<T> {
  while (true) {
    try {
      const response = await net.fetchJson(url);
      if (options?.onPreValidate) {
        await options.onPreValidate(response);
      }

      if (validate(response)) {
        return response;
      } else {
        const e = (validate as any).errors ?? [];
        console.warn(`Validation error: ${JSON.stringify(e)}`);
        throw new ValidationError(e);
      }
    } catch (e) {
      if (!(options?.allowRetry ?? true)) {
        console.warn(`Error fetching JSON: ${e}`);
        throw e;
      }

      let message = (e as any)?.message ?? `${e}`;
      if (message.toString() == "[object Object]" || !message) {
        message = JSON.stringify(e);
      }

      const choice = await osd.alert({
        title: "Error fetching JSON",
        message: `URL: ${url}\n\n${(e as any)?.message ?? JSON.stringify(e)}\n`,
        choices: ["Retry fetching", "Cancel"],
      });

      if (choice === 1) {
        throw e;
      }
    }
  }
}
