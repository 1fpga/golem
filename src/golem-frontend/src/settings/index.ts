import * as ui from "@:golem/ui";
import * as settings from "@:golem/settings";
import { commands_settings_menu } from "./commands";

const fontSizes: settings.FontSize[] = ["small", "medium", "large"];

function fontSizeMarker() {
  switch (settings.getSettings().ui?.menuFontSize || "medium") {
    case "small":
      return "Small";
    case "medium":
      return "Medium";
    case "large":
      return "Large";
  }
}

function updateFontSize(item: ui.TextMenuItem<void>) {
  const current = settings.getSettings().ui?.menuFontSize || "medium";
  const next = fontSizes[(fontSizes.indexOf(current) + 1) % fontSizes.length];
  settings.updateSettings({ ui: { menuFontSize: next } });
  item.marker = fontSizeMarker();
}

const datetimeFormats: settings.DateTimeFormat[] = [
  "default",
  "short",
  "timeOnly",
  "hidden",
];

function datetimeFormat() {
  return settings.getSettings().ui?.toolbarDatetimeFormat || "default";
}

function datetimeFormatMarker() {
  switch (datetimeFormat()) {
    case "default":
      return "Default";
    case "short":
      return "Short";
    case "timeOnly":
      return "Time Only";
    case "hidden":
      return "Hidden";
  }
}

function updateDateTimeFormat(item: ui.TextMenuItem<void>) {
  const current = datetimeFormat();
  const next =
    datetimeFormats[
      (datetimeFormats.indexOf(current) + 1) % datetimeFormats.length
    ];
  settings.updateSettings({ ui: { toolbarDatetimeFormat: next } });
  item.marker = datetimeFormatMarker();
}

export async function settings_menu() {
  await ui.textMenu({
    back: () => {},
    title: "Settings",
    items: [
      {
        label: "Show FPS",
        select: () => {
          let current = settings.getSettings().ui?.showFps || false;
          settings.updateSettings({ ui: { showFps: !current } });
        },
        marker: settings.getSettings().ui?.showFps ? "On" : "Off",
      },
      {
        label: "Font Size",
        select: updateFontSize,
        marker: fontSizeMarker(),
      },
      {
        label: "Toolbar Date Format",
        select: updateDateTimeFormat,
        marker: datetimeFormatMarker(),
      },
      {
        label: "Shortcuts...",
        marker: ">",
        select: commands_settings_menu,
      },
      {
        label: "Save States",
        select: () => {},
      },
    ],
  });
}
