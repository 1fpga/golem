import { Commands, Games } from "$/services";
import { StartGameAction } from "$/actions/start_game";

interface GameDef {
  gameId: number;
}

class StartGameCommand extends CoreCommandImpl {
  key = "startSpecificGame";
  label = "Start a specific game";
  category = "Core";

  validate(v: unknown): v is GameDef {
    return (
      typeof v == "object" &&
      v !== null &&
      typeof (v as any)["gameId"] == "number"
    );
  }

  async labelOf(game: GameDef) {
    const g = await Games.byId(game.gameId);
    return `Game ${g.name}`;
  }

  async execute(core: Core, game: GameDef) {
    const g = await Games.byId(game.gameId);
    throw new StartGameAction(g);
  }
}

export async function init() {
  await Commands.register({
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

  await Commands.register<GameDef>({
    type: "core",
    key: "startSpecificGame",
    name: "Start a specific game",
    category: "Core",
    labelOf: async (game: GameDef) => {
      const g = await Games.byId(game.gameId);
      return `Game ${g.name}`;
    },
    validator: (v: unknown): v is GameDef =>
      typeof v == "object" &&
      v !== null &&
      typeof (v as any)["gameId"] == "number",
    async handler(_core, game: GameDef) {
      const g = await Games.byId(game.gameId);
      throw new StartGameAction(g);
    },
  });
}
