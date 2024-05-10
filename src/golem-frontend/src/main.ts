// The root file being executed by Golem by default.
import * as db from "golem/db";
import * as ui from "golem/ui";

import { games_menu } from "./games";
import { cores_menu } from "./cores";
import { about } from "./about";

function settings_menu() {
  ui.alert("Settings", "Not implemented yet.");
}

function downloads_menu() {
  ui.alert("Downloads", "Not implemented yet.");
}

async function main_menu() {
  const nb_games = db.queryOne("SELECT COUNT(*) as count FROM games")
    ?.count as number;
  const nb_cores = db.queryOne("SELECT COUNT(*) as count FROM cores")
    ?.count as number;

  const games_lbl = nb_games > 0 ? `(${nb_games})` : "";
  const cores_lbl = nb_cores > 0 ? `(${nb_cores})` : "";

  ui.textMenu({
    title: "",
    items: [
      { label: "Games...", select: games_menu, marker: games_lbl },
      { label: "Cores...", select: cores_menu, marker: cores_lbl },
      "---",
      { label: "Settings...", select: settings_menu },
      { label: "Downloads...", select: downloads_menu },
      "---",
      { label: "About", select: about },
      "---",
      { label: "Exit", select: () => true },
    ],
  });
}

export async function main() {
  while (true) {
    try {
      return await main_menu();
    } catch (e: any) {
      console.error(e);
      ui.alert("Error", e.message);
    }
  }
}
