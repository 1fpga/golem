import * as ui from "@:golem/ui";
import { BaseCommand, Commands } from "$/services";

function markerForCommand(cmd: BaseCommand<unknown>) {
  const shortcuts = cmd.shortcuts;

  return shortcuts.length === 0 ? "" : `${shortcuts.length}`;
}

export async function shortcutsMenu() {
  const commands = await Commands.list();
  const byCategory = new Map<string, BaseCommand<unknown>[]>();
  for (const c of commands) {
    const category = byCategory.get(c.category) ?? [];
    category.push(c);
    byCategory.set(c.category, category);
  }

  const items = [];
  for (const [category, commands] of byCategory.entries()) {
    items.push("-");
    items.push(category);
    items.push("-");

    for (const c of commands) {
      items.push({
        label: ` ${c.name}`,
        marker: markerForCommand(c),
        select: async () => {
          const shortcut = await ui.promptShortcut(
            "Enter a new shortcut",
            c.name,
          );
          console.log(shortcut);
          if (shortcut) {
            await c.addShortcut(shortcut, undefined);
          }
        },
      });
    }
  }
  items.splice(0, 1);

  await ui.textMenu({
    title: "Shortcuts",
    back: 0,
    items: [...items],
  });
}
