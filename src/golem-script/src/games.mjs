import * as core from "golem/core";
import * as db from "golem/db";
import * as ui from "golem/ui";

function start_game(game_id) {
    const db_game = db.queryOne("SELECT * FROM games WHERE id = ?", [game_id]);
    const db_core = db.queryOne("SELECT * from cores WHERE id = ?", [db_game.core_id]);

    const g = db_game;
    const c = db_core;

    if (!g || !c) {
        ui.alert("Game not found", `Could not find the game with id ${game_id} or a core for it.`);
        return;
    }

    core.run({
        core: {type: "path", path: c.path},
        game: {type: "rom-path", path: g.path},
        autoloop: true,
        showmenu: false,
    })
}

export function games_menu() {
    const games = db.query("SELECT games.id as id, games.name as name, cores.system_slug as system FROM games LEFT JOIN cores ON games.core_id = cores.id");
    const [action, id] = ui.textMenu({
        title: "Games",
        back: true,
        items: games.map(game => ({label: game.name, id: game.id, marker: game.system})),
    });
    switch (action) {
        case "back":
            return;
        default:
            start_game(id);
    }
}
