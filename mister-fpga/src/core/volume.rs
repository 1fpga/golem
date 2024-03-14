use crate::fpga::user_io;
use std::fmt::Debug;

pub struct Volume(u8);

impl Debug for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Volume({} ({}))", self.0, 1 << self.0)
    }
}

impl Volume {
    /// Initialize a volume from a raw value sent to the core.
    pub fn raw(volume: u8) -> Result<Self, String> {
        Ok(Self(volume.clamp(0, 16)))
    }

    /// Initialize a volume from a 0-255 "linear" scale.
    pub fn scaled(volume: u8) -> Self {
        let scale = volume.leading_zeros() as u8;
        Self(scale)
    }

    /// Get the user_io message to set the volume.
    pub(crate) fn into_user_io(self) -> user_io::SetAudioVolume {
        self.into()
    }
}

impl From<Volume> for user_io::SetAudioVolume {
    fn from(volume: Volume) -> Self {
        Self(volume.0)
    }
}

pub trait IntoVolume {
    fn into_volume(self) -> Volume;
}

impl IntoVolume for u8 {
    fn into_volume(self) -> Volume {
        Volume::scaled(self)
    }
}

impl IntoVolume for Volume {
    fn into_volume(self) -> Volume {
        self
    }
}

#[test]
fn volume() {
    assert_eq!(Volume::scaled(255).0, 0);
    assert_eq!(Volume::scaled(0).0, 8);
}
