import * as ui from "1fpga:ui";
import { Command, Commands } from "$/services";

async function shortcutCommandMenu(c: Command) {
  let done = false;
  let highlighted: number | undefined;
  while (!done) {
    // Group by same labels.
    let maps = new Map<string, { meta: unknown; shortcuts: string[] }>();
    for (const { shortcut, meta } of c.shortcutsWithMeta) {
      const label = await c.labelOf(meta);
      if (maps.has(label)) {
        maps.get(label)!.shortcuts.push(shortcut);
      } else {
        maps.set(label, { meta, shortcuts: [shortcut] });
      }
    }

    if (maps.size === 0) {
      const shortcut = await ui.promptShortcut(
        "Enter a new shortcut",
        await c.labelOf(undefined),
      );
      if (shortcut) {
        await c.addShortcut(shortcut, undefined);
        continue;
      } else {
        return;
      }
    }

    const alone = maps.size === 1;
    const ident = alone ? "" : "  ";

    const items: (string | ui.TextMenuItem<boolean>)[] = [];
    for (const [label, { meta, shortcuts }] of maps.entries()) {
      if (items.length > 0) {
        items.push("-");
      }
      if (!alone) {
        items.push(`${label}`);
      }
      items.push({
        label: `${ident}Add a new shortcut...`,
        select: async (_, i) => {
          highlighted = undefined;
          const shortcut = await ui.promptShortcut(
            "Enter a new shortcut",
            await c.labelOf(meta),
          );

          if (shortcut) {
            await c.addShortcut(shortcut, meta);
            highlighted = i;
            return false;
          }
        },
      });
      for (const s of shortcuts) {
        items.push({
          label: `${ident}Delete ${s}...`,
          select: async (_, i) => {
            const confirm = await ui.alert({
              title: `Deleting shortcut`,
              message: `Are you sure you want to delete this shortcut?\n${s}`,
              choices: ["Cancel", "Delete shortcut"],
            });
            if (confirm === 1) {
              await c.deleteShortcut(s);
              highlighted = i;
              return c.shortcuts.length == 0;
            }
          },
        });
      }
    }

    done = await ui.textMenu({
      title: c.label,
      back: true,
      highlighted,
      items,
    });
  }
}

export async function shortcutsMenu() {
  const commands = await Commands.list();
  const byCategory = new Map<string, Command[]>();
  for (const c of commands) {
    const category = byCategory.get(c.category) ?? [];
    category.push(c);
    byCategory.set(c.category, category);
  }

  const items: (ui.TextMenuItem<number> | string)[] = [];
  for (const [category, commands] of byCategory.entries()) {
    items.push("-");
    items.push(category);
    items.push("-");

    for (const c of commands) {
      items.push({
        label: `${c.label}...`,
        marker: `${c.shortcuts.length === 0 ? "" : c.shortcuts.length}`,
        select: async (item) => {
          await shortcutCommandMenu(c);
          item.marker = `${c.shortcuts.length === 0 ? "" : c.shortcuts.length}`;
        },
      });
    }
  }
  items.splice(0, 1);

  await ui.textMenu({
    title: "Shortcuts",
    back: 0,
    items,
  });
}
