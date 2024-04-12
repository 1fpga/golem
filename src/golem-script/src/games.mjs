import * as core from "golem/core";
import * as db from "golem/db";
import * as ui from "golem/ui";

function start_game(game_id) {
    const db_game = db.queryOne("SELECT * FROM games WHERE id = ?", [game_id]);
    const db_core = db.queryOne("SELECT * from cores WHERE id = ?", [db_game.core_id]);

    const g = db_game;
    const c = db_core;

    if (!g || !c) {
        ui.alert(
            "Game not found",
            `Could not find the game with id ${game_id} or a core for it.`,
        );
        return;
    }

    db.execute("UPDATE games SET last_played = ? WHERE id = ?", [new Date().toISOString(), g.id]);

    core.run({
        core: {type: "path", path: c.path},
        game: {type: "rom-path", path: g.path},
        autoloop: true,
        showmenu: false,
    })
}

export function games_menu() {
    const sortOptions = {
        "Name (A-Z)": "name ASC",
        "Name (Z-A)": "name DESC",
        "System (A-Z)": "cores.system_slug ASC",
        "Last Played": "games.last_played DESC",
        "Favorites": "games.favorite DESC, name ASC",
    };
    let current_sort = 0;

    function buildItems() {
        let games = db.query(
            `SELECT games.id as id, games.name as name, cores.system_slug as system
             FROM games
                      LEFT JOIN cores ON games.core_id = cores.id
             ORDER BY ${Object.values(sortOptions)[current_sort]}`
        );
        return games.map(game => ({
            label: game.name,
            select: () => start_game(game.id),
            marker: game.system,
        }));
    }

    ui.textMenu({
        title: "Games",
        back: () => true,
        sort_label: Object.keys(sortOptions)[current_sort],
        sort: () => {
            current_sort = (current_sort + 1) % Object.keys(sortOptions).length;

            return {
                sort_label: Object.keys(sortOptions)[current_sort],
                items: buildItems(),
            };
        },
        items: buildItems(),
    });
}
