// The root file being executed by Golem by default.
import * as ui from "@:golem/ui";

import { games_menu } from "./ui/games";
import { cores_menu } from "./ui/cores";
import { settings_menu } from "./settings";
import { downloads_menu } from "./downloads";
import { about } from "./ui/about";
import { initCommands } from "./ui/commands";
import { getDb } from "./services/database";
import { login } from "./ui/login";

// Polyfill for events.
globalThis.performance = <any>{
  now: () => Date.now(),
};

async function main_menu() {
  let coreDb = await getDb();
  const nb_games = 0;
  const nb_cores = 0;

  const games_lbl = nb_games > 0 ? `(${nb_games})` : "";
  const cores_lbl = nb_cores > 0 ? `(${nb_cores})` : "";

  await ui.textMenu({
    title: "",
    items: [
      {
        label: "Games...",
        select: games_menu,
        marker: games_lbl,
      },
      { label: "Cores...", select: cores_menu, marker: cores_lbl },
      "---",
      {
        label: "Settings...",
        select: settings_menu,
      },
      { label: "Download Center...", select: downloads_menu },
      "---",
      { label: "About", select: about },
      "---",
      { label: "Exit", select: () => true },
    ],
  });
}

export async function main() {
  // Before setting commands (to avoid commands to interfere with the login menu),
  // we need to initialize the user.
  let user = await login();

  if (user === null) {
    // Run first time setup.
    await (await import("./ui/wizards/first-time-setup")).firstTimeSetup();
  }

  try {
    await initCommands();

    return await main_menu();
  } catch (e: any) {
    console.error(e);
    await ui.alert("Error", e.toString());
  }
}
