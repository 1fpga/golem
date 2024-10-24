// The root file being executed by Golem by default.
import * as ui from "@:golem/ui";
import {
  Commands,
  Core,
  Games,
  StartOnKind,
  StartOnSetting,
  User,
  UserSettings,
} from "$/services";
import { stripIndents } from "common-tags";
import { StartGameAction } from "$/actions/start_game";
import { MainMenuAction } from "$/actions/main_menu";
import { gamesMenu } from "$/ui/games";
import { coresMenu } from "$/ui/cores"; // Import the basic commands.
import "./commands/basic";
import { settingsMenu } from "$/ui/settings";
import { login } from "$/ui/login";
import { downloads_menu } from "$/downloads";
import { about } from "$/ui/about"; // Polyfill for events.

// Polyfill for events.
globalThis.performance = <any>{
  now: () => Date.now(),
};

async function debugMenu() {
  await ui.textMenu({
    title: "Debug",
    back: false,
    items: [
      {
        label: "Reset All...",
        select: async () => {
          await ui.alert("Reset everything");
        },
      },
      {
        label: "Input Tester...",
        select: async () => {
          await ui.inputTester();
        },
      },
    ],
  });
}

async function mainMenu(startOn: StartOnSetting, settings: UserSettings) {
  let quit = false;
  let logout = false;

  // Check the startOn option.
  switch (startOn.kind) {
    case StartOnKind.GameLibrary:
      await gamesMenu();
      break;
    case StartOnKind.LastGamePlayed:
      {
        const game = await Games.lastPlayed();
        if (game) {
          await game.launch();
        } else {
          await gamesMenu();
          break;
        }
      }
      break;
    case StartOnKind.SpecificGame:
      {
        const game = await Games.byId(startOn.game);
        if (game) {
          await game.launch();
        }
      }
      break;

    case StartOnKind.MainMenu:
    default:
      break;
  }

  // There are no back menu, but we still need to loop sometimes (when selecting a game, for example).
  while (!(quit || logout)) {
    const nbGames = await Games.count({});
    const nbCores = await Core.count();

    const gamesMarker = nbGames > 0 ? `(${nbGames})` : "";
    const coresMarker = nbCores > 0 ? `(${nbCores})` : "";

    await ui.textMenu({
      title: "",
      items: [
        {
          label: "Game Library",
          select: async () => await gamesMenu(),
          marker: gamesMarker,
        },
        {
          label: "Cores",
          select: async () => await coresMenu(),
          marker: coresMarker,
        },
        "---",
        {
          label: "Settings...",
          select: async () => await settingsMenu(),
        },
        {
          label: "Download Center...",
          select: async () => await downloads_menu(),
        },
        "---",
        { label: "About", select: about },
        ...((await settings.getDevTools())
          ? [
              "-",
              {
                label: "Developer Tools",
                select: async () => await debugMenu(),
              },
            ]
          : []),
        "---",
        ...((await User.canLogOut())
          ? [{ label: "Logout", select: () => (logout = true) }]
          : []),
        { label: "Exit", select: () => (quit = true) },
      ],
    });
  }

  if (quit) {
    return true;
  } else if (logout) {
    await Commands.logout();
    await User.logout();
  }
  return false;
}

/**
 * Main function of the frontend.
 * @returns `true` if the application should exit.
 */
async function mainInner(): Promise<boolean> {
  // Before setting commands (to avoid commands to interfere with the login menu),
  // we need to initialize the user.
  let user = await login();

  if (user === null) {
    // Run first time setup.
    await (await import("./ui/wizards/first-time-setup")).firstTimeSetup();
    user = await User.login(undefined, true);

    if (user === null) {
      await ui.alert(
        "Error",
        stripIndents`
          Could not log in after initial setup. This is a bug.

          Please report this issue to the developers.

          The application will now exit.
        `,
      );
      return true;
    }
    await Commands.init(user, true);
  } else {
    await Commands.init(user, false);
  }

  const settings = await UserSettings.init(user);
  let startOn = await settings.startOn();
  console.log("Starting on:", JSON.stringify(startOn));

  await settings.updateDateTimeIfNecessary();
  console.log("Date: ", new Date());

  while (true) {
    try {
      return await mainMenu(startOn, settings);
    } catch (e: any) {
      if (e instanceof StartGameAction) {
        console.log(`Action - Starting game: ${JSON.stringify(e.game.name)}`);
        await e.game.launch();
      } else if (e instanceof MainMenuAction) {
        console.log("Action - Main Menu");
        // Do nothing, just loop.
      } else {
        // Rethrow to show the user the actual error.
        let choice = await ui.alert({
          title: "An error happened",
          message: (e as Error)?.message ?? JSON.stringify(e),
          choices: ["Restart", "Quit"],
        });
        if (choice === 1) {
          return false;
        }
      }
    }
    startOn = { kind: StartOnKind.MainMenu };
  }
}

export async function main() {
  console.log("Golem frontend started: ", JSON.stringify(ONE_FPGA));
  let quit = false;

  while (!quit) {
    quit = await mainInner();
  }
}
