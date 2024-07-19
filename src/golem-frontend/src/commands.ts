import * as commands from "@/golem/commands";
import * as storage from "@/golem/storage";

export const BASIC_COMMANDS: commands.CommandDef[] = [
  {
    type: "core",
    shortName: "showCoreMenu",
    name: "Show Core Menu",
    description: "Show the running core's menu.",
    action: async (core) => {
      core.showMenu();
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
    defaultShortcuts: ["'F4'"],
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

export async function initCommands() {
  let validate = (await import("$schemas:shortcut")).default;

  for (const cmd of BASIC_COMMANDS) {
    let settingsName = `shortcuts-${cmd.shortName}`;

    let command = await addCommand_(cmd);
    const maybeShortcuts = storage.get(settingsName);
    if (validate(maybeShortcuts)) {
      command.shortcuts = maybeShortcuts;
    } else if (cmd.defaultShortcuts !== undefined) {
      command.shortcuts = cmd.defaultShortcuts;
    }
  }
}

async function addCommand_(
  cmd: commands.CommandDef,
): Promise<commands.Command> {
  // Add the command to the list of commands.
  if (cmd.type === "general") {
    return commands.createGeneralCommand(
      cmd.shortName,
      cmd.name,
      cmd.description,
      cmd.action,
    );
  } else if (cmd.type === "core") {
    return commands.createCoreCommand(
      cmd.shortName,
      cmd.name,
      cmd.description,
      cmd.action,
    );
  } else {
    return commands.createCoreSpecificCommand(
      cmd.shortName,
      cmd.name,
      cmd.description,
      cmd.core,
      cmd.action,
    );
  }
}
