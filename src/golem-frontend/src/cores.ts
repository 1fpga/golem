import * as core from "golem/core";
import * as db from "golem/db";
import * as ui from "golem/ui";

function start_core(db_core: { path: string }) {
  core.run({
    core: { type: "path", path: db_core.path },
  });
}

function select_core_file() {
  let f = ui.selectFile("Select Core", "/media/fat", {
    dirFirst: false,
    extensions: ["rbf"],
  });

  if (f !== undefined) {
    core.run({
      core: { type: "path", path: f },
    });
  }
}

export function cores_menu() {
  const cores = db.query("SELECT * FROM cores");
  ui.textMenu({
    title: "Cores",
    back: () => true,
    items: [
      ...cores.map((core) => ({
        label: "" + core.name,
        select: () => start_core(core as any),
      })),
      "-",
      { label: "Select File...", select: select_core_file },
    ],
  });
}
