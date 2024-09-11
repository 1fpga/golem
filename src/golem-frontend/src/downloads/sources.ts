import * as storage from "@:golem/storage";
import * as net from "@:golem/net";
import * as ui from "@:golem/ui";
import { validate as validateSource, Source } from "$schemas:source/source";

export interface SourceWithUrl extends Source {
  _url: string;
}

export class Storage {
  get sources(): SourceWithUrl[] {
    const sources = storage.get("downloadSources") || [];

    // Validate sources because people can mess with storage outside the application.
    if (!Array.isArray(sources)) {
      return [];
    }

    if (sources.every((s) => validateSource(s) && typeof s._url === "string")) {
      return sources as SourceWithUrl[];
    } else {
      return [];
    }
  }

  set sources(sources: SourceWithUrl[]) {
    storage.set("downloadSources", sources as storage.StorageItem);
  }

  addOrUpdateSource(source: SourceWithUrl) {
    let sources = this.sources;
    let maybeIndex = sources.findIndex((s) => s._url === source._url);
    if (maybeIndex !== -1) {
      sources[maybeIndex] = source;
    } else {
      sources = [...this.sources, source];
    }
    this.sources = sources;
  }

  deleteSourceByUrl(url: string) {
    this.sources = this.sources.filter((s) => s._url !== url);
  }
}

async function fetchSource(url: string): Promise<SourceWithUrl> {
  function inner(url: string) {
    try {
      return net.fetchJson(url + "/golem.json");
    } catch (e) {
      return fetchSource(url.replace("http://", "https://"));
    }
  }

  // Normalize the URL.
  if (!url.startsWith("https://") && !url.startsWith("http://")) {
    url = "https://" + url;
  }

  const maybeJson = inner(url);

  // Validate source function.
  if (validateSource(maybeJson)) {
    return { _url: url, ...maybeJson };
  }

  throw new Error(
    (validateSource.errors || []).map((e) => e.message || "").join("\n"),
  );
}

async function addSourceMenu() {
  let url = ui.prompt("Add Source", "Enter the URL of the source:");

  if (url === undefined) {
    return;
  }

  // Try to reach the source.
  let newSource;
  try {
    newSource = await fetchSource(url);
  } catch (e) {
    ui.alert("Error", "" + e);
    return false;
  }

  // Update the storage.
  const storage = new Storage();
  storage.addOrUpdateSource(newSource);

  return true;
}

async function exploreSource(source: SourceWithUrl) {
  await ui.textMenu({
    title: source._url,
    back: true,
    items: [
      { label: "Name:", marker: source.name },
      { label: "Url:", marker: source._url },
    ],
  });
}

async function manage_source(source: SourceWithUrl) {
  const storage = new Storage();
  await ui.textMenu({
    title: source.name,
    back: true,
    items: [
      { label: "Url:", marker: source._url },
      "-",
      {
        label: "Rename...",
        select: () => {
          const newName = ui.prompt({
            title: "Rename Source",
            message: "Enter the new name:",
            default: source.name,
          });
          if (newName !== undefined) {
            source.name = newName;
            storage.addOrUpdateSource(source);
          }
        },
      },
      {
        label: "Explore...",
        select: () => {
          ui.alert("Explore", "Not implemented yet.");
          return true;
        },
      },
      {
        label: "Delete Source...",
        select: () => {
          storage.deleteSourceByUrl(source._url);
          return true;
        },
      },
    ],
  });

  return true;
}

export async function sources_menu() {
  while (true) {
    const storage = new Storage();
    const result = await ui.textMenu({
      title: "Sources",
      back: false,
      items: [
        {
          label: "Add Source...",
          select: addSourceMenu,
        },
        "-",
        ...storage.sources.map((source) => ({
          label: source.name,
          select: () => manage_source(source),
        })),
      ],
    });

    if (!result) {
      return;
    }
  }
}
