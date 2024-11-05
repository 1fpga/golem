import * as ui from "@:golem/ui";
import {RemoteCatalog, RemoteCore, RemoteSystem} from "$/services";
import type {Core as CoresCoreSchema} from "$schemas:catalog/cores";
import {filesize} from "filesize";

/**
 * Select cores from a remote catalog options.
 */
export interface SelectCoresOptions {
  /**
   * A predicate to filter cores.
   */
  predicate?: (uniqueName: string, core: CoresCoreSchema) => boolean;

  /**
   * Show an option to install all cores.
   */
  installAll?: boolean;
}

export interface SelectCoresResult {
  cores: RemoteCore[];
  systems: RemoteSystem[];
}

export async function selectCoresFromRemoteCatalog(
  catalog: RemoteCatalog,
  options: SelectCoresOptions = {},
) {
  const predicate = options.predicate ?? (() => true);
  const installAll = options.installAll ?? false;
  let selected = new Set<string>();

  const cores = await catalog.fetchCores(predicate, true);
  const systems = await catalog.fetchSystems((name, system) => {
    return Object.values(cores).some((c) => c.systems.includes(name));
  });

  if (Object.values(systems).length === 0) {
    return { cores: [], systems: [] };
  }

  const items: (ui.TextMenuItem<boolean> | string)[] = [];

  for (const [name, system] of Object.entries(systems).sort(([a], [b]) =>
    a.localeCompare(b),
  )) {
    const systemCores = Object.entries(cores).filter(([_name, core]) => {
      return core.systems.includes(name);
    });

    // If there's only one core, do not show the system name.
    let indent = "";
    let coreStartSize = system.size;
    switch (systemCores.length) {
      case 0:
        continue;
      case 1:
        break;
      default:
        if (items.length > 0) {
          items.push("-");
        }
        items.push({ label: system.name, marker: name });
        indent = "  ";
        if (system.size > 0) {
          coreStartSize = 0;
          items.push({ label: "  Size:", marker: filesize(system.size) });
        }
        items.push("-");
        break;
    }

    for (const [coreName, core] of systemCores) {
      if (core.systems.includes(name)) {
        const coreSize = core.latestRelease.files.reduce(
          (a, b) => a + b.size,
          coreStartSize,
        );

        items.push({
          label: `${indent}${core.name}`,
          marker: selected.has(coreName) ? "install" : "",
          select: (item) => {
            if (selected.has(coreName)) {
              selected.delete(coreName);
              item.marker = "";
            } else {
              selected.add(coreName);
              item.marker = "install";
            }
          },
        });
        if (coreSize > 0) {
          items.push({ label: `${indent}  Size:`, marker: filesize(coreSize) });
        }
      }
    }
  }

  let shouldInstall = await ui.textMenu({
    title: "Choose Cores to install",
    back: false,
    items: [
      ...(installAll
        ? [
            {
              label: "Install All...",
              select: () => {
                for (const core of Object.values(cores)) {
                  selected.add(core.uniqueName);
                }
                return true;
              },
            },
            "-",
          ]
        : []),
      ...items,
      "-",
      { label: "Install selected cores", select: () => true },
    ],
  });

  if (shouldInstall) {
    const coresToInstall = Object.values(cores).filter((core) =>
      selected.has(core.uniqueName),
    );
    const systemsToInstall = Object.values(systems).filter((system) =>
      coresToInstall.some((core) => core.systems.includes(system.uniqueName)),
    );
    return { cores: coresToInstall, systems: systemsToInstall };
  } else {
    return { cores: [], systems: [] };
  }
}
