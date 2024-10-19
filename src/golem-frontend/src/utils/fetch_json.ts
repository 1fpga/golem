import type {ValidateFunction} from "ajv";
import * as net from "@:golem/net";
import * as ui from "@:golem/ui";

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
        throw new Error(
          `Validation error: ` +
            ((validate as any).errors ?? [])
              .map((e: any) => e.message || "")
              .join("\n"),
        );
      }
    } catch (e) {
      if (!(options?.allowRetry ?? true)) {
        throw e;
      }

      const choice = await ui.alert({
        title: "Error fetching JSON",
        message: `URL: ${url}\n${JSON.stringify(e)}`,
        choices: ["Retry fetching", "Cancel"],
      });

      if (choice === 1) {
        throw e;
      }
    }
  }
}
