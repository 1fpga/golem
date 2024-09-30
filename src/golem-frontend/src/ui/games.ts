import * as core from "@:golem/core";
import * as ui from "@:golem/ui";
import { Image } from "@:golem/util";
import { writeFile } from "@:fs";
import { getDb } from "../services/database";

async function start_game(game_id: number) {
  const gameDb = await getDb();
  const db_game = await gameDb.queryOne("SELECT * FROM games WHERE id = ?", [
    game_id,
  ]);
  if (db_game === null) {
    ui.alert("Game not found", `Could not find the game with id ${game_id}.`);
    return;
  }

  const db_core = await gameDb.queryOne("SELECT * from cores WHERE id = ?", [
    db_game.core_id,
  ]);
  if (db_core === null) {
    ui.alert("Game not found", `Could not find the game with id ${game_id}.`);
    return;
  }

  const { rows: db_files } = await gameDb.query(
    "SELECT * from core_files WHERE game_id = ? AND core_id = ?",
    [game_id, db_game.core_id],
  );

  const g = db_game;
  const c = db_core;
  const f = db_files;

  if (!g || !c) {
    ui.alert(
      "Game not found",
      `Could not find the game with id ${game_id} or a core for it.`,
    );
    return;
  }

  await gameDb.execute("UPDATE games SET last_played = ? WHERE id = ?", [
    new Date().toISOString(),
    g.id,
  ]);

  const golem_core = core.load({
    core: { type: "Path", path: "" + c.path },
    game: { type: "RomPath", path: "" + g.path },
    files: f.map((file) => "" + file.path),
  });
  if (golem_core) {
    console.log("Starting core: " + golem_core.name);

    golem_core.loop({
      async onSaveState(savestate: Uint8Array, screenshot: Image) {
        console.log(
          "savestate: ",
          savestate.byteLength,
          screenshot.width,
          screenshot.height,
        );
        try {
          await writeFile("/media/fat/savestate.ss", savestate);
        } catch (e) {
          console.error(e);
        }
      },
    });
  }
}

export async function games_menu() {
  const sortOptions = {
    "Name (A-Z)": "name ASC",
    "Name (Z-A)": "name DESC",
    "System (A-Z)": "cores.system_slug ASC",
    "Last Played": "games.last_played DESC",
    Favorites: "games.favorite DESC, name ASC",
  };
  let current_sort = 0;

  async function buildItems() {
    let gameDb = await getDb();
    // let games = await gameDb.query(
    //   `SELECT games.id as id, games.name as name, cores.system_slug as system
    //          FROM games
    //                   LEFT JOIN cores ON games.core_id = cores.id
    //          ORDER BY ${Object.values(sortOptions)[current_sort]}`,
    // );
    let games: any[] = [];
    return games.map((game) => ({
      label: "" + game.name,
      select: () => start_game(game.id as number),
      marker: "" + game.system,
    }));
  }

  await ui.textMenu({
    title: "Games",
    back: true,
    sort_label: Object.keys(sortOptions)[current_sort],
    sort: async () => {
      current_sort = (current_sort + 1) % Object.keys(sortOptions).length;

      return {
        sort_label: Object.keys(sortOptions)[current_sort],
        items: await buildItems(),
      };
    },
    items: await buildItems(),
  });
}
