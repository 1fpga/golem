import * as db from "golem/db";
import * as ui from "golem/ui";

import {games_menu} from "./games.mjs";

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
            {label: "Cores...", id: cores_menu, marker: cores_lbl},
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

while (true) {
    try {
        main_menu();
    } catch (e) {
        console.error(e);
        ui.alert("Error", e.message);
    }
}
