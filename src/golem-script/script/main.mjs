import * as core from "golem/core";
import * as db from "golem/db";
import * as ui from "golem/ui";

function start_game(game_id) {
    console.log(game_id);
    const db_game = db.get("SELECT * FROM games WHERE id = ?", [game_id]);
    const db_core = db.get("SELECT * from cores WHERE id = ?", [db_game.core_id]);

    const g = db_game;
    const c = db_core;

    if (!g || !c) {
        ui.alert("Game not found", `Could not find the game with id ${game_id} or a core for it.`);
        return;
    }

    core.run({
        core: c.path,
        game: g.path,
    })
}

function test_menu() {
    const [action, id] = ui.menu({
        title: "Test",
        back: true,
        items: [
            {label: "Test 1", id: "test1"},
            {label: "Test 2", id: "test2"},
        ],
    });

    switch (action) {
        case "select":
            switch (id) {
                case "test1":
                    ui.alert("Test 1", "Test 2");
                    break;
                case "test2":
                    ui.alert("Test 2");
                    break;
            }
            break;
        case "back":
            break;
    }
}

function main_menu() {
    const nb_games = db.query("SELECT COUNT(*) as count FROM games")?.[0]?.count;
    const nb_cores = db.query("SELECT COUNT(*) as count FROM cores")?.[0]?.count;

    const games_lbl = nb_games > 0 ? `(${nb_games})` : "";
    const cores_lbl = nb_cores > 0 ? `(${nb_cores})` : "";

    const [action, id] = ui.menu({
        title: "",
        back: false,
        items: [
            {label: "Games...", id: games_menu, marker: games_lbl},
            {label: "Cores...", id: "cores", marker: cores_lbl},
            "---",
            {label: "Settings...", id: "settings"},
            {label: "Downloads...", id: "downloads"},
            "---",
            {label: "About", id: "about"},
            {label: "Test", id: test_menu},
            "---",
            {label: "Exit", id: "exit"},
        ],
    });

    switch (action) {
        case "select":
            switch (id) {
                case "exit":
                    return false;
                default:
                    id();
            }
            break;
        case "back":
            throw new Error("Should never back here");
    }

    return true;
}

function games_menu() {
    const games = db.query("SELECT games.id as id, games.name as name, cores.system_slug as system FROM games LEFT JOIN cores ON games.core_id = cores.id");
    const [action, id] = ui.menu({
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

while (main_menu()) {
}
