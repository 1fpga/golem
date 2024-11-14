import * as core from "1fpga:core";
import { Commands, Games, GeneralCommandImpl } from "$/services";
import { StartGameAction } from "$/actions/start_game";

interface GameDef {
  gameId: number;
}

export class StartGameCommand extends GeneralCommandImpl<GameDef> {
  key = "startSpecificGame";
  label = "Launch a specific game";
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
    return `Launch "${g.name}"`;
  }

  async execute(_: core.OneFpgaCore, game: GameDef) {
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
