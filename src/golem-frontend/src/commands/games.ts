import { Commands, Games } from "$/services";
import { StartGameAction } from "$/actions/start_game";

interface GameDef {
  gameId: number;
}

export async function init() {
  await Commands.registerCommand({
    type: "general",
    key: "startLastPlayed",
    name: "Start the last played game",
    category: "Core",
    handler: async () => {
      console.log("Starting last played game.");
      const maybeGame = await Games.lastPlayed();
      if (maybeGame) {
        throw new StartGameAction(maybeGame);
      }
    },
  });

  await Commands.registerCommand({
    type: "core",
    key: "startSpecificGame",
    name: "Start a specific game",
    category: "Core",
    validator: (v: unknown): v is GameDef =>
      typeof v == "object" &&
      v !== null &&
      typeof (v as any)["gameId"] == "number",
    async handler(core, game: GameDef) {
      const g = await Games.byId(game.gameId);
      throw new StartGameAction(g);
    },
  });
}
