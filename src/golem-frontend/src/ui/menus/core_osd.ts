import * as ui from "@:golem/ui";
import * as core from "@:golem/core";
import { CoreSettingPage } from "@:golem/core";
import type { Core } from "$/services/database/core";

enum SettingReturn {
  Continue,
  ReturnContinue,
  Back,
  Quit,
}

export async function coreSettingsMenu(
  core: core.GolemCore,
  pageLabel?: string,
): Promise<SettingReturn> {
  let shouldReturn = SettingReturn.Continue;
  while (shouldReturn === SettingReturn.Continue) {
    let menu: core.CoreSettingPage | core.CoreSettings = core.settings;

    if (pageLabel !== undefined) {
      menu =
        menu.items.find(
          (item): item is CoreSettingPage =>
            item.kind === "page" && item.label === pageLabel,
        ) ?? menu;
    }

    shouldReturn = await ui.textMenu<SettingReturn>({
      title: "Core Settings",
      back: SettingReturn.ReturnContinue,
      items: [
        ...(await Promise.all(
          menu.items.map((item) => {
            switch (item.kind) {
              case "page":
                return {
                  label: item.label,
                  marker: ">",
                  select: async () => {
                    return await coreSettingsMenu(core, item.label);
                  },
                };
              case "separator":
                return "-";
              case "label":
                return {
                  label: item.label,
                  selectable: item.selectable,
                };
              case "file":
                return {
                  label: item.label,
                  marker: item.extensions.join(","),
                  select: async () => {
                    let path = await ui.selectFile(item.label, "/media/fat", {
                      extensions: item.extensions,
                    });
                    if (path) {
                      core.fileSelect(item.id, path);
                      return false;
                    }
                  },
                };
              case "trigger":
                return {
                  label: item.label,
                  marker: "!",
                  select: () => {
                    core.trigger(item.id);
                  },
                };
              case "bool":
                return {
                  label: item.label,
                  marker: item.value ? "[X]" : "[ ]",
                  select: (menuItem: ui.TextMenuItem<SettingReturn>) => {
                    item.value = core.boolSelect(item.id, !item.value);
                    menuItem.marker = item.value ? "[X]" : "[ ]";
                  },
                };
              case "int":
                return {
                  label: item.label,
                  marker: item.choices[item.value],
                  select: (menuItem: ui.TextMenuItem<SettingReturn>) => {
                    item.value = core.intSelect(
                      item.id,
                      (item.value + 1) % item.choices.length,
                    );
                    menuItem.marker = item.choices[item.value];
                  },
                };
            }
          }),
        )),
      ],
    });

    if (shouldReturn === undefined) {
      shouldReturn = SettingReturn.Continue;
    }
  }

  return shouldReturn === SettingReturn.ReturnContinue
    ? SettingReturn.Continue
    : shouldReturn;
}

function isKindFile(
  item: core.CoreSettingsItem,
): item is core.CoreSettingFileSelect {
  return item.kind === "file";
}

export async function coreOsdMenu(
  golemCore: core.GolemCore,
  coreDb: Core | null,
): Promise<core.OsdResult> {
  let menu = golemCore.settings;

  let fileMenus = menu.items.filter(isKindFile);

  console.log(JSON.stringify(menu));

  return await ui.textMenu({
    title: "Core Menu",
    back: false,
    items: [
      ...(coreDb
        ? [
            {
              label: "Load Game...",
              select: async () => {
                // let system = await coreDb.getSystem();
                // let game = await (
                //   await import("$/services/database/games")
                // ).Games.select({
                //   title: "Load Game",
                //   system: system.uniqueName,
                // });
                // if (game?.romPath) {
                //   golemCore.fileSelect(0, game.romPath);
                //   return false;
                // }
              },
            },
          ]
        : []),
      ...(await Promise.all(
        fileMenus.map((item) => ({
          label: item.label,
          select: async () => {
            let path = await ui.selectFile(item.label, "/media/fat", {
              extensions: item.extensions,
            });
            if (path) {
              golemCore.fileSelect(item.id, path);
              return false;
            }
          },
        })),
      )),
      "-",
      {
        label: "Core Settings...",
        select: async () => {
          switch (await coreSettingsMenu(golemCore)) {
            case SettingReturn.Back:
              return undefined;
            case SettingReturn.Quit:
              return true;
            default:
              return undefined;
          }
        },
      },
      {
        label: "Reset Core",
        select: () => {
          golemCore.reset();
          return false;
        },
      },
      "-",
      {
        label: "Quit",
        select: () => true,
      },
    ],
  });
}