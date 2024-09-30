import * as ui from "@:golem/ui";
import * as settings from "@:golem/settings";
import { Core } from "../services/database/core";

const MAX_COMMAND_LENGTH = 6;

function markerFor(command?: settings.CommandShortcut) {
  if (!command) {
    return "";
  } else if (Array.isArray(command)) {
    return "...";
  } else if (command.length > MAX_COMMAND_LENGTH) {
    return command.toString().substring(0, MAX_COMMAND_LENGTH) + "...";
  } else {
    return command;
  }
}

export async function commands_settings_menu() {
  const cores = await Core.list();

  await ui.textMenu({
    title: "Commands",
    back: true,
    items: [
      {
        label: "Show Menu",
        marker: markerFor(settings.getSettings().commands?.global?.showMenu),
        select: () => {},
      },
      {
        label: "Reset Core",
        marker: markerFor(settings.getSettings().commands?.global?.resetCore),
        select: () => {},
      },
      {
        label: "Quit Core",
        marker: markerFor(settings.getSettings().commands?.global?.quitCore),
        select: () => {},
      },
      "-",
      {
        label: "Core Specific Commands",
      },
      "-",
      ...(await Promise.all(
        cores.map(async (core) => ({
          label: "" + core.name + "...",
          marker: "" + (await core.getSystem()).uniqueName,
          select: () => {},
        })),
      )),
    ],
  });
}
