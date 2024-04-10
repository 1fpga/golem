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
        let [action, id] = ui.textMenu({
            title: "About",
            back: true,
            items: [
                {label: "< Join our Patron server... >", marker: "...", id: 0},
                "-",
                "Thanks to our patrons",
                ...patronsList,
            ]
        });

        switch (action) {
            case "select":
                if (id === 0) {
                    ui.qrCode(
                        "https://patreon.com/golem_fpga/join",
                        "Use this code to join our Patreon community",
                    );
                }
                break;
            case "back":
                return;
        }
    }
}
