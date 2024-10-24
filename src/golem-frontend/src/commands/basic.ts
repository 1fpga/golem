import { coreOsdMenu } from "$/ui/menus/core_osd";
import { Commands, Core } from "$/services";

export async function init() {
  await Commands.registerCommand({
    type: "core",
    key: "showMenu",
    name: "Show the core menu",
    category: "Core",
    running: false,
    async handler(core) {
      if (core && !this.running) {
        this.running = true;
        const coreDb = Core.running();
        core.showOsd(() => coreOsdMenu(core, coreDb));
        this.running = false;
      }
    },
    default: "'F12'",
  });

  await Commands.registerCommand({
    type: "core",
    key: "quitCore",
    name: "Quit to the main menu",
    category: "Core",
    handler: (c) => {
      c?.quit();
    },
    default: "'F10'",
  });

  await Commands.registerCommand({
    type: "general",
    key: "showDebugLog",
    name: "Show a debug log",
    category: "Debug",
    handler: async () => {
      console.log("Debug log.");
    },
    default: "Ctrl + 'D'",
  });
}
