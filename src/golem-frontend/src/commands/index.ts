import { init as initBasic } from "./basic";
import { init as initGames } from "./games";

export async function init() {
  await initBasic();
  await initGames();
}
