import * as core from "@:golem/core";
import * as ui from "@:golem/ui";
import { Core } from "$/services/database/core";

async function selectCoreFile() {
  let f = await ui.selectFile("Select Core", "/media/fat", {
    dirFirst: false,
    extensions: ["rbf"],
  });

  if (f !== undefined) {
    Core.setRunning(null);
    let c = core.load({
      core: { type: "Path", path: f },
    });
    c.showOsd(async () =>
      (await import("$/ui/menus/core_osd")).coreOsdMenu(c, null),
    );
    c.loop();
  }
}

export async function coresMenu() {
  const cores = await Core.list();

  await ui.textMenu({
    title: "Cores",
    back: 0,
    items: [
      ...cores.map((core) => ({
        label: "" + core.name,
        select: async () => {
          await core.launch();
        },
      })),
      "-",
      { label: "Select File...", select: selectCoreFile },
    ],
  });
}
