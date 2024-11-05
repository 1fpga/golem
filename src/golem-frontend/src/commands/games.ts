import * as core from "@:golem/core";
import { Commands, Games, GeneralCommandImpl } from "$/services";
import { StartGameAction } from "$/actions/start_game";

interface GameDef {
  gameId: number;
}

export class StartGameCommand extends GeneralCommandImpl<GameDef> {
  key = "startSpecificGame";
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

  async execute(_: core.GolemCore, game: GameDef) {
    const g = await Games.byId(game.gameId);
    throw new StartGameAction(g);
  }
}

export class StartLastPlayedCommand extends GeneralCommandImpl {
  key = "startLastPlayed";
  label = "Start the last played game";
  category = "Core";

  async execute() {
    const maybeGame = await Games.lastPlayed();
    if (maybeGame) {
      throw new StartGameAction(maybeGame);
    }
  }
}

export async function init() {
  await Commands.register(StartGameCommand);
  await Commands.register(StartLastPlayedCommand);
}
