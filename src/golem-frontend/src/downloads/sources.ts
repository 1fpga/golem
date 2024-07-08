import * as storage from "@/golem/storage";
import * as net from "@/golem/net";
import * as ui from "@/golem/ui";
import type {Source} from "$schemas:source";

export interface SourceWithUrl extends Source {
    _url: string;
}

export class Storage {
    get sources(): SourceWithUrl[] {
        return storage.get("downloadSources") || [];
    }

    set sources(sources: SourceWithUrl[]) {
        storage.set("downloadSources", sources);
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
    // A validate source.
    const validateSource = await import("$schemas:source").then((m) => m.default);

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

    if (validateSource(maybeJson)) {
        return {_url: url, ...maybeJson};
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

async function manage_source(source: SourceWithUrl) {
    const storage = new Storage();
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
