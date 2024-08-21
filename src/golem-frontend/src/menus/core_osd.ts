import * as ui from "@/golem/ui";
import * as core from "@/golem/core";

export async function coreSettingsMenu(core: core.GolemCore) {
  let menu = core.settings;
}

function isKindFile(
  item: core.CoreSettingsItem,
): item is core.CoreSettingFileSelect {
  return item.kind === "file";
}

export async function coreOsdMenu(
  core: core.GolemCore,
): Promise<core.OsdResult> {
  let menu = core.settings;

  let fileMenus = menu.items.filter(isKindFile);

  console.log(JSON.stringify(menu));
  return await ui.textMenu({
    title: "Core Menu",
    back: false,
    items: [
      ...fileMenus.map((item) => ({
        label: item.label,
        select: () => {
          let path = ui.selectFile(item.label, "/media/fat", {
            extensions: item.extensions,
          });
          if (path) {
            core.fileSelect(item.id, path);
            return false;
          }
        },
      })),
      {
        label: "Core Settings...",
        select: () => {
          return coreSettingsMenu(core);
        },
      },
      {
        label: "Reset Core",
        select: () => {
          core.reset();
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
