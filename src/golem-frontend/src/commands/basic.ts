import * as core from "@:golem/core";
import { coreOsdMenu } from "$/ui/menus/core_osd";
import { Commands, Core, CoreCommandImpl } from "$/services";

export class ShowCoreMenuCommand extends CoreCommandImpl {
  key = "showCoreMenu";
  label = "Show the core's menu";
  category = "Core";
  default = "'F12'";

  // This is used to prevent the menu from being shown multiple times.
  shown = false;

  async execute(core: core.GolemCore) {
    if (!this.shown && Core.running() !== null) {
      this.shown = true;
      const coreDb = Core.running();
      core.showOsd(() => coreOsdMenu(core, coreDb));
      this.shown = false;
    }
  }
}

export class QuitCoreCommand extends CoreCommandImpl {
  key = "quitCore";
  label = "Quit to the main menu";
  category = "Core";
  default = "'F10'";

  async execute(core: core.GolemCore) {
    core.quit();
  }
}

export class ShowDebugLogCommand extends CoreCommandImpl {
  key = "showDebugLog";
  label = "Show a debug log";
  category = "Debug";
  default = "Ctrl + 'D'";

  async execute() {
    console.log("Debug log.");
  }
}

export async function init() {
  await Commands.register(ShowCoreMenuCommand);
  await Commands.register(QuitCoreCommand);
  await Commands.register(ShowDebugLogCommand);
}
