import * as ui from "@:golem/ui";
import * as golemSettings from "@:golem/settings";
import { shortcutsMenu } from "./settings/shortcuts";
import {
  DatetimeUpdate,
  Games,
  StartOn,
  StartOnSetting,
  UserSettings,
} from "$/services";

const FONT_SIZE_LABELS: { [key in golemSettings.FontSize]: string } = {
  small: "Small",
  medium: "Medium",
  large: "Large",
};

const DATETIME_FORMAT_LABELS: {
  [key in golemSettings.DateTimeFormat]: string;
} = {
  default: "Default",
  short: "Short",
  timeOnly: "Time Only",
  hidden: "Hidden",
};

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

export async function uiSettingsMenu(settings: UserSettings) {
  await ui.textMenu({
    title: "UI Settings",
    back: 0,
    items: [
      {
        label: "Show FPS",
        marker: (await settings.getShowFps()) ? "On" : "Off",
        select: async (item) => {
          await settings.toggleShowFps();
          item.marker = (await settings.getShowFps()) ? "On" : "Off";
        },
      },
      {
        label: "Font Size",
        marker: FONT_SIZE_LABELS[await settings.getFontSize()],
        select: async (item) => {
          await settings.toggleFontSize();
          item.marker = FONT_SIZE_LABELS[await settings.getFontSize()];
        },
      },
      {
        label: "Toolbar Date Format",
        marker: DATETIME_FORMAT_LABELS[await settings.getDatetimeFormat()],
        select: async (item) => {
          await settings.toggleDatetimeFormat();
          item.marker =
            DATETIME_FORMAT_LABELS[await settings.getDatetimeFormat()];
        },
      },
      {
        label: "Invert Toolbar",
        marker: (await settings.getInvertToolbar()) ? "On" : "Off",
        select: async (item) => {
          await settings.toggleInvertToolbar();
          item.marker = (await settings.getInvertToolbar()) ? "On" : "Off";
        },
      },
    ],
  });

  return false;
}

async function setTimezone(settings: UserSettings) {
  return await ui.textMenu({
    title: "Pick a Timezone",
    back: null,
    items: [
      ...golemSettings.listTimeZones().map((tz) => ({
        label: tz,
        select: async () => {
          golemSettings.setTimeZone(tz);
          await settings.setTimeZone(tz);
          return tz;
        },
      })),
    ],
  });
}

async function setDateTimeMenu(settings: UserSettings) {
  interface DateTimeValues {
    title: string;
    value: string;

    choices(): string[];
  }

  let date = new Date();
  const values: DateTimeValues[] = [
    {
      get title() {
        return `Year (${date.getFullYear()})`;
      },
      get value() {
        return date.getFullYear().toString();
      },
      set value(value) {
        date.setFullYear(parseInt(value, 10));
      },
      choices: () =>
        Array.from({ length: 100 }, (_, i) =>
          (date.getFullYear() + i - 50).toString(),
        ),
    },
    {
      get title() {
        return `Month (${date.getFullYear()}-${date.getMonth() + 1})`;
      },
      get value() {
        return (date.getMonth() + 1).toString();
      },
      set value(value) {
        date.setMonth(parseInt(value, 10) - 1);
      },
      choices: () => Array.from({ length: 12 }, (_, i) => (i + 1).toString()),
    },
    {
      get title() {
        return `Day (${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()})`;
      },
      get value() {
        return date.getDate().toString();
      },
      set value(value) {
        date.setDate(parseInt(value, 10));
      },
      choices: () => {
        const daysInMonth = new Date(
          date.getFullYear(),
          date.getMonth() + 1,
          0,
        ).getDate();
        return Array.from({ length: daysInMonth }, (_, i) =>
          (i + 1).toString(),
        );
      },
    },
  ];

  let i = 0;
  while (i < values.length) {
    const menu = values[i];
    const pick = await ui.textMenu({
      title: menu.title,
      back: -1,
      items: [
        ...menu.choices().map((choice) => ({
          label: choice,
          select: async () => {
            menu.value = choice;
            return 0;
          },
        })),
      ],
    });

    if (pick === -1) {
      if (i === 0) {
        return null;
      }
      i--;
    } else {
      i++;
    }
  }

  golemSettings.setDateTime(date);
  return date;
}

async function settingsMenuDateTime(settings: UserSettings) {
  while (true) {
    const auto =
      (await settings.getDateTimeUpdate()) === DatetimeUpdate.Automatic;
    const d = new Date();
    let items: ui.TextMenuItem<any>[] = [];
    if (!auto) {
      items = [
        {
          label: "Set TimeZone...",
          marker: await settings.getTimeZone(
            golemSettings.getTimeZone() ?? "UTC",
          ),
          select: async (item: ui.TextMenuItem<undefined>) => {
            const newTZ = await setTimezone(settings);
            if (newTZ !== null) {
              item.marker = newTZ;
              await settings.setTimeZone(newTZ);
            }
          },
        },
        {
          label: "Set Date",
          marker: `${d.getFullYear()}-${d.getMonth() + 1}-${d.getDate()}`,
          select: async (item) => {
            const n = await setDateTimeMenu(settings);
            if (n) {
              item.marker = `${n.getFullYear()}-${n.getMonth() + 1}-${n.getDate()}`;
            }
          },
        },
      ];
    }

    const result = await ui.textMenu({
      title: "Date and Time",
      back: 0,
      items: [
        {
          label: "Update Date and Time",
          marker: auto ? "Automatic" : "Manual",
          select: async (item) => {
            await settings.setDateTimeUpdate(
              auto ? DatetimeUpdate.Manual : DatetimeUpdate.Automatic,
            );
            return 1;
          },
        },
        ...items,
      ],
    });
    if (result === 0) {
      break;
    }
  }

  return false;
}

export async function settingsMenu() {
  const settings = await UserSettings.forLoggedInUser();
  let reloadMainMenu = false;

  await ui.textMenu({
    back: 0,
    title: "Settings",
    items: [
      {
        label: "UI...",
        select: async () => {
          if (await uiSettingsMenu(settings)) {
            reloadMainMenu = true;
          }
        },
      },
      {
        label: "Date and Time...",
        select: async () => {
          if (await settingsMenuDateTime(settings)) {
            reloadMainMenu = true;
          }
        },
      },
      "---",
      {
        label: "Shortcuts...",
        select: shortcutsMenu,
      },
      "---",
      {
        label: "Startup Options",
        select: async () => await start_options_menu(settings),
      },
      "-",
      {
        label: "Developer Tools",
        marker: (await settings.getDevTools()) ? "On" : "Off",
        select: async (item) => {
          await settings.toggleDevTools();
          item.marker = (await settings.getDevTools()) ? "On" : "Off";
          reloadMainMenu = true;
        },
      },
    ],
  });

  return reloadMainMenu ? true : undefined;
}
