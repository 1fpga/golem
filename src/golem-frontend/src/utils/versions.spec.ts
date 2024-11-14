import { expect, test } from "@jest/globals";
import { compareVersions } from "./versions";

const cases = [
  ["a", "b", -1],
  ["a", "a", 0],
  [0, 1, -1],
  [1, 0, 1],
  [123, 123, 0],
  ["0.2", "0.12", -1],
  ["0.2.1", "0.2.2", -1],
  ["0.2.3", "0.2.2", 1],
  ["0.2.3-beta", "0.2.2", 1],
];

test.each(cases)("compareVersions(%p, %p) == %p", (a, b, expected) => {
  const result = compareVersions(a, b);
  const actual = result < 0 ? -1 : result > 0 ? 1 : 0;
  expect(actual).toBe(expected);
});
