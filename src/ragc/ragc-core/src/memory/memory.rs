use crate::constants;
use crate::memory::MemoryType;

/// Implements AGC's eraseable memory (RAM) with fixed banking
/// Stores 15-bit words + parity bit for special registers
pub struct Ram {
    memory_banks: [[u16; constants::MEMORY_SEGMENT_SIZE]; constants::MEMORY_SEGMENTS],
}

impl Ram {
    pub fn new() -> Self {
        Self {
            memory_banks: [[0; constants::MEMORY_SEGMENT_SIZE]; constants::MEMORY_SEGMENTS],
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.memory_banks = [[0; constants::MEMORY_SEGMENT_SIZE]; constants::MEMORY_SEGMENTS];
    }
}

impl MemoryType for Ram {
    fn read(&self, bank_index: usize, address_offset: usize) -> u16 {
        let value = if bank_index == 0x0
            && address_offset == constants::registers::REGISTER_ACCUMULATOR
        {
            // Preserve full 16 bits for accumulator (including sign)
            self.memory_banks[bank_index][address_offset]
        } else if bank_index == 0x0 && address_offset == constants::registers::REGISTER_MULTIPLIER {
            // Preserve full 16 bits for multiplier product
            self.memory_banks[bank_index][address_offset]
        } else {
            // Normal memory locations use 15-bit values
            self.memory_banks[bank_index][address_offset] & 0x7FFF
        };
        value
    }

    fn write(&mut self, bank_index: usize, address_offset: usize, value: u16) {
        if bank_index == 0x0 && address_offset == constants::registers::REGISTER_ACCUMULATOR {
            // Store full 16-bit accumulator value
            self.memory_banks[bank_index][address_offset] = value;
        } else if bank_index == 0x0 && address_offset == constants::registers::REGISTER_MULTIPLIER {
            // Store full 16-bit multiplier value
            self.memory_banks[bank_index][address_offset] = value;
        } else {
            // Truncate to 15 bits + parity for regular storage
            let masked_value = value & 0x7FFF;
            self.memory_banks[bank_index][address_offset] = masked_value;
        }
    }
}
