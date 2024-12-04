// The root file being executed by 1FPGA by default.
import rev from "consts:revision";
import production from "consts:revision";
import * as osd from "1fpga:osd";
import * as video from "1fpga:video";
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
import { settingsMenu } from "$/ui/settings";
import { login } from "$/ui/login";
import { downloadCenterMenu } from "$/ui/downloads";
import { about } from "$/ui/about";
import { resetDb } from "$/utils";

// Polyfill for events.
globalThis.performance = <any>{
  now: () => Date.now(),
};

async function debugMenu() {
  await osd.textMenu({
    title: "Debug",
    back: false,
    items: [
      {
        label: "Reset All...",
        select: async () => {
          await osd.alert("Reset everything");
        },
      },
      {
        label: "Input Tester...",
        select: async () => {
          await osd.inputTester();
        },
      },
    ],
  });
}

async function mainMenu(
  user: User,
  startOn: StartOnSetting,
  settings: UserSettings,
) {
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
    const nbGames = await Games.count({ mergeByGameId: true });
    const nbCores = await Core.count();

    const gamesMarker = nbGames > 0 ? `(${nbGames})` : "";
    const coresMarker = nbCores > 0 ? `(${nbCores})` : "";
    const downloadMarker = (await Catalog.count(true)) > 0 ? "!" : "";

    await osd.textMenu({
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
        ...(user.admin
          ? [
              {
                label: "Download Center...",
                marker: downloadMarker,
                select: async () => await downloadCenterMenu(),
              },
            ]
          : []),
        {
          label: "Controllers...",
          select: async () => {
            await osd.alert("Controllers", "Not implemented yet.");
          },
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
 * Initialize the application.
 */
async function initAll() {
  // Before setting commands (to avoid commands to interfere with the login menu),
  // we need to initialize the user.
  let user = await login();

  if (user === null) {
    // Run first time setup.
    await (await import("./ui/wizards/first-time-setup")).firstTimeSetup();
    user = await User.login(undefined, true);

    if (user === null) {
      await osd.alert(
        "Error",
        stripIndents`
          Could not log in after initial setup. This is a bug.

          Please report this issue to the developers.

          The application will now exit.
        `,
      );
      return {};
    }
    await Commands.login(user, true);
  } else {
    await Commands.login(user, false);
  }

  const [settings, global] = await Promise.all([
    UserSettings.init(user),
    GlobalSettings.init(),
  ]);

  return { user, settings, global };
}

/**
 * Main function of the frontend.
 * @returns `true` if the application should exit.
 */
async function mainInner(): Promise<boolean> {
  const { user, settings, global } = await initAll();

  if (!user || !settings || !global) {
    const choice = await osd.alert({
      title: "Error",
      message:
        "Could not initialize the application. Do you want to reset everything?",
      choices: ["Reset", "Exit"],
    });
    if (choice === 1) {
      // Clear the database.
      await resetDb();
      return true;
    } else {
      return false;
    }
  }

  let startOn = await settings.startOn();

  console.log("Starting on:", JSON.stringify(startOn));
  console.log("Date: ", new Date());

  let action = undefined;

  while (true) {
    try {
      if (action === undefined) {
        return await mainMenu(user, startOn, settings);
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
        let choice = await osd.alert({
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
  console.log(`Build: "${rev}" (${production ? "production" : "development"})`);
  console.log("1FPGA started. ONE_FPGA =", JSON.stringify(ONE_FPGA));
  let quit = false;

  const start = Date.now();
  const resolution = video.getResolution();
  let image = await Image.embedded("background");

  if (resolution) {
    console.log("Resolution:", resolution.width, "x", resolution.height);
    const imageAr = image.width / image.height;
    const resolutionAr = resolution.width / resolution.height;
    if (imageAr > resolutionAr) {
      resolution.width = resolution.height * imageAr;
    } else if (imageAr < resolutionAr) {
      resolution.height = resolution.width / imageAr;
    }
    image = image.resize(resolution.width, resolution.height);
  }

  image.sendToBackground({ position: "center", clear: true });
  console.log("Background set in", Date.now() - start, "ms");

  while (!quit) {
    quit = await mainInner();
  }
}
