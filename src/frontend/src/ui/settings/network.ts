import * as osd from "1fpga:osd";
import * as fs from "1fpga:fs";
import * as net from "1fpga:net";

async function networkDetails() {}

export async function networkSettingsMenu() {
  const interfaces = await net.interfaces();

  await osd.textMenu({
    title: "Network Settings",
    back: false,
    items: [
      ...interfaces.flatMap((iface) => [
        {
          label: iface.name,
          marker: iface.status,
          select: () => {},
        },
        {
          label: "  Address:",
          marker: iface.address ?? "-",
        },
      ]),
      "-",
      {
        label: "Speed Test...",
        select: async () => {
          osd.show("Speed Test", "Testing download speed...");
          const url = "http://cachefly.cachefly.net/10mb.test";
          try {
            const now = Date.now();
            const destination = await net.download(url);
            const elapsed = Date.now() - now;
            await osd.alert(
              "Speed Test",
              `Bandwidth: ${(10 / (elapsed / 1000)).toFixed(2)} MB/s`,
            );
            await fs.deleteFile(destination);
          } catch (e) {
            await osd.alert("Speed Test", `Download failed: ${e}`);
            return;
          }
        },
      },
    ],
  });

  return false;
}
