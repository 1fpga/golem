import * as storage from "@/golem/storage";
import * as ui from "@/golem/ui";

import { sources_menu } from "./sources";

export function downloads_menu() {
  const sources = storage.get("downloadSources") || [];

  ui.textMenu({
    title: "Download Center",
    back: true,
    items: [
      {
        label: "Sources...",
        marker: sources.length.toString(),
        select: sources_menu,
      },
    ],
  });
}
