import * as ui from "@:golem/ui";
import * as net from "@:golem/net";
import { DEFAULT_USERNAME, User, Catalog, RemoteCatalog } from "../../services";
import {
  conditional,
  message,
  repeat,
  sequence,
  map,
  wizard,
  WizardStep,
  skipIf,
  first,
  choice,
  ignore,
  call,
  last,
  generate,
} from "./wizard";
import { TextMenuItem } from "@:golem/ui";
import { Core } from "../../services/database/core";
import type { Files as RemoteCoreFiles } from "$schemas:catalog/core";
import { filesize } from "filesize";

/**
 * A wizard step that prompts the user for a password.
 */
function password(
  title: string,
  message: string,
  length: number,
): WizardStep<string[] | undefined> {
  return async (options) => {
    const password = await ui.promptPassword(title, message, length);

    if (password === null) {
      await options.previous();
      return undefined;
    }
    return password;
  };
}

/**
 * A wizard step that prompts the user to set a password, verify it and
 * will succeed if both match.
 */
function passwordAndVerify(
  title: string = "Set Password",
  length = 4,
): WizardStep<string[] | undefined> {
  const fn = first(
    repeat<[string[], string[]] | undefined>(
      async (matches: [string[], string[]] | undefined) => {
        // User cancelled.
        if (matches === undefined) {
          return false;
        }
        // Passwords don't match.
        if (
          User.passwordToString(matches[0]) !==
          User.passwordToString(matches[1])
        ) {
          const should = await ui.alert({
            title: "Error",
            message: "Passwords do not match. Please try again.",
            choices: ["OK", "Don't set password"],
          });
          return should === 0;
        }
        return false;
      },
      map(
        sequence(
          password(title, "Enter a new password:", length),
          password(title, "Verify your password:", length),
        ),
        async (c) => {
          const [password, check] = c;
          return password && check ? [password, check] : undefined;
        },
      ),
    ),
  );

  // Always 2 steps (set password and verify password).
  fn.count = 2;
  return fn;
}

const createUserWizardStep = map(
  conditional(
    map(
      message(
        "Set Password",
        "By default, there is one default user (named 'admin'). You'll be able to add more later.\n\n" +
          "Would you like to set a password?",
        { choices: ["No", "Yes"] },
      ),
      async (choice) => choice === 1,
    ),
    passwordAndVerify(),
  ),
  async (password) => {
    await User.create(DEFAULT_USERNAME, password ?? null, true);
    const user = await User.login(DEFAULT_USERNAME, true);
    if (user === null) {
      throw new Error("Could not log in the user.");
    }
  },
);

const SHOULD_RETRY_ADD_CATALOG = Symbol.for("SHOULD_RETRY_ADD_CATALOG");

async function add1FpgaCatalog(): Promise<Catalog | null | Symbol> {
  while (true) {
    try {
      const catalog = await RemoteCatalog.fetch1Fpga();
      return await Catalog.create(catalog, 0);
    } catch (e) {
      console.error("Could not add 1fpga catalog:", e);

      const should = await ui.alert({
        title: "Error",
        message: "Could not fetch the catalog. Please try again.",
        choices: ["Retry", "Skip adding catalog", "Back"],
      });
      if (should === 1) {
        return null;
      } else if (should === 2) {
        return SHOULD_RETRY_ADD_CATALOG;
      }
    }
  }
}

async function addCustomCatalog(): Promise<Catalog | null | Symbol> {
  let url: string | null = null;
  while (true) {
    url = (await ui.prompt("Enter the URL of the catalog:")) || null;
    if (url === null) {
      return null;
    }

    try {
      const catalog = await RemoteCatalog.fetch(url);
      return await Catalog.create(catalog, 0);
    } catch (e) {
      console.error("Could not add custom catalog:", e);

      const should = await ui.alert({
        title: "Error",
        message: "Could not fetch the catalog. Please try again.",
        choices: ["Retry", "Skip adding catalog", "Back"],
      });
      if (should === 1) {
        return null;
      } else if (should === 2) {
        return SHOULD_RETRY_ADD_CATALOG;
      }
    }
  }
}

const catalogAddStep = repeat(
  async (result) => result === SHOULD_RETRY_ADD_CATALOG,
  last(
    sequence(
      ignore(
        message(
          "Catalogs - Introduction",
          `
              Catalogs are websites where you can download games and cores from.
              Catalogs can be added or removed later. They require an internet connection when setting up, updating or downloading from.
            `,
        ),
      ),
      conditional(
        map(
          skipIf(
            // Skip this if we're online (only show warning while offline).
            async () => await net.isOnline(),
            repeat<number | undefined>(
              async (choice) => choice === 0 && !(await net.isOnline()),
              message(
                "Catalogs (No Internet Connection)",
                "You need to be online to set up catalogs. Please connect to the internet and try again. " +
                  "You can also skip this step and set up catalogs later.",
                {
                  choices: ["Try again", "Skip"],
                },
              ),
            ),
            0,
          ),
          async (c) => c !== undefined && c === 0,
        ),

        choice(
          "Catalogs",
          "1FPGA comes with a default catalog of officially supported cores and homebrew games.\n " +
            "Its URL is 'https://catalog.1fpga.cloud'.\n\n" +
            "Would you like to add it?",
          [
            [
              "Add the 1FPGA Catalog",
              call(async () => await add1FpgaCatalog()),
            ],
            ["Add custom catalog", call(async () => await addCustomCatalog())],
            ["Skip", async () => null],
          ],
        ),
      ),
    ),
  ),
);

const selectCores = async (catalog: Catalog) => {
  let selected = new Set<string>();
  let systems = await catalog.listSystems();

  let shouldInstall = await ui.textMenu({
    title: "Choose Systems to install",
    back: false,
    items: [
      ...(
        await Promise.all(
          Object.entries(systems).map(async ([_key, system]) => {
            const remote = await system.fetchRemote();
            const cores = Object.values(await remote.fetchCores());
            const coreSize = cores
              .reduce(
                (a, b) => [...a, ...b.latestRelease.files],
                [] as RemoteCoreFiles,
              )
              .reduce((a, b) => a + b.size, 0);

            return [
              {
                label: system.name,
                marker: selected.has(system.uniqueName) ? "install" : "",
                select: (item: TextMenuItem<boolean>) => {
                  if (selected.has(system.uniqueName)) {
                    selected.delete(system.uniqueName);
                    item.marker = "";
                  } else {
                    selected.add(system.uniqueName);
                    item.marker = "install";
                  }
                },
              },
              {
                label: "  - Cores:",
                marker: "" + Object.keys(await remote.fetchCores()).length,
              },
              {
                label: "  - Size:",
                marker: filesize(remote.size + coreSize),
              },
            ];
          }),
        )
      ).flat(),
      "-",
      { label: "Install selected systems", select: () => true },
    ],
  });

  if (shouldInstall) {
    for (const system of Object.values(systems)) {
      if (selected.has(system.uniqueName)) {
        await system.install(catalog);
      }
    }
  } else {
    await ui.alert(
      "Warning",
      "Skipping core installation. This may cause some games to not work.\n" +
        "You can always install cores later in the Download Center.",
    );
  }
};

const catalogSetup = sequence(
  ignore(
    message(
      "Catalogs - Installing Cores",
      "Choose cores to install from the catalog.",
    ),
  ),
  generate(async () => {
    const [catalog, ...rest] = await Catalog.listCatalogs();
    if (!catalog || rest.length !== 0) {
      throw new Error("Should be exactly 1 catalog during the tutorial.");
    }

    return [call(async () => await selectCores(catalog))];
  }),
  ignore(
    message(
      "Catalogs",
      "Catalogs have been set up. You can always add more catalogs later in the Download Center.",
    ),
  ),
);

// The first message in the first time setup wizard.
// Language has not been selected yet, so we can't use i18next.
const firstMessage = message(
  "First Time Setup",
  "Welcome to 1FPGA. Follow this wizard to get started.\n",
  { choices: ["Let's Go!"] },
);

/**
 * Runs the first time setup wizard.
 */
export async function firstTimeSetup() {
  console.warn("Running first time setup.");
  await wizard<any>(
    firstMessage,
    createUserWizardStep,
    skipIf(
      map(catalogAddStep, async (catalog) => catalog === null),
      catalogSetup,
    ),
    message(
      "First Time Setup",
      "You're all set up! Welcome to 1FPGA and enjoy your stay.",
      { choices: ["Thanks"] },
    ),
  );
}
