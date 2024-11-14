/**
 * Compare two versions in the catalog JSONs.
 * @param a The first version.
 * @param b The second version.
 * @returns `<= 1` if `a` is smaller than `b`,
 *          `== 0` if `a` is equal to `b`,
 *          `>= 1` if `a` is greater than `b`.
 */
export function compareVersions(
  a: string | number | null | undefined,
  b: string | number | null | undefined,
): number {
  if (a === null || a === undefined) {
    return b === null || b === undefined ? 0 : 1;
  } else if (b === null || b === undefined) {
    return -1;
  } else if (typeof a === "number") {
    if (typeof b === "number") {
      return a - b;
    } else {
      a = a.toString();
    }
  } else {
    b = b.toString();
  }

  const aParts = a.split(".");
  const bParts = b.split(".");
  const length = Math.max(aParts.length, bParts.length);
  const zipped = Array.from({ length }).map((_, i) => [aParts[i], bParts[i]]);

  for (const [aPart, bPart] of zipped) {
    if (aPart === undefined) {
      return bPart === undefined ? 0 : -1;
    }
    if (Number.isFinite(+aPart) && Number.isFinite(+bPart)) {
      const maybeResult = +aPart - +bPart;
      if (maybeResult !== 0) {
        return maybeResult;
      }
    }

    const maybeResult = aPart.localeCompare(bPart);
    if (maybeResult !== 0) {
      return maybeResult;
    }
  }

  return 0;
}
