#![no_std]

/// Safety-critical binary inclusion macro for AGC ROM images
/// Transmutes byte arrays to u16 arrays without copying
macro_rules! transmute {
    ($path:expr) => {{
        let binary_data = bytes!($path);
        unsafe { &core::mem::transmute(*binary_data) }
    }};
}

const NUM_ROM_BLOCKS: usize = 36;
const WORDS_PER_BLOCK: usize = 1024;

/// Preloaded Apollo Guidance Computer ROM images
/// Structure: 36 banks × 1024 15-bit words (stored in u16)
pub static RETREAD50_ROPE: &[[u16; WORDS_PER_BLOCK]; NUM_ROM_BLOCKS] =
    transmute!("../RETREAD50.bin");

pub static LUMINARY99_ROPE: &[[u16; WORDS_PER_BLOCK]; NUM_ROM_BLOCKS] =
    transmute!("../LUMINARY99.bin");

pub static COMANCHE55_ROPE: &[[u16; WORDS_PER_BLOCK]; NUM_ROM_BLOCKS] =
    transmute!("../COMANCHE55.bin");
