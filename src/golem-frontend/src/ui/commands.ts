import * as commands from "@:golem/commands";
import BASIC_COMMANDS from "../commands/basic";
import { DbStorage } from "../services";

export async function initCommands() {
  const storage = await DbStorage.global();
  let validate = (await import("$schemas:shortcut")).validate;

  for (const cmd of BASIC_COMMANDS) {
    let settingsName = `shortcuts-${cmd.shortName}`;

    let command = await addCommand_(cmd);
    const maybeShortcuts = await storage.get(settingsName);
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
