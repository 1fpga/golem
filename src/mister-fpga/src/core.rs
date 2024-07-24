pub mod buttons;
pub mod file;
pub mod volume;

pub mod video;

pub mod mister_fpga;
pub use mister_fpga::*;

pub mod menu;
pub use menu::MenuCore;

mod private {
    pub trait Sealed {}
}

/// Helper trait to allow downcasting to MisterFpgaCore or MenuCore directly.
pub trait AsMisterCore: private::Sealed {
    fn as_mister_core(&self) -> Option<&MisterFpgaCore>;
    fn as_mister_core_mut(&mut self) -> Option<&mut MisterFpgaCore>;
    fn as_menu_core(&self) -> Option<&MenuCore>;
    fn as_menu_core_mut(&mut self) -> Option<&mut MenuCore>;
}

impl<T: one_fpga::Core> private::Sealed for T {}

impl<T: one_fpga::Core> AsMisterCore for T {
    #[inline]
    fn as_mister_core(&self) -> Option<&MisterFpgaCore> {
        self.as_any().downcast_ref::<MisterFpgaCore>()
    }

    #[inline]
    fn as_mister_core_mut(&mut self) -> Option<&mut MisterFpgaCore> {
        self.as_any_mut().downcast_mut::<MisterFpgaCore>()
    }

    #[inline]
    fn as_menu_core(&self) -> Option<&MenuCore> {
        self.as_any().downcast_ref::<MenuCore>()
    }

    #[inline]
    fn as_menu_core_mut(&mut self) -> Option<&mut MenuCore> {
        self.as_any_mut().downcast_mut::<MenuCore>()
    }
}
