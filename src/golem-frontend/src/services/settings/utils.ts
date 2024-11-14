import { DbStorage } from "$/services";

export async function getOrFail<T>(
  storage: DbStorage,
  key: string,
  d?: T,
  validator?: (v: unknown) => v is T,
): Promise<T> {
  let value;

  try {
    value = await storage.get<T>(key, validator);
  } catch (_) {
    // If anything happens, treat it as a missing value.
  }

  if (value === undefined) {
    if (d !== undefined) {
      return d;
    }
    throw new Error(`Missing value for key: ${key}`);
  }
  return value;
}
