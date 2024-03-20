import * as ui from "golem/ui";
import * as db from "golem/db";

function main_menu() {
    const [action, id] = ui.menu({
        title: "",
        back: false,
        items: [
            {label: "Games", id: "games"},
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
    const [action, id] = ui.menu({
        title: "Games",
        back: true,
        items: db.query("SELECT id, name FROM games").map(game => ({label: game[1], id: game[0]})),
    });
    switch (action) {
        case "back":
            return;
    }
}

while (main_menu()) {
}
