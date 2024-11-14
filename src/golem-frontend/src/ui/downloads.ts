import * as ui from "@:golem/ui";
import {
  Binary,
  Catalog,
  KnownBinary,
  RemoteCatalog,
  User,
  WellKnownCatalogs,
} from "$/services";

const BINARY_LABELS: { [name: string]: string } = {
  [KnownBinary.OneFpga]: "Update 1FPGA firmware...",
};

async function catalogDetails(c: Catalog) {
  return false;
}

async function addCustomUrl() {
  const url = await ui.prompt("Enter the URL of the new catalog:");
  if (url === undefined) {
    return false;
  }
  const remote = await RemoteCatalog.fetch(url);
  const catalog = await Catalog.create(remote, 0);
  await installCoresFromCatalog(catalog);
  return true;
}

async function addNewCatalog() {
  const has1FpgaCatalog = await Catalog.hasCatalog(WellKnownCatalogs.OneFpga);
  const choices = [
    "Custom URL",
    ...(has1FpgaCatalog ? [] : ["1FPGA"]),
    "Cancel",
  ];

  const choice = await ui.alert({
    title: "Add a new Catalog",
    message:
      "Do you want to add a well known catalog (if available), or enter a URL?",
    choices,
  });
  if (choice === null) {
    return false;
  }

  switch (choice) {
    case 0:
      return await addCustomUrl();
    case 1:
      if (has1FpgaCatalog) {
        const remote = await RemoteCatalog.fetchWellKnown(
          WellKnownCatalogs.OneFpga,
        );
        const catalog = await Catalog.create(remote, 0);
        await installCoresFromCatalog(catalog);

        return true;
      }
      return false;
    default:
      return false;
  }
}

export async function installCoresFromCatalog(catalog: Catalog) {
  throw new Error("Not implemented");
}

async function performBinaryUpdate(b: Binary) {
  const remoteBinary = await b.fetchRemote();
  const release = remoteBinary.latestVersion();
  if (!release) {
    throw new Error("No release found");
  }

  const update = await ui.alert({
    title: `Update ${b.name}`,
    message: `Do you want to update ${b.name} to version ${release.version}?`,
    choices: ["Cancel", "Update and Restart"],
  });

  if (update !== 1) {
    return undefined;
  }

  await b.clean();
  // The line below will kill the process and restart it.
  await release.doUpgrade();
  //
  return false;
}

export async function downloadCenterMenu() {
  if (!User.loggedInUser(true).admin) {
    return undefined;
  }

  let done = false;
  let refresh = false;
  while (!done) {
    const catalogs = await Catalog.listCatalogs();

    // List the binaries to update.
    let binaries = [];
    for (const b of await Binary.listBinaries()) {
      if (!b.updatePending) {
        continue;
      }

      binaries.push({
        label: BINARY_LABELS[b.name] ?? `Update the ${b.name} binary...`,
        marker: b.updatePending ? "!" : "",
        select: async () => {
          return await performBinaryUpdate(b);
        },
      });
    }

    done = await ui.textMenu({
      title: "Download Center",
      back: true,
      items: [
        {
          label: "Check for updates...",
          select: async () => {
            refresh = await Catalog.checkForUpdates();
            if (await Binary.checkForUpdates()) {
              refresh = true;
            }
            return refresh ? false : undefined;
          },
        },
        {
          label: "Update All...",
          select: async () => {
            if (await Catalog.updateAll()) {
              refresh = true;
              return false;
            }
          },
        },
        ...(binaries.length > 0 ? ["-", ...binaries] : []),
        "-",
        "Catalogs:",
        ...catalogs.map((c) => ({
          label: `  ${c.name}`,
          marker: c.updatePending ? "!" : "",
          select: async () => {
            if (await catalogDetails(c)) {
              refresh = true;
              return false;
            }
          },
        })),
        "-",
        {
          label: "Add a new Catalog...",
          select: async () => {
            if (await addNewCatalog()) {
              refresh = true;
              return false;
            }
          },
        },
      ],
    });
  }

  return refresh ? true : undefined;
}
