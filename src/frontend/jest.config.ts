import type { JestConfigWithTsJest } from "ts-jest";

const jestConfig: JestConfigWithTsJest = {
  testEnvironment: "node",
  transform: {
    "^.+.tsx?$": ["ts-jest", {}],
  },
  rootDir: ".",
  moduleNameMapper: {
    "^1fpga:(.*)$": "<rootDir>/types/1fpga/$1",
  },
};

export default jestConfig;
