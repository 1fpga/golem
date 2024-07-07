import * as storage from "@/golem/storage";
import * as net from "@/golem/net";
import * as ui from "@/golem/ui";

export interface Source {
    // The URL loaded for the source.
    _url: string;

    // The name of the source. This is self identified by the source JSON
    // so it is not required to be unique.
    name: string;
}

export class Storage {
    get sources(): Source[] {
        return storage.get("downloadSources") || [];
    }

    set sources(sources: Source[]) {
        storage.set("downloadSources", sources);
    }

    addOrUpdateSource(source: Source) {
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

async function fetchSource(url: string): Promise<Source> {
    // A validate source.
    const validateSource = await import("$schemas:source").then((m) => m.default);

    function inner(url: string) {
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

    // Normalize the URL.
    if (!url.startsWith("https://") && !url.startsWith("http://")) {
        url = "https://" + url;
    }

    const maybeJson = inner(url);
    const isValid = validateSource(maybeJson);

    if (!isValid) {
        throw new Error(
            (validateSource.errors || []).map((e: any) => e.message || "").join("\n"),
        );
    }

    return {_url: url, ...maybeJson};
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

async function manage_source(source: Source) {
    const storage = new Storage();
    console.log("Managing source", source._url, source.name);
    await ui.textMenu({
        title: source.name,
        back: true,
        items: [
            {label: "Url:", marker: source._url},
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
