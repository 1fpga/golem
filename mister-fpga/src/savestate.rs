use crate::config_string::Config;
use cyclone_v::memory::{DevMemMemoryMapper, MemoryMapper};
use std::io::{Read, Write};

const DEFAULT_MISTER_SAVESTATE_SLOTS: u32 = 4;

pub struct SaveStateManager<M: MemoryMapper> {
    memory: M,
    nb_slots: u32,
    slots: Vec<SaveState<'static>>,
}

impl SaveStateManager<DevMemMemoryMapper> {
    pub fn from_config_string(config: &Config) -> Option<Self> {
        let (base, size) = config.settings().save_state?;
        let nb_slots = DEFAULT_MISTER_SAVESTATE_SLOTS;

        // The memory setup is:
        //   0x00: u32 change detector.     A value that changes when the savestate changes.
        //   0x04: u32 size                 Save of the savestate, in bytes.
        //   0x08..0x08+size                The savestate data.

        let mut memory = DevMemMemoryMapper::create(base.as_usize(), size * 4).unwrap();
        let slots = (0..nb_slots)
            .map(|i| {
                let base = (i as usize) * size;
                let change_detector: &mut u32 =
                    &mut unsafe { *memory.as_mut_ptr::<u8>().add(base) as _ };
                let size = &mut unsafe { *memory.as_mut_ptr::<u8>().add(base + 4) as _ };
                let data = memory.as_mut_range((base + 8)..*size as usize);
                SaveState {
                    change_detector,
                    size,
                    data,
                }
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
    pub fn nb_slots(&self) -> u32 {
        self.nb_slots
    }

    pub fn save_state_mut(&mut self, slot: u32) -> Option<&mut SaveState<'_>> {
        self.slots.get_mut(slot as usize)
    }
}

pub struct SaveState<'a> {
    change_detector: &'a mut u32,
    size: &'a mut u32,
    data: &'a mut [u8],
}

impl<'a> SaveState<'a> {
    pub fn save(&mut self, mut writer: impl Write) -> Result<(), String> {
        let sz = *self.size;
        writer
            .write_all(&self.change_detector.to_be_bytes())
            .map_err(|e| e.to_string())?;
        writer
            .write_all(&self.size.to_be_bytes())
            .map_err(|e| e.to_string())?;
        writer
            .write_all(&(&self.data)[..(sz as usize)])
            .map_err(|e| e.to_string())?;
        *self.change_detector = 0xFFFFFFFF;
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
        if sz >= self.data.len() as u32 {
            return Err("Save state too large".to_string());
        }

        *self.change_detector = u32::from_be_bytes(change_detector);
        *self.size = sz;

        reader
            .read_exact(self.data[..self.size])
            .map_err(|e| e.to_string())?;
        *self.size = sz as u32;
        *self.change_detector = 0xFFFFFFFF;
        Ok(())
    }
}
