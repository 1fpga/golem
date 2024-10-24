import * as ui from "@:golem/ui";

export async function downloads_menu() {
  await ui.textMenu({
    title: "Download Center",
    back: 0,
    items: [
      {
        label: "Catalogs...",
        marker: "(0)",
        select: () => {},
      },
    ],
  });
}
