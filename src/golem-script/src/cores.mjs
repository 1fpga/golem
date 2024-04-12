import * as core from "golem/core";
import * as db from "golem/db";
import * as ui from "golem/ui";

function start_core(db_core) {
    core.run({
        core: {type: "path", path: db_core.path},
    });
}

export function cores_menu() {
    const cores = db.query("SELECT * FROM cores");
    const [action, id] = ui.textMenu({
        title: "Cores",
        back: true,
        items: [
            ...cores.map(core => ({label: core.name, id: core})),
            "-",
            {label: "Select File...", id: "select"},
        ],
    });

    switch (action) {
        case "select":
            if (id === "select") {
                let f = ui.selectFile("Select Core",
                    "/media/fat",
                    {
                        dirFirst: false,
                        extensions: ["rbf"],
                    });

                core.run({
                    core: {type: "path", path: f},
                });
            } else {
                start_core(id);
            }
            return;
        case "back":
            return;
    }
}
