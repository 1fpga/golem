import { Ajv } from "ajv";
import * as storage from "@/golem/storage";
import * as net from "@/golem/net";
import * as ui from "@/golem/ui";

const schema = new Ajv();

// A validate source.
const validateSource = schema.compile({
  type: "object",
  properties: {
    name: { type: "string" },
    cores: { type: "string" },
    systems: { type: "string" },
  },
  required: ["name"],
});

export interface Source {
  baseUrl: string;
  name: string;
}

export class Storage {
  get sources(): Source[] {
    return storage.get("downloadSources") || [];
  }

  set sources(sources: Source[]) {
    storage.set("downloadSources", sources);
  }

  addSource(source: Source) {
    if (!this.sources.some((s) => s.baseUrl === source.baseUrl)) {
      this.sources = [...this.sources, source];
    }
  }

  removeSource(url: string) {
    this.sources = this.sources.filter((s) => s.baseUrl !== url);
  }
}

function fetchSource(url: string): any {
  function inner() {
    try {
      return net.fetchJson(url + "/golem.json");
    } catch (e) {
      if (url.startsWith("http://")) {
        return fetchSource(url.replace("http://", "https://"));
      } else if (!url.startsWith("https://")) {
        return fetchSource("https://" + url);
      }
    }
  }

  const maybeJson = inner();
  console.log(typeof maybeJson, JSON.stringify(maybeJson));
  const isValid = validateSource(maybeJson);
  console.log(isValid, JSON.stringify(validateSource.errors));
  if (!isValid) {
    throw new Error(
      (validateSource.errors || []).map((e) => e.message || "").join("\n"),
    );
  }

  return maybeJson;
}

function add_source_menu() {
  let baseUrl = ui.prompt("Add Source", "Enter the URL of the source:");

  if (baseUrl === undefined) {
    return;
  }

  // Try to reach the source.
  let newSource;
  try {
    newSource = fetchSource(baseUrl);
  } catch (e) {
    ui.alert("Error", "" + e);
    return false;
  }

  // Update the storage.
  const storage = new Storage();
  storage.addSource({ baseUrl, ...newSource });

  return true;
}

function manage_source(url: string) {
  const storage = new Storage();
  ui.textMenu({
    title: url,
    back: true,
    items: [
      {
        label: "Remove Source...",
        select: () => {
          storage.removeSource(url);
          return true;
        },
      },
    ],
  });
  return true;
}

export function sources_menu() {
  while (true) {
    const storage = new Storage();
    const result = ui.textMenu({
      title: "Sources",
      back: false,
      items: [
        {
          label: "Add Source...",
          select: add_source_menu,
        },
        "-",
        ...storage.sources.map((source) => ({
          label: source.baseUrl,
          select: () => manage_source(source.baseUrl),
        })),
      ],
    });

    if (!result) {
      return;
    }
  }
}
