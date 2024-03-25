import * as ui from "golem/ui";
import * as db from "golem/db";

function main_menu() {
    const nb_games = db.query("SELECT COUNT(*) as count FROM games")?.[0]?.count;
    const nb_cores = db.query("SELECT COUNT(*) as count FROM cores")?.[0]?.count;

    const games_lbl = nb_games > 0 ? `(${nb_games})` : "";
    const cores_lbl = nb_cores > 0 ? `(${nb_cores})` : "";

    const [action, id] = ui.menu({
        title: "",
        back: false,
        items: [
            {label: "Games...", id: "games", marker: games_lbl},
            {label: "Cores...", id: "cores", marker: cores_lbl},
            "---",
            {label: "Settings...", id: "settings"},
            {label: "Downloads...", id: "downloads"},
            "---",
            {label: "About", id: "about"},
            "---",
            {label: "Exit", id: "exit"},
        ],
    });

    switch (action) {
        case "select":
            switch (id) {
                case "exit":
                    return false;
                case "games":
                    games_menu();
                    break;
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
    }
}

while (main_menu()) {
}
