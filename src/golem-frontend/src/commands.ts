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
  },
  {
    type: "core",
    shortName: "quitCore",
    name: "Quit Core",
    description: "Quit the running core and go back to the main menu.",
    action: async (core) => {
      core.quit();
    },
  },
  {
    type: "core",
    shortName: "screenshot",
    name: "Take Screenshot",
    description: "Take a screenshot of the current screen.",
    action: async (core) => {
      core.screenshot("screenshot.png");
    },
  },
];

export async function initCommands() {
  for (const cmd of BASIC_COMMANDS) {
    let settingsName = `shortcuts-${cmd.shortName}`;
    let shortcuts = storage.get(settingsName);

    let command = await addCommand_(cmd);
    command.shortcuts = Array.isArray(shortcuts) ? shortcuts : [];
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
