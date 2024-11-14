import * as core from "1fpga:core";
import { coreOsdMenu } from "$/ui/menus/core_osd";
import { Commands, CoreCommandImpl } from "$/services/database/commands";
import { Core } from "$/services/database/core";

export class ShowCoreMenuCommand extends CoreCommandImpl {
  key = "showCoreMenu";
  label = "Show the core's menu";
  category = "Core";
  default = "'F12'";

  // This is used to prevent the menu from being shown multiple times.
  shown = false;

  async execute(core: core.OneFpgaCore) {
    if (!this.shown) {
      try {
        this.shown = true;
        const coreDb = Core.running();
        let error = undefined;
        core.showOsd(async () => {
          try {
            return await coreOsdMenu(core, coreDb);
          } catch (e) {
            error = e;
            return true;
          }
        });
        if (error) {
          throw error;
        }
      } finally {
        this.shown = false;
      }
    }
  }
}

export class QuitCoreCommand extends CoreCommandImpl {
  key = "quitCore";
  label = "Quit to the main menu";
  category = "Core";
  default = "'F10'";

  execute(core: core.OneFpgaCore) {
    core.quit();
  }
}

export class ShowDebugLogCommand extends CoreCommandImpl {
  key = "showDebugLog";
  label = "Show a debug log";
  category = "Developer";
  default = "Ctrl + 'D'";

  execute() {
    console.log("Debug log.");
  }
}

export async function init() {
  await Commands.register(ShowCoreMenuCommand);
  await Commands.register(QuitCoreCommand);
  await Commands.register(ShowDebugLogCommand);
}
