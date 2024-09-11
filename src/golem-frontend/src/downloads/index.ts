import * as ui from "@:golem/ui";

import { sources_menu, Storage } from "./sources";

export async function downloads_menu() {
  const sources = new Storage().sources;

  await ui.textMenu({
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
