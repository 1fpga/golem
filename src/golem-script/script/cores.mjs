import * as db from "golem/db";
import * as ui from "golem/ui";

function start_core(core_id) {
    const db_core = db.get("SELECT * FROM cores WHERE id = ?", [core_id]);
    if (!db_core) {
        ui.alert("Core not found", `Could not find the core with id ${core_id}.`);
        return;
    }

    core.run({
        core: db_core.path,
    });
}

export function cores_menu() {
    const cores = db.query("SELECT * FROM cores");
    const [action, id] = ui.menu({
        title: "Cores",
        back: true,
        items: cores.map(core => ({label: core.name, id: core.id})),
    });

    switch (action) {
        case "select":
            start_core(id);
            return;
        case "back":
            return;
    }
}
