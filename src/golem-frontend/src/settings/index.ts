import * as ui from "@/golem/ui";
import * as settings from "@/golem/settings";

export function settings_menu() {
  while (true) {
    const result = ui.textMenu({
      back: () => true,
      title: "Settings",
      items: [
        {
          label: "Show FPS",
          select: () => {
            let current = settings.getSettings().ui?.showFps || false;
            settings.updateSettings({ ui: { showFps: !current } });
            return false;
          },
          marker: settings.getSettings().ui?.showFps ? "On" : "Off",
        },
        {
          label: "Font Size",
          select: (item) => {
            let current = settings.getSettings().ui?.menuFontSize || "medium";
            let options: ("small" | "medium" | "large")[] = [
              "small",
              "medium",
              "large",
            ];
            let index = options.indexOf(current);
            let next = options[(index + 1) % options.length];
            settings.updateSettings({ ui: { menuFontSize: next } });
            item.marker = next;
            console.log(JSON.stringify(item));
            // return false;
          },
          marker: settings.getSettings().ui?.menuFontSize || "medium",
        },
        {
          label: "Commands",
          select: () => {},
        },
      ],
    });

    if (result) {
      return;
    }
  }
}
