import * as core from "@:golem/core";
import * as db from "@:golem/db";
import * as ui from "@:golem/ui";
import * as retronomicon from "./retronomicon";

async function download_cores() {
  ui.show("Downloading Cores, please wait...");
  let cores = retronomicon.cores();

  await ui.textMenu({
    title: "Download Cores",
    back: true,
    items: cores.map((core) => ({
      label: "" + core.name,
      select: () => {
        let releases = retronomicon.releases(core.id);
        console.log(1);
        console.log(JSON.stringify(core));
        console.log(JSON.stringify(releases));
        console.log(3);
        return true;
        //
        // db.execute(
        //   "INSERT OR REPLACE INTO cores (id, name, system_slug, path) VALUES (?, ?, ?, ?)",
        //   [core.id, core.name, core.system.slug, core.path],
        // );
      },
    })),
  });
}

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
  const cores = db.query("SELECT * FROM cores");
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
      { label: "Download Cores...", select: download_cores },
    ],
  });
}
