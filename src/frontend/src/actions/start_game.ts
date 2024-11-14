import { Games } from "$/services/database/games";

export class StartGameAction {
  constructor(public readonly game: Games) {}
}
