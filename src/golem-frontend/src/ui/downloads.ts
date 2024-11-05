import * as ui from "@:golem/ui";
import { Catalog, RemoteCatalog, WellKnownCatalogs } from "$/services";

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
  // let selected = new Set<string>();
  // let systems = await catalog.listSystems();
  //
  // let shouldInstall = await ui.textMenu({
  //   title: "Choose Systems to install",
  //   back: false,
  //   items: [
  //     ...(
  //       await Promise.all(
  //         Object.entries(systems).map(async ([_key, system]) => {
  //           const remote = await system.fetchRemote();
  //           const cores = Object.values(await remote.fetchCores());
  //           const coreSize = cores.reduce(
  //             (a, b) =>
  //               a + b.latestRelease.files.reduce((a, b) => a + b.size, 0),
  //             0,
  //           );
  //           return [
  //             {
  //               label: system.name,
  //               marker: selected.has(system.uniqueName) ? "install" : "",
  //               select: (item: ui.TextMenuItem<boolean>) => {
  //                 if (selected.has(system.uniqueName)) {
  //                   selected.delete(system.uniqueName);
  //                   item.marker = "";
  //                 } else {
  //                   selected.add(system.uniqueName);
  //                   item.marker = "install";
  //                 }
  //               },
  //             },
  //             ...(cores.length > 1
  //               ? [
  //                   {
  //                     label: "  - Cores:",
  //                     marker: "" + cores.length,
  //                   },
  //                 ]
  //               : []),
  //             {
  //               label: "  - Size:",
  //               marker: filesize(remote.size + coreSize),
  //             },
  //           ];
  //         }),
  //       )
  //     ).flat(),
  //     "-",
  //     { label: "Install selected systems", select: () => true },
  //   ],
  // });
  //
  // if (shouldInstall) {
  //   for (const system of Object.values(systems)) {
  //     if (selected.has(system.uniqueName)) {
  //       await system.installAll(catalog);
  //     }
  //   }
  //   return true;
  // } else {
  //   await ui.alert(
  //     "Warning",
  //     stripIndents`
  //       Skipping core installation. This may cause some games to not work.
  //       You can always install cores later in the Download Center.
  //     `,
  //   );
  //   return false;
  // }
}

export async function downloadCenterMenu() {
  const catalogs = await Catalog.listCatalogs();
  let done = false;
  let refresh = false;
  while (!done) {
    done = await ui.textMenu({
      title: "Download Center",
      back: true,
      items: [
        {
          label: "Check for updates...",
          select: async () => {
            await Catalog.checkForUpdates();
            refresh = true;
            return false;
          },
        },
        {
          label: "Update All...",
          select: async () => {
            refresh = refresh || (await Catalog.updateAll());
            return false;
          },
        },
        "-",
        ...catalogs.map((c) => ({
          label: c.name,
          marker: c.updatePending ? "!" : "",
          select: async () => {
            refresh = refresh || (await catalogDetails(c));
          },
        })),
        "-",
        {
          label: "Add a new Catalog...",
          select: async () => {
            refresh = refresh || (await addNewCatalog());
          },
        },
      ],
    });
  }

  return refresh;
}
