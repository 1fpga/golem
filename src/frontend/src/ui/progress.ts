import production from "consts:production";
import * as osd from "1fpga:osd";
import { oneLine } from "common-tags";

const PARTITION_TOO_LARGE_MSEC = 500;
const PARTITION_TOO_SMALL_MSEC = 200;

/**
 * Partition an array of items, and show progress while processing each
 * partition. This is useful when processing a large array of items, and you
 * want to show progress to the user.
 *
 * The partition size is the number of items to process in each partition,
 * and can change over time if it takes too long to process each partition.
 *
 * @param array The array to partition.
 * @param partitionSize The size of each partition.
 * @param title The title to show in the progress bar.
 * @param progressMessage A function that returns the message to show in the
 *                        progress bar, given the current index and total
 *                        number of items.
 * @param progress A function that processes each partition.
 */
export async function partitionAndProgress<T>(
  array: T[],
  partitionSize: number,
  title: string,
  progressMessage: (current: number, total: number) => string,
  progress: (partition: T[]) => Promise<void>,
): Promise<void> {
  let last = +new Date();
  const total = array.length;

  for (let i = 0; i < total; ) {
    const message = progressMessage(i, total);
    osd.show(title, message);

    const partition = array.slice(i, i + partitionSize);
    await progress(partition);

    const elapsed = +new Date() - last;
    if (!production) {
      console.log(oneLine`
        Processed ${Math.min(partitionSize, total)} items in ${elapsed}ms,
        current index: ${Math.min(i + partitionSize, total)}
      `);
    }

    i += partitionSize;
    // Adjust the partition size based on how long it took to process the
    // partition.
    if (elapsed > PARTITION_TOO_LARGE_MSEC) {
      partitionSize = Math.max(1, Math.floor((partitionSize * 2) / 3));
    } else if (elapsed < PARTITION_TOO_SMALL_MSEC) {
      partitionSize = Math.min(total, Math.floor((partitionSize * 3) / 2));
    }

    last = +new Date();
  }
}
