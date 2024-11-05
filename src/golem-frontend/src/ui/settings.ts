import * as ui from "@:golem/ui";
import * as golemSettings from "@:golem/settings";
import { shortcutsMenu } from "./settings/shortcuts";
import {
  CatalogCheckFrequency,
  DatetimeUpdate,
  Games,
  GlobalSettings,
  StartOnKind,
  User,
  UserSettings,
} from "$/services";
import { accountsSettingsMenu } from "$/ui/settings/accounts";

const UPDATE_FREQUENCY_LABELS = {
  [CatalogCheckFrequency.Manually]: "Manually",
  [CatalogCheckFrequency.EveryStartup]: "On Startup",
  [CatalogCheckFrequency.Daily]: "Daily",
  [CatalogCheckFrequency.Weekly]: "Once a week",
  [CatalogCheckFrequency.Monthly]: "Once a month",
};

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

async function startOptionsMenu(settings: UserSettings) {
  const labels = {
    [StartOnKind.MainMenu]: "Main Menu",
    [StartOnKind.GameLibrary]: "Game Library",
    [StartOnKind.LastGamePlayed]: "Last Game Played",
    [StartOnKind.SpecificGame]: "Specific Game",
  };

  let done = false;

  let startOn = await settings.startOn();
  let maybeGame: undefined | Games;
  while (!done) {
    if (startOn.kind === StartOnKind.SpecificGame) {
      maybeGame = await Games.byId(startOn.game);
    }

    done = await ui.textMenu({
      back: true,
      title: "Startup Options",
      items: [
        {
          label: "Start on:",
          marker: labels[startOn.kind],
          select: async (item) => {
            // Cannot select specific game from this menu.
            const keys = Object.keys(labels);
            let kind = keys[
              (keys.indexOf(startOn.kind) + 1) % keys.length
            ] as StartOnKind;

            switch (kind) {
              case StartOnKind.SpecificGame:
                const g = maybeGame ?? (await Games.first());
                if (g) {
                  maybeGame = g;
                  startOn = { kind, game: g.id };
                } else {
                  while (kind === StartOnKind.SpecificGame) {
                    kind = keys[
                      (keys.indexOf(kind) + 1) % keys.length
                    ] as StartOnKind;
                  }
                  startOn = { kind };
                }
                break;
              default:
                startOn = { kind };
                break;
            }

            item.marker = labels[startOn.kind];
            await settings.setStartOn(startOn);
            console.log("Start on:", JSON.stringify(await settings.startOn()));
            return false;
          },
        },
        ...(startOn.kind === StartOnKind.SpecificGame
          ? [
              {
                label: "  ",
                marker: maybeGame?.name ?? "",
                select: async (item: ui.TextMenuItem<boolean>) => {
                  const game = await Games.select({
                    title: "Select a game",
                  });
                  if (game) {
                    maybeGame = game;
                    startOn = { kind: StartOnKind.SpecificGame, game: game.id };
                    await settings.setStartOn(startOn);
                    item.marker = game.name;
                  }
                },
              },
            ]
          : []),
      ],
    });
  }
}

export async function uiSettingsMenu() {
  if (!User.loggedInUser(true).admin) {
    throw new Error("Only admins can change the UI settings.");
  }

  const settings = await GlobalSettings.create();
  await ui.textMenu({
    title: "UI Settings",
    back: 0,
    items: [
      {
        label: "Show FPS",
        marker: (await settings.getShowFps()) ? "On" : "Off",
        select: async (item) => {
          item.marker = (await settings.toggleShowFps()) ? "On" : "Off";
        },
      },
      {
        label: "Font Size",
        marker: FONT_SIZE_LABELS[await settings.getFontSize()],
        select: async (item) => {
          item.marker = FONT_SIZE_LABELS[await settings.toggleFontSize()];
        },
      },
      {
        label: "Toolbar Date Format",
        marker: DATETIME_FORMAT_LABELS[await settings.getDatetimeFormat()],
        select: async (item) => {
          item.marker =
            DATETIME_FORMAT_LABELS[await settings.toggleDatetimeFormat()];
        },
      },
      {
        label: "Invert Toolbar",
        marker: (await settings.getInvertToolbar()) ? "On" : "Off",
        select: async (item) => {
          item.marker = (await settings.toggleInvertToolbar()) ? "On" : "Off";
        },
      },
    ],
  });

  return false;
}

async function setTimezone() {
  const settings = await GlobalSettings.create();
  return await ui.textMenu({
    title: "Pick a Timezone",
    back: null,
    items: [
      ...golemSettings.listTimeZones().map((tz) => ({
        label: tz,
        select: async () => {
          await settings.setTimeZone(tz);
          return tz;
        },
      })),
    ],
  });
}

interface DateTimeMenuValues {
  title: string;
  value: string;

  choices(): string[];
}

/**
 * Show a series of menus to set the date or time.
 * @param values The list of values to set.
 * @returns Whether the user completed the menu (or false if cancelled).
 */
async function setDateTimeUi(values: DateTimeMenuValues[]): Promise<boolean> {
  let i = 0;
  while (i < values.length) {
    const menu = values[i];
    const choices = menu.choices();
    const currentValue = choices.indexOf(menu.value);
    const pick = await ui.textMenu({
      title: menu.title,
      back: -1,
      highlighted: currentValue == -1 ? 0 : currentValue,
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
        return false;
      }
      i--;
    } else {
      i++;
    }
  }
  return true;
}

async function setDateMenu() {
  let date = new Date();
  const completed = await setDateTimeUi([
    {
      get title() {
        return `Year (____-${date.getMonth() + 1}-${date.getDate()})`;
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
        return `Month (${date.getFullYear()}-__-${date.getDate()})`;
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
        return `Day (${date.getFullYear()}-${date.getMonth() + 1}-__)`;
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
  ]);

  if (completed) {
    golemSettings.setDateTime(date);
    return date;
  } else {
    return null;
  }
}

async function setTimeMenu() {
  let date = new Date();
  const completed = await setDateTimeUi([
    {
      get title() {
        return `Hour (__:${date.getMinutes()}:${date.getSeconds()})`;
      },
      get value() {
        return date.getHours().toString();
      },
      set value(value) {
        date.setHours(parseInt(value, 10));
      },
      choices: () => {
        return Array.from({ length: 24 }, (_, i) => i.toString());
      },
    },
    {
      get title() {
        return `Minutes (${date.getHours()}:__:${date.getSeconds()})`;
      },
      get value() {
        return date.getMinutes().toString();
      },
      set value(value) {
        date.setMinutes(parseInt(value, 10));
      },
      choices: () => {
        return Array.from({ length: 60 }, (_, i) => i.toString());
      },
    },
    {
      get title() {
        return `Seconds (${date.getHours()}:${date.getMinutes()}:__)`;
      },
      get value() {
        return date.getSeconds().toString();
      },
      set value(value) {
        date.setSeconds(parseInt(value, 10));
      },
      choices: () => {
        return Array.from({ length: 60 }, (_, i) => i.toString());
      },
    },
  ]);

  if (completed) {
    golemSettings.setDateTime(date);
    return date;
  } else {
    return null;
  }
}

async function settingsMenuDateTime() {
  if (!User.loggedInUser(true).admin) {
    throw new Error("Only admins can change the date and time settings.");
  }

  const settings = await GlobalSettings.create();
  while (true) {
    const type = await settings.getDateTimeUpdate();
    const d = new Date();
    let items: ui.TextMenuItem<any>[] = [];
    if (type !== DatetimeUpdate.Automatic) {
      items.push({
        label: "Set TimeZone...",
        marker: await settings.getTimeZone(
          golemSettings.getTimeZone() ?? "UTC",
        ),
        select: async (item: ui.TextMenuItem<undefined>) => {
          const newTZ = await setTimezone();
          if (newTZ !== null) {
            item.marker = newTZ;
            await settings.setTimeZone(newTZ);
          }
        },
      });
    }
    if (type === DatetimeUpdate.Manual) {
      items.push(
        {
          label: "Set Date",
          marker: `${d.getFullYear()}-${d.getMonth() + 1}-${d.getDate()}`,
          select: async (item) => {
            const n = await setDateMenu();
            if (n) {
              item.marker = `${n.getFullYear()}-${n.getMonth() + 1}-${n.getDate()}`;
            }
          },
        },
        {
          label: "Set Time",
          marker: `${d.getHours()}:${d.getMinutes()}:${d.getSeconds()}`,
          select: async (item) => {
            const n = await setTimeMenu();
            if (n) {
              item.marker = `${n.getHours()}:${n.getMinutes()}:${n.getSeconds()}`;
            }
          },
        },
      );
    }

    let marker;
    switch (type) {
      case DatetimeUpdate.Automatic:
        marker = "Automatic";
        break;
      case DatetimeUpdate.Manual:
        marker = "Manual";
        break;
      case DatetimeUpdate.AutoWithTz:
        marker = "Automatic (with TZ)";
        break;
    }

    const result = await ui.textMenu({
      title: "Date and Time",
      back: 0,
      items: [
        {
          label: "Update Date and Time",
          marker,
          select: async () => {
            const next =
              type === DatetimeUpdate.Automatic
                ? DatetimeUpdate.AutoWithTz
                : type === DatetimeUpdate.AutoWithTz
                  ? DatetimeUpdate.Manual
                  : DatetimeUpdate.Automatic;
            await settings.setDateTimeUpdate(next);
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
  const user = User.loggedInUser(true);
  const settings = await UserSettings.forLoggedInUser();
  const globals = await GlobalSettings.create();
  let reloadMainMenu = false;

  await ui.textMenu({
    back: 0,
    title: "Settings",
    items: [
      ...(user.admin
        ? [
            {
              label: "UI...",
              select: async () => {
                if (await uiSettingsMenu()) {
                  reloadMainMenu = true;
                }
              },
            },
            {
              label: "Date and Time...",
              select: async () => {
                if (await settingsMenuDateTime()) {
                  reloadMainMenu = true;
                }

                await (
                  await GlobalSettings.create()
                ).updateDateTimeIfNecessary();
              },
            },
            {
              label: "Check for Updates",
              marker:
                UPDATE_FREQUENCY_LABELS[
                  await globals.getCatalogCheckFrequency()
                ],
              select: async (item: ui.TextMenuItem<any>) => {
                item.marker =
                  UPDATE_FREQUENCY_LABELS[
                    await globals.toggleCatalogCheckFrequency()
                  ];
              },
            },
          ]
        : []),
      {
        label: "Accounts...",
        select: async () => {
          reloadMainMenu = reloadMainMenu || (await accountsSettingsMenu());
        },
      },
      "---",
      {
        label: "Shortcuts...",
        select: shortcutsMenu,
      },
      {
        label: "Startup Options...",
        select: async () => await startOptionsMenu(settings),
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
