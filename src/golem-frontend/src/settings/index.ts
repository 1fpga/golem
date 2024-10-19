import * as ui from "@:golem/ui";
import * as golemSettings from "@:golem/settings";
import { commands_settings_menu } from "./commands";
import { Games, StartOn, StartOnSetting, UserSettings } from "$/services";

const fontSizes: golemSettings.FontSize[] = ["small", "medium", "large"];

function fontSizeMarker() {
  switch (golemSettings.getSettings().ui?.menuFontSize || "medium") {
    case "small":
      return "Small";
    case "medium":
      return "Medium";
    case "large":
      return "Large";
  }
}

function updateFontSize<T>(item: ui.TextMenuItem<T>) {
  const current = golemSettings.getSettings().ui?.menuFontSize || "medium";
  const next = fontSizes[(fontSizes.indexOf(current) + 1) % fontSizes.length];
  golemSettings.updateSettings({ ui: { menuFontSize: next } });
  item.marker = fontSizeMarker();
}

const datetimeFormats: golemSettings.DateTimeFormat[] = [
  "default",
  "short",
  "timeOnly",
  "hidden",
];

function datetimeFormat() {
  return golemSettings.getSettings().ui?.toolbarDatetimeFormat || "default";
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

function updateDateTimeFormat<T>(item: ui.TextMenuItem<T>) {
  const current = datetimeFormat();
  const next =
    datetimeFormats[
      (datetimeFormats.indexOf(current) + 1) % datetimeFormats.length
    ];
  golemSettings.updateSettings({ ui: { toolbarDatetimeFormat: next } });
  item.marker = datetimeFormatMarker();
}

async function start_options_menu(settings: UserSettings) {
  const labels = {
    [StartOn.MainMenu]: "Main Menu",
    [StartOn.GameLibrary]: "Game Library",
    [StartOn.LastGamePlayed]: "Last Game Played",
    [StartOn.SpecificGame]: "Specific Game",
  };

  let startMenuKind: StartOn = (await settings.startOn()).kind as StartOn;

  await ui.textMenu({
    back: false,
    title: "Startup Options",
    items: [
      {
        label: "Start on:",
        marker: labels[startMenuKind],
        select: async (item) => {
          // Cannot select specific game from this menu.
          const keys = Object.keys(labels);
          startMenuKind = keys[
            (keys.indexOf(startMenuKind) + 1) % keys.length
          ] as StartOn;

          let startOn: StartOnSetting;
          switch (startMenuKind) {
            case StartOn.SpecificGame:
              const game = await Games.select({
                title: "Select a game",
                details: false,
              });
              if (game) {
                startOn = {
                  kind: startMenuKind,
                  game: game.id,
                };
              } else {
                while (startMenuKind === StartOn.SpecificGame) {
                  startMenuKind = keys[
                    (keys.indexOf(startMenuKind) + 1) % keys.length
                  ] as StartOn;
                }
                startOn = {
                  kind: startMenuKind,
                };
              }
              break;
            default:
              startOn = {
                kind: startMenuKind,
              };
              break;
          }

          item.marker = labels[startMenuKind];
          await settings.setStartOn(startOn);
          console.log("Start on:", JSON.stringify(await settings.startOn()));
        },
      },
    ],
  });
}

export async function settings_menu() {
  const settings = await UserSettings.forLoggedInUser();

  await ui.textMenu<boolean>({
    back: false,
    title: "Settings",
    items: [
      {
        label: "Show FPS",
        select: (item) => {
          let current = golemSettings.getSettings().ui?.showFps || false;
          golemSettings.updateSettings({ ui: { showFps: !current } });
          item.marker = golemSettings.getSettings().ui?.showFps ? "On" : "Off";
        },
        marker: golemSettings.getSettings().ui?.showFps ? "On" : "Off",
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
      "---",
      {
        label: "Startup Options",
        select: async () => await start_options_menu(settings),
      },
    ],
  });
}
