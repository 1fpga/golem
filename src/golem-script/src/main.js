// The root file being executed by Golem by default.
import * as db from "golem/db";
import * as ui from "golem/ui";

import {games_menu} from "./games.mjs";
import {cores_menu} from "./cores.mjs";
import {about} from "./about.mjs"
import {video_menu} from "./settings/video.mjs";

function main_menu() {
    const nb_games = db.queryOne("SELECT COUNT(*) as count FROM games")?.count;
    const nb_cores = db.queryOne("SELECT COUNT(*) as count FROM cores")?.count;

    const games_lbl = nb_games > 0 ? `(${nb_games})` : "";
    const cores_lbl = nb_cores > 0 ? `(${nb_cores})` : "";

    const [action, id] = ui.textMenu({
        title: "",
        back: false,
        items: [
            {label: "Games...", id: games_menu, marker: games_lbl},
            {label: "Cores...", id: cores_menu, marker: cores_lbl},
            "---",
            {label: "Settings...", id: video_menu},
            {label: "Downloads...", id: "downloads"},
            "---",
            {label: "About", id: about},
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


