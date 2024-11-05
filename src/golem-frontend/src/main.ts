// The root file being executed by Golem by default.
import environment from "consts:environment";
import * as ui from "@:golem/ui";
import {
  Catalog,
  Commands,
  Core,
  Games,
  GlobalSettings,
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
import { downloadCenterMenu } from "$/ui/downloads";
import { about } from "$/ui/about";

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
    const downloadMarker = (await Catalog.count(true)) > 0 ? "!" : "";

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
          marker: downloadMarker,
          select: async () => await downloadCenterMenu(),
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
    await Commands.login(user, true);
  } else {
    await Commands.login(user, false);
  }

  const [settings, global] = await Promise.all([
    UserSettings.init(user),
    GlobalSettings.init(),
  ]);
  let startOn = await settings.startOn();

  console.log("Starting on:", JSON.stringify(startOn));
  console.log("Date: ", new Date());

  let action = undefined;

  while (true) {
    try {
      if (action === undefined) {
        return await mainMenu(startOn, settings);
      } else if (action instanceof StartGameAction) {
        await action.game.launch();
      }
      action = undefined;
      startOn = { kind: StartOnKind.MainMenu };
    } catch (e: any) {
      action = undefined;
      startOn = { kind: StartOnKind.MainMenu };
      if (e instanceof StartGameAction) {
        // Set the action for the next round.
        action = e;
      } else if (e instanceof MainMenuAction) {
        // There is a quirk here that if the StartOn is GameLibrary, we will go back
        // to the game library instead of the main menu.
        switch ((await settings.startOn()).kind) {
          case StartOnKind.GameLibrary:
            startOn = { kind: StartOnKind.GameLibrary };
        }
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
  }
}

export async function main() {
  console.log("Golem frontend started: ", JSON.stringify(ONE_FPGA));
  let quit = false;
  console.log(environment);

  while (!quit) {
    quit = await mainInner();
  }
}
