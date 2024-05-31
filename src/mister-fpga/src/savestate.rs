use crate::config_string::Config;
use cyclone_v::memory::{DevMemMemoryMapper, MemoryMapper};
use one_fpga::core::Error;
use std::io::{Read, Write};
use std::ptr::NonNull;
use std::slice;

const DEFAULT_MISTER_SAVESTATE_SLOTS: u32 = 4;

pub struct SaveStateManager<M: MemoryMapper> {
    /// Memory Mapper. The Manager needs to own it to avoid it being dropped
    /// prematurely.
    _memory: M,

    /// The number of savestate slots.
    nb_slots: u32,

    /// The savestate slots.
    slots: Vec<SaveState>,
}

impl SaveStateManager<DevMemMemoryMapper> {
    pub fn from_config_string(config: &Config) -> Option<Self> {
        let (base, size) = config.settings().save_state?;
        let nb_slots = DEFAULT_MISTER_SAVESTATE_SLOTS;

        // The memory setup is:
        //   0x00: u32 change detector.     A value that changes when the savestate changes.
        //   0x04: u32 size                 Size of the savestate, in 32-bits words.
        //   0x08..0x08 + (size * 4)        The savestate data.

        let mut memory =
            DevMemMemoryMapper::create(base.as_usize(), size * (nb_slots as usize)).unwrap();

        let slots = (0..nb_slots)
            .map(|i| {
                let offset = (i as usize) * size;
                SaveState::from_base(&mut memory, offset)
            })
            .collect();

        Some(Self {
            nb_slots,
            _memory: memory,
            slots,
        })
    }
}

impl<M: MemoryMapper> SaveStateManager<M> {
    #[inline]
    pub fn slots(&self) -> &[SaveState] {
        &self.slots[..(self.nb_slots as usize)]
    }

    #[inline]
    pub fn slots_mut(&mut self) -> &mut [SaveState] {
        &mut self.slots[..(self.nb_slots as usize)]
    }

    #[inline]
    pub fn nb_slots(&self) -> usize {
        self.nb_slots as usize
    }
}

#[repr(C)]
struct SaveStateInner {
    counter: u32,
    size: u32,
    data: [u8; 0],
}

impl SaveStateInner {
    fn size_adjusted(&self) -> usize {
        let size: usize =
            unsafe { core::ptr::read_volatile(core::ptr::addr_of!(self.size)) } as usize;
        (size + 2).checked_mul(4).unwrap()
    }

    fn counter(&self) -> u32 {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!(self.counter)) }
    }

    fn all(&self) -> &[u8] {
        unsafe {
            let s = std::ptr::addr_of!(*self) as *const u8;
            slice::from_raw_parts(s, self.size_adjusted())
        }
    }

    fn all_mut(&mut self) -> &mut [u8] {
        unsafe {
            let s = std::ptr::addr_of!(*self) as *mut u8;
            slice::from_raw_parts_mut(s, self.size_adjusted())
        }
    }

    fn reset(&mut self) {
        self.counter = 0xFFFFFFFF
    }
}

pub struct SaveState {
    /// The savestate data itself.
    inner: NonNull<SaveStateInner>,

    /// The last counter known, used to detect any changes to the savestate data.
    counter: u32,
}

impl one_fpga::core::SaveState for SaveState {
    fn is_dirty(&self) -> bool {
        self.is_dirty()
    }

    fn save(&mut self, writer: &mut dyn Write) -> Result<(), Error> {
        writer.write_all(self.inner().all())?;
        self.counter = self.inner().counter;
        Ok(())
    }

    fn load(&mut self, reader: &mut dyn Read) -> Result<(), Error> {
        reader.read_exact(self.inner_mut().all_mut())?;
        self.inner_mut().reset();
        self.counter = self.inner().counter;
        Ok(())
    }
}

impl SaveState {
    fn from_base(memory: &mut impl MemoryMapper, offset: usize) -> Self {
        let inner = unsafe { NonNull::new(memory.as_mut_ptr::<u8>().add(offset) as _).unwrap() };

        Self {
            inner,
            counter: unsafe { inner.as_ref() }.counter,
        }
    }

    fn inner(&self) -> &SaveStateInner {
        unsafe { self.inner.as_ref() }
    }

    fn inner_mut(&mut self) -> &mut SaveStateInner {
        unsafe { self.inner.as_mut() }
    }

    /// Whether the data has changed since it was last loaded/written.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.counter != self.inner().counter()
    }

    #[inline]
    pub fn live_counter(&self) -> u32 {
        self.inner().counter()
    }
}
