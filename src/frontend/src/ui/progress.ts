import production from "consts:production";
import * as ui from "1fpga:ui";
import { oneLine } from "common-tags";

export async function partitionAndProgress<T>(
  array: T[],
  partitionSize: number,
  title: string,
  progressMessage: (current: number, total: number) => string,
  progress: (partition: T[]) => Promise<void>,
): Promise<void> {
  let last = +new Date();
  const total = array.length;

  for (let i = 0; i < total; i += partitionSize) {
    const message = progressMessage(i, total);
    ui.show(title, message);

    const partition = array.slice(i, i + partitionSize);
    await progress(partition);

    if (!production) {
      console.log(oneLine`
        Processed ${Math.min(partitionSize, total)} items in ${+new Date() - last}ms,
        current index: ${Math.min(i + partitionSize, total)}
      `);
    }
    last = +new Date();
  }
}
