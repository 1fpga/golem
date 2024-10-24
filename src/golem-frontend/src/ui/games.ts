import * as ui from "@:golem/ui";
import { Games, GameSortOrder } from "$/services/database/games";

const PAGE_SIZE = 100;

export interface PickGameOptions {
  title?: string;
  sort?: GameSortOrder;
  includeUnplayed?: boolean;
  details?: boolean;
  system?: string;
}

/**
 * Pick a game from the list of games available on the platform.
 * @param options Options for the game selection.
 * @returns The selected game, or `null` if no game was selected (e.g. "Back").
 */
export async function pickGame(
  options: PickGameOptions = {},
): Promise<Games | null> {
  const sortOptions = {
    "Name (A-Z)": GameSortOrder.NameAsc,
    "Name (Z-A)": GameSortOrder.NameDesc,
    "System (A-Z)": GameSortOrder.SystemAsc,
    "Last Played": GameSortOrder.LastPlayed,
    Favorites: GameSortOrder.Favorites,
  };
  let currentSort = Object.keys(sortOptions).indexOf(
    options.sort ?? GameSortOrder.NameAsc,
  );
  let includeUnplayed = options.includeUnplayed ?? true;
  let index = 0;

  async function buildItems(): Promise<ui.TextMenuItem<Games | string>[]> {
    const { total, games } = await Games.list(
      {
        sort: Object.values(sortOptions)[currentSort],
        system: options.system,
        includeUnplayed,
      },
      {
        limit: PAGE_SIZE,
      },
    );

    const gameSet: Map<String, Games[]> = games.reduce((acc, game) => {
      if (!acc.has(game.name)) {
        acc.set(game.name, []);
      }
      acc.get(game.name).push(game);
      return acc;
    }, new Map());

    const gameSetItems = [...gameSet.entries()].map(([name, gameArray]) => ({
      label: "" + name,
      select: async () => {
        return gameArray[0];
      },
      details: async () => {
        const result = await show_details_menu("" + name, gameArray);
        if (result) {
          return result;
        }
      },
      marker: gameArray[0]?.systemName ?? "",
    }));

    return [
      ...gameSetItems,
      ...(total > PAGE_SIZE
        ? [
            {
              label: "Next page...",
              select: async () => {
                index += PAGE_SIZE;
                return "next";
              },
            },
          ]
        : []),
    ];
  }

  let selected: string | Games = "next";

  while (selected === "next") {
    const items = await buildItems();
    selected = await ui.textMenu<string | Games>({
      title: options.title ?? "",
      back: "back",
      sort_label: Object.keys(sortOptions)[currentSort],
      sort: async () => {
        currentSort = (currentSort + 1) % Object.keys(sortOptions).length;
        index = 0;

        return {
          sort_label: Object.keys(sortOptions)[currentSort],
          items: await buildItems(),
        };
      },
      items,
    });
  }
  return typeof selected == "string" ? null : selected;
}

async function show_details_menu(
  name: string,
  gameArray: Games[],
): Promise<Games | null> {
  const result = await ui.textMenu<Games | 0>({
    title: name,
    back: 0,
    items: [
      {
        label: "Favorite",
        marker: gameArray[0].favorite ? "Yes" : "No",
        select: async (item) => {
          await gameArray[0].setFavorite(!gameArray[0].favorite);
          item.marker = gameArray[0].favorite ? "Yes" : "No";
        },
      },
      ...(gameArray.length > 1
        ? [
            "-",
            "Multiple versions available:",
            ...gameArray.map((game) => ({
              label: game.romPath ?? "-",
              select: async () => {
                return game;
              },
            })),
          ]
        : [
            {
              label: "Select",
              select: async () => {
                return gameArray[0];
              },
            },
          ]),
    ],
  });

  return result === 0 ? null : result;
}

export async function gamesMenu() {
  while (true) {
    const game = await pickGame({ title: "Game Library" });
    if (game) {
      await game.launch();
    } else {
      return;
    }
  }
}
