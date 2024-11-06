import * as ui from "@:golem/ui";
import * as fs from "@:golem/fs";
import * as net from "@:golem/net";

async function networkDetails() {}

export async function networkSettingsMenu() {
  const interfaces = await net.interfaces();

  await ui.textMenu({
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
          ui.show("Speed Test", "Testing download speed...");
          const url = "http://cachefly.cachefly.net/10mb.test";
          try {
            const now = Date.now();
            const destination = await net.download(url);
            const elapsed = Date.now() - now;
            await ui.alert(
              "Speed Test",
              `Bandwidth: ${(10 / (elapsed / 1000)).toFixed(2)} MB/s`,
            );
            await fs.deleteFile(destination);
          } catch (e) {
            await ui.alert("Speed Test", `Download failed: ${e}`);
            return;
          }
        },
      },
    ],
  });

  return false;
}
