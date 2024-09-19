import * as ui from "@:golem/ui";

export async function downloads_menu() {
  await ui.textMenu({
    title: "Download Center",
    back: true,
    items: [
      {
        label: "Catalogs...",
        marker: "(0)",
        select: () => {},
      },
    ],
  });
}
