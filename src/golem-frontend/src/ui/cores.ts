import * as core from "@:golem/core";
import * as ui from "@:golem/ui";
import { Core } from "../services/database/core";

function start_core(path: string) {
  console.log(`Starting core: ${JSON.stringify(path)}`);
  let c = core.load({
    core: { type: "Path", path },
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
  const cores = await Core.list();

  await ui.textMenu({
    title: "Cores",
    back: true,
    items: [
      ...cores.map((core) => ({
        label: "" + core.name,
        select: async () => {
          if (core.rbfPath) {
            start_core(core.rbfPath);
          } else {
            await ui.alert("Core is not an RBF core?");
          }
        },
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
