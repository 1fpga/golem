// The about page.
import * as ui from "@:golem/ui";
import { default as p } from "@:golem/patrons";

export async function about() {
  let { patrons, tiers } = p;

  // Sort tiers by amount.
  const tiersSorted = Object.entries(tiers)
    .sort((a, b) => Number(b[0]) - Number(a[0]))
    .map(([_amount, title]) => title);

  const patronsList = [];
  for (let tier of tiersSorted) {
    if (patrons[tier]) {
      patronsList.push("-");
      patronsList.push({ label: tier, selectable: false });
      patronsList.push("-");
      for (let patron of patrons[tier]) {
        patronsList.push({ label: patron, select: () => undefined });
      }
    }
  }

  await ui.textMenu({
    title: "About",
    back: true,
    items: [
      {
        label: "< Join the GoLEm FPGA Patreon >",
        marker: "...",
        select: () => {
          ui.qrCode(
            "https://patreon.com/golem_fpga/join",
            "Use this code to join our Patreon community",
          );
        },
      },
      "-",
      "Thanks to our patrons",
      ...patronsList,
    ],
  });
}
