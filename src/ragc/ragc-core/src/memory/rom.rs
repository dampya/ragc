use crate::constants;
use crate::memory::MemoryType;
use crate::utils::Option;
use log::warn;

#[allow(dead_code)]
const DATA_LINE_NUM_PARTS: usize = 8;
#[allow(dead_code)]
const DATA_LINE_PART_LEN: usize = 6;

/// Struct representing read-only memory (ROM), typically used for fixed program storage
pub struct ReadOnlyMemory<'a> {
    // Optional reference to the ROM storage layout: 36 segments, each of fixed size
    memory_banks: Option<&'a [[u16; constants::STORAGE_SEGMENT_SIZE]; constants::STORAGE_SEGMENTS]>,
}

impl<'a> MemoryType for ReadOnlyMemory<'a> {
    fn read(&self, memory_bank: usize, bank_address: usize) -> u16 {
        // Bounds check for memory segment and address
        if memory_bank >= constants::STORAGE_SEGMENTS
            || bank_address >= constants::STORAGE_SEGMENT_SIZE
        {
            return 0x0;
        }

        match self.memory_banks {
            Option::Value(memory_data) => {
                // BANK_MAPPING maps logical bank numbers to physical segment indices
                const BANK_MAPPING: [usize; 36] = [
                    2, 3, 0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                    22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
                ];
                // Data is stored in big-endian;
                // shift right to drop unused parity bit and mask to 15 bits
                (u16::from_be(memory_data[BANK_MAPPING[memory_bank]][bank_address]) >> 1) & 0x7FFF
            }
            _ => 0,
        }
    }

    fn write(&mut self, memory_bank: usize, bank_address: usize, _data_value: u16) {
        // ROM is read-only; log a warning and do nothing on write
        if memory_bank >= constants::STORAGE_SEGMENTS
            || bank_address >= constants::STORAGE_SEGMENT_SIZE
        {
            return;
        }
        warn!("Write operation is not allowed on ROM");
    }
}

impl<'a> ReadOnlyMemory<'a> {
    /// Creates a new ReadOnlyMemory with a reference to actual ROM storage
    pub fn new(
        storage: &'a [[u16; constants::STORAGE_SEGMENT_SIZE]; constants::STORAGE_SEGMENTS],
    ) -> Self {
        Self {
            memory_banks: Option::Value(storage),
        }
    }

    /// Creates an empty ReadOnlyMemory instance (no data attached)
    pub fn empty() -> Self {
        Self {
            memory_banks: Option::Empty,
        }
    }
}
