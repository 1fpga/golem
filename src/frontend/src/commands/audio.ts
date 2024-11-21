import * as core from "1fpga:core";
import { Commands, GeneralCommandImpl } from "$/services/database/commands";
import { UserSettings } from "$/services";

/**
 * A command to set the volume to 0.
 */
export class VolumeMuteCommand extends GeneralCommandImpl {
  key = "volumeMute";
  label = "Mute volume";
  category = "Audio";

  private settings_: undefined | UserSettings;

  async execute(core?: core.OneFpgaCore) {
    if (!core) {
      return;
    }
    if (!this.settings_) {
      this.settings_ = await UserSettings.forLoggedInUser();
    }

    const newVolume = 0;
    core.volume = newVolume;
    await this.settings_.setDefaultVolume(newVolume);
  }
}

/**
 * A command to set the volume to 100%.
 */
export class VolumeMaxCommand extends GeneralCommandImpl {
  key = "volumeMax";
  label = "Max volume (100%)";
  category = "Audio";

  private settings_: undefined | UserSettings;

  async execute(core?: core.OneFpgaCore) {
    if (!core) {
      return;
    }
    if (!this.settings_) {
      this.settings_ = await UserSettings.forLoggedInUser();
    }

    const newVolume = 1.0;
    core.volume = newVolume;
    await this.settings_.setDefaultVolume(newVolume);
  }
}

/**
 * A command to raise the volume by 10%.
 */
export class VolumeUpCommand extends GeneralCommandImpl {
  key = "volumeUp";
  label = "Volume up by 10%";
  category = "Audio";
  default = "'VolumeUp'";

  private settings_: undefined | UserSettings;

  async execute(core?: core.OneFpgaCore) {
    if (!core) {
      return;
    }
    if (!this.settings_) {
      this.settings_ = await UserSettings.forLoggedInUser();
    }

    const volume = core.volume;
    const newVolume = Math.min(1, volume + 0.1);
    core.volume = newVolume;
    await this.settings_.setDefaultVolume(newVolume);
  }
}

/**
 * A command to lower the volume by 10%.
 */
export class VolumeDownCommand extends GeneralCommandImpl {
  key = "volumeDown";
  label = "Volume down by 10%";
  category = "Audio";
  default = "'VolumeDown'";

  private settings_: undefined | UserSettings;

  async execute(core?: core.OneFpgaCore) {
    if (!core) {
      return;
    }
    if (!this.settings_) {
      this.settings_ = await UserSettings.forLoggedInUser();
    }

    const volume = core.volume;
    const newVolume = Math.max(0, volume - 0.1);
    core.volume = newVolume;
    await this.settings_.setDefaultVolume(newVolume);
  }
}

export async function init() {
  await Commands.register(VolumeMuteCommand);
  await Commands.register(VolumeMaxCommand);
  await Commands.register(VolumeUpCommand);
  await Commands.register(VolumeDownCommand);
}
