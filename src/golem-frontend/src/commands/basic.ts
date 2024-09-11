import commands from "@:golem/commands";
import { coreOsdMenu } from "../menus/core_osd";

const BASIC_COMMANDS: commands.CommandDef[] = [
  {
    type: "core",
    shortName: "showCoreMenu",
    name: "Show Core Menu",
    description: "Show the running core's menu.",
    action: async (core) => {
      core.showOsd(() => coreOsdMenu(core));
    },
    defaultShortcuts: ["'F12'"],
  },
  {
    type: "core",
    shortName: "quitCore",
    name: "Quit Core",
    description: "Quit the running core and go back to the main menu.",
    action: async (core) => {
      core.quit();
    },
    defaultShortcuts: ["'F10'"],
  },
  {
    type: "core",
    shortName: "screenshot",
    name: "Take Screenshot",
    description: "Take a screenshot of the current screen.",
    action: async (core) => {
      core.screenshot("screenshot.png");
    },
    defaultShortcuts: ["'SysReq'"],
  },
  {
    type: "general",
    shortName: "debugLog",
    name: "Debug Log",
    description: "Print a line in debug.",
    action: async () => {
      console.log("Debug log.");
    },
    defaultShortcuts: ["'D' + 'F1'"],
  },
];

export default BASIC_COMMANDS;
