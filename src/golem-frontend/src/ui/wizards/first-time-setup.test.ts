import { firstTimeSetup } from "./first-time-setup";

const ui = jest.createMockFromModule<typeof import("@:golem/ui")>("@:golem/ui");
ui.alert = jest.fn(async () => 0) as any;
console.log(ui.alert("a"));

describe("firstTimeSetup", () => {
  it("works", async () => {
    await firstTimeSetup();
  });
});
