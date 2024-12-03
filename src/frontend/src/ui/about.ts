// The about page.
import * as osd from "1fpga:osd";
import p from "1fpga:patrons";

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

  const oneFpga = ONE_FPGA.version;
  const version = `${oneFpga.major}.${oneFpga.minor}.${oneFpga.patch}`;

  await osd.textMenu({
    title: "About",
    back: true,
    items: [
      {
        label: "1FPGA Version",
        marker: version,
        select: () => {},
      },
      "-",
      {
        label: "< Join the 1FPGA Patreon >",
        marker: "...",
        select: () => {
          osd.qrCode(
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
