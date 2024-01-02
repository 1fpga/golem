use crate::config_string::Config;
use cyclone_v::memory::{DevMemMemoryMapper, MemoryMapper};
use std::io::{Read, Write};

const DEFAULT_MISTER_SAVESTATE_SLOTS: u32 = 4;

pub struct SaveStateManager<M: MemoryMapper> {
    memory: M,
    nb_slots: u32,
    slots: Vec<SaveState>,
}

impl SaveStateManager<DevMemMemoryMapper> {
    pub fn from_config_string(config: &Config) -> Option<Self> {
        let (base, size) = config.settings().save_state?;
        let nb_slots = DEFAULT_MISTER_SAVESTATE_SLOTS;

        // The memory setup is:
        //   0x00: u32 change detector.     A value that changes when the savestate changes.
        //   0x04: u32 size                 Save of the savestate, in bytes.
        //   0x08..0x08+size                The savestate data.

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
            memory,
            slots,
        })
    }
}

impl<M: MemoryMapper> SaveStateManager<M> {
    pub fn iter(&self) -> impl Iterator<Item = &SaveState> {
        self.slots.iter()
    }

    // pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SaveState<'_>> + '_ {
    //     // self.slots.iter_mut()
    //     todo!()
    // }

    pub fn nb_slots(&self) -> usize {
        self.nb_slots as usize
    }
}

pub struct SaveState {
    change_detector: *mut u32,
    size: *mut u32,
    data: *mut u8,
}

impl SaveState {
    fn from_base(memory: &mut impl MemoryMapper, offset: usize) -> Self {
        let change_detector = unsafe { memory.as_mut_ptr::<u8>().add(offset) as _ };
        let size: *mut u32 = unsafe { memory.as_mut_ptr::<u8>().add(offset + 4) as _ };
        let data = unsafe { memory.as_mut_ptr::<u8>().add(offset + 8) };

        Self {
            change_detector,
            size,
            data,
        }
    }
}

impl SaveState {
    pub fn size(&self) -> usize {
        unsafe { *self.size as usize }
    }

    pub fn change_detector(&self) -> u32 {
        unsafe { *self.change_detector }
    }

    pub(crate) fn reset_change_detector(&mut self) {
        unsafe { *self.change_detector = 0xFFFFFFFF }
    }

    pub fn data(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data, self.size()) }
    }

    fn data_mut(&self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.data, self.size()) }
    }

    pub fn save(&mut self, mut writer: impl Write) -> Result<(), String> {
        let sz = self.size();
        writer
            .write_all(&self.change_detector().to_be_bytes())
            .map_err(|e| e.to_string())?;
        writer
            .write_all(&sz.to_be_bytes())
            .map_err(|e| e.to_string())?;
        writer
            .write_all(&self.data()[..(sz as usize)])
            .map_err(|e| e.to_string())?;
        self.reset_change_detector();
        Ok(())
    }

    pub fn load(&mut self, mut reader: impl Read) -> Result<(), String> {
        let mut change_detector = [0u8; 4];
        let mut sz = [0u8; 4];
        reader
            .read_exact(&mut change_detector)
            .map_err(|e| e.to_string())?;
        reader.read_exact(&mut sz).map_err(|e| e.to_string())?;
        let sz = u32::from_be_bytes(sz);
        if sz >= self.data().len() as u32 {
            return Err("Save state too large".to_string());
        }

        unsafe {
            *self.change_detector = u32::from_be_bytes(change_detector);
            *self.size = sz;
        }

        reader
            .read_exact(&mut self.data_mut()[..(self.size() as usize)])
            .map_err(|e| e.to_string())?;
        self.reset_change_detector();
        Ok(())
    }
}
