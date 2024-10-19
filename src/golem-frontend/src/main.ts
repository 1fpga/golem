// The root file being executed by Golem by default.
import * as ui from "@:golem/ui";

import { games_menu } from "./ui/games";
import { cores_menu } from "./ui/cores";
import { settings_menu } from "./settings";
import { downloads_menu } from "./downloads";
import { about } from "./ui/about";
import { initCommands } from "./ui/commands";
import { login } from "./ui/login";
import { Games, StartOn, StartOnSetting, User, UserSettings } from "$/services";
import { stripIndents } from "common-tags"; // Polyfill for events.

// Polyfill for events.
globalThis.performance = <any>{
  now: () => Date.now(),
};

async function main_menu(startOn: StartOnSetting) {
  let quit = false;
  let logout = false;

  // Check the startOn option.
  switch (startOn.kind) {
    case StartOn.GameLibrary:
      await games_menu();
      break;
    case StartOn.LastGamePlayed:
      {
        const game = await Games.lastPlayed();
        if (game) {
          await game.launch();
        } else {
          await games_menu();
          break;
        }
      }
      break;
    case StartOn.SpecificGame:
      {
        const game = await Games.byId(startOn.game);
        if (game) {
          await game.launch();
        }
      }
      break;

    case StartOn.MainMenu:
    default:
      break;
  }

  // There are no back menu, but we still need to loop sometimes (when selecting a game, for example).
  while (!(quit || logout)) {
    const nb_games = await Games.count({});
    const nb_cores = 0;

    const games_lbl = nb_games > 0 ? `(${nb_games})` : "";
    const cores_lbl = nb_cores > 0 ? `(${nb_cores})` : "";

    await ui.textMenu({
      title: "",
      items: [
        {
          label: "Game Library",
          select: games_menu,
          marker: games_lbl,
        },
        { label: "Cores", select: cores_menu, marker: cores_lbl },
        "---",
        {
          label: "Settings...",
          select: settings_menu,
        },
        { label: "Download Center...", select: downloads_menu },
        "---",
        { label: "About", select: about },
        "---",
        { label: "Logout", select: () => (logout = true) },
        { label: "Exit", select: () => (quit = true) },
      ],
    });
  }

  if (quit) {
    return true;
  } else if (logout) {
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
  }

  await initCommands();

  try {
    const settings = await UserSettings.forLoggedInUser();
    const startOn = await settings.startOn();
    console.log("Starting on:", JSON.stringify(startOn));

    return await main_menu(startOn);
  } catch (e: any) {
    console.error(e);
    await ui.alert("Error", e.toString());
    return false;
  }
}

export async function main() {
  console.log("Golem frontend started: ", JSON.stringify(ONE_FPGA));
  let quit = false;

  while (!quit) {
    quit = await mainInner();
  }
}
