import * as core from "1fpga:core";
import * as ui from "1fpga:ui";
import { Core } from "$/services/database/core";
import { ShowCoreMenuCommand } from "$/commands/basic";
import { Commands } from "$/services";

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

    await Commands.get(ShowCoreMenuCommand)?.execute(c, undefined);
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
