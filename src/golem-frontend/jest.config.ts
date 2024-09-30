import type { JestConfigWithTsJest } from "ts-jest";

const jestConfig: JestConfigWithTsJest = {
  testEnvironment: "node",
  transform: {
    "^.+.tsx?$": ["ts-jest", {}],
  },
  rootDir: ".",
  moduleNameMapper: {
    "^@:golem/(.*)$": "<rootDir>/types/golem/$1",
  },
};

export default jestConfig;
