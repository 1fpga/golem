import { init as initAudio } from "./audio";
import { init as initBasic } from "./basic";
import { init as initGames } from "./games";

export async function init() {
  await initAudio();
  await initBasic();
  await initGames();
}
