import * as ui from "golem/ui";
import * as video from "golem/video";

export function video_menu() {
    let [action, id] = ui.textMenu({
        title: "Video",
        back: true,
        items: [
            {label: "1080p", id: "V1920x1080r60"},
            {label: "720p", id: "V1280x720r60"},
        ],
    });

    switch (action) {
        case "select":
            video.setMode(id);
            break;
        case "back":
            break;
    }

    // storage.set("video", "1080p");
}

