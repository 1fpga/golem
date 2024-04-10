// The about page.
import * as ui from "golem/ui";
import patrons from "golem/patrons";

export function about() {
    console.log(patrons);
    ui.alert("About", "Golem Script is a scripting language for the Golem emulator.");
}
