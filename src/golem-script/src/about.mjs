// The about page.
import * as ui from "golem/ui";
import {default as p} from "golem/patrons";

export function about() {
    let {patrons, tiers} = p;

    // Sort tiers by amount.
    const tiersSorted = Object.entries(tiers)
        .sort((a, b) => Number(b[0]) - Number(a[0]))
        .map(([amount, title]) => title);

    const patronsList = [];
    for (let tier of tiersSorted) {
        if (patrons[tier]) {
            patronsList.push("-");
            patronsList.push({label: tier, selectable: false});
            patronsList.push("-");
            for (let patron of patrons[tier]) {
                patronsList.push({label: patron, selectable: true});
            }
        }
    }

    while (true) {
        let [action] = ui.textMenu({
            title: "About",
            back: true,
            items: [
                "Thanks to our patrons",
                ...patronsList,
            ]
        });

        if (action === "back") {
            return;
        }
    }
}
