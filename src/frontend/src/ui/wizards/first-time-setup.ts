import production from "consts:production";
import * as osd from "1fpga:osd";
import * as net from "1fpga:net";
import {
  Binary,
  Catalog,
  Core,
  DEFAULT_USERNAME,
  GamesIdentification,
  RemoteCatalog,
  User,
  WellKnownCatalogs
} from "$/services";
import {
  call,
  choice,
  conditional,
  first,
  generate,
  ignore,
  last,
  map,
  message,
  repeat,
  sequence,
  skipIf,
  value,
  wizard,
  WizardStep
} from "./wizard";
import { oneLine, stripIndents } from "common-tags";
import { selectCoresFromRemoteCatalog } from "$/ui/catalog/cores";

/**
 * A wizard step that prompts the user for a password.
 */
function password(
  title: string,
  message: string,
  length: number,
): WizardStep<string[] | undefined> {
  return async (options) => {
    const password = await osd.promptPassword(title, message, length);

    if (password === null) {
      await options.previous();
      return undefined;
    }
    return password;
  };
}

/**
 * A wizard step that prompts the user for a file or directory.
 */
function selectPath(
  title: string,
  options?: { initialDir?: string },
): WizardStep<string | undefined> {
  return async (o) => {
    const path = await osd.selectFile(
      title,
      options?.initialDir ?? "/media/fat",
      {
        directory: true,
      },
    );

    if (path === undefined) {
      await o.previous();
      return undefined;
    }
    return path;
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
          const should = await osd.alert({
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
        stripIndents`
          By default, there is one default user (named 'admin'). You'll be able to add more later.
          
          Would you like to set a password?
        `,
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

async function addWellKnownCatalog(
  wellKnownCatalogs: WellKnownCatalogs,
): Promise<Catalog | null | Symbol> {
  while (true) {
    try {
      const catalog = await RemoteCatalog.fetchWellKnown(wellKnownCatalogs);
      return await Catalog.create(catalog, 0);
    } catch (e) {
      console.error("Could not add 1fpga catalog:", e);

      const should = await osd.alert({
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
    url = (await osd.prompt("Enter the URL of the catalog:")) || null;
    if (url === null) {
      return null;
    }

    try {
      const catalog = await RemoteCatalog.fetch(url);
      return await Catalog.create(catalog, 0);
    } catch (e) {
      console.error("Could not add custom catalog:", e);

      const should = await osd.alert({
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
          stripIndents`
            Catalogs are online repositories where you can download games and cores from.
            Catalogs can be added or removed later.
            They require an internet connection when setting up, updating or downloading from.
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
                oneLine`You need to be online to set up catalogs.
                  Please connect to the internet and try again. 
                  You can also skip this step and set up catalogs later.
                `,
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
          stripIndents`
            1FPGA comes with a default catalog of officially supported cores and homebrew games.
            It also includes updates to 1FPGA itself, if you skip this, you will not be able to update your firmware.
            
            Would you like to add it?
          `,
          [
            [
              "Add the 1FPGA catalog",
              call(
                async () =>
                  await addWellKnownCatalog(WellKnownCatalogs.OneFpga),
              ),
            ],
            ...(!production
              ? [
                  [
                    "Add a local test catalog",
                    call(
                      async () =>
                        await addWellKnownCatalog(WellKnownCatalogs.LocalTest),
                    ),
                  ] as [string, WizardStep<null>],
                ]
              : []),
            ["Add custom catalog", call(async () => await addCustomCatalog())],
            ["Skip", async () => null],
          ],
        ),
      ),
    ),
  ),
);

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

    return call(async () => {
      const remote = await RemoteCatalog.fetch(catalog.url);
      const { cores, systems } = await selectCoresFromRemoteCatalog(remote, {
        installAll: true,
      });
      if (cores.length === 0 && systems.length === 0) {
        await osd.alert(
          "Warning",
          stripIndents`
              Skipping core installation. This may cause some games to not work.
              You can always install cores later in the Download Center.
            `,
        );
        return;
      }

      // Update the binaries table to make sure we check for updates.
      const releases = await remote.fetchReleases();
      for (const name of Object.getOwnPropertyNames(releases)) {
        const binary = releases[name];
        if (binary) {
          await Binary.create(binary, catalog);
        }
      }

      for (const system of (await catalog.listSystems()).filter((s) =>
        systems.some((r) => r.uniqueName === s.uniqueName),
      )) {
        await system.install(catalog);
      }
      for (const core of cores) {
        await Core.install(core, catalog);
      }
    });
  }),
  ignore(
    message(
      "Catalogs",
      "Catalogs have been set up. You can always add more catalogs later in the Download Center.",
    ),
  ),
);

const addGames = sequence(
  map(selectPath("Select the directory with your games"), async (root) => {
    console.log("Selected root: ", root);
    if (root === undefined) {
      return null;
    }

    await GamesIdentification.addGamesFromRoot(root);
  }),
);

const maybeAddGames = ignore(
  choice(
    "Adding Games",
    oneLine`
    If you have games on your SD card, you can add them to the database now.
    This may take a while depending on the number of games you have.
    You can always add (or remove) games later.
  `,
    [
      ["Add My Games", addGames],
      ["Skip", value(null)],
    ],
  ),
);

// The first message in the first time setup wizard.
// Language has not been selected yet, so we can't use i18next.
const firstMessage = message(
  "First Time Setup",
  "Welcome to 1FPGA. Follow this wizard to get started.\n",
  { choices: ["Let's Go!"], noCancel: true },
);

/**
 * Runs the first time setup wizard.
 */
export async function firstTimeSetup() {
  console.warn("Running first time setup.");
  await wizard<any>(
    [
      firstMessage,
      createUserWizardStep,
      skipIf(
        map(catalogAddStep, async (catalog) => catalog === null),
        sequence(catalogSetup, maybeAddGames),
      ),
      message(
        "First Time Setup",
        "You're all set up! Welcome to 1FPGA and enjoy your stay.",
        { choices: ["Thanks"] },
      ),
    ],
    async (err) => {
      await osd.alert({
        title: "Error",
        message: "An unexpected error occurred: " + err.toString(),
      });
    },
  );
}
