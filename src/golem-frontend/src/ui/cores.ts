import * as core from "@:golem/core";
import * as ui from "@:golem/ui";
import { getDb } from "../services/database";

function start_core(db_core: { path: string }) {
  let c = core.load({
    core: { type: "Path", path: db_core.path },
  });
  c.loop();
}

function select_core_file() {
  let f = ui.selectFile("Select Core", "/media/fat", {
    dirFirst: false,
    extensions: ["rbf"],
  });

  if (f !== undefined) {
    let c = core.load({
      core: { type: "Path", path: f },
    });
    c.loop();
  }
}

export async function cores_menu() {
  const userDb = await getDb();
  const cores = await userDb.query("SELECT * FROM cores");

  await ui.textMenu({
    title: "Cores",
    back: true,
    items: [
      ...cores.map((core) => ({
        label: "" + core.name,
        select: () => start_core(core as any),
      })),
      "-",
      { label: "Select File...", select: select_core_file },
      "-",
      {
        label: "Download Cores...",
        select: () => ui.alert("Not implemented yet!"),
      },
    ],
  });
}
