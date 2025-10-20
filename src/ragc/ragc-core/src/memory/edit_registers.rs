use crate::constants::cycle_registers::*;
use crate::memory::MemoryType;
use log::error;

/// Handles AGC's special editing registers for shift/cycle operations
pub struct EditRegisters {
    cycle_right: u16, // Right cyclic shift register
    shift_reg: u16,   // Arithmetic shift register (preserves sign)
    cycle_left: u16,  // Left cyclic shift register
    edit_op: u16,     // Edit operation code register
}

impl EditRegisters {
    pub fn new() -> Self {
        Self {
            cycle_left: 0,
            cycle_right: 0,
            shift_reg: 0,
            edit_op: 0,
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.cycle_left = 0;
        self.cycle_right = 0;
        self.shift_reg = 0;
        self.edit_op = 0;
    }
}

/// Memory-mapped interface for cycle/shift registers
impl MemoryType for EditRegisters {
    fn read(&self, _bank: usize, address: usize) -> u16 {
        match address {
            SPECIAL_REGISTER_CYCLE_LEFT => self.cycle_left,
            SPECIAL_REGISTER_CYCLE_RIGHT => self.cycle_right,
            SPECIAL_REGISTER_SHIFT => self.shift_reg,
            SPECIAL_REGISTER_EDIT_OP => self.edit_op,
            _ => {
                error!("Invalid EditRegisters at: 0o{:o}", address);
                0
            }
        }
    }

    fn write(&mut self, _bank: usize, address: usize, value: u16) {
        // Truncate to 15 bits (0x7FFF = 0b0111111111111111)
        let masked_value = value & 0x7FFF;

        match address {
            SPECIAL_REGISTER_CYCLE_LEFT => {
                // Left cycle operation with sign bit preservation
                let sign_bit = masked_value & 0x4000; // Capture bit 14
                self.cycle_left = (masked_value << 1) & 0x7FFF; // Shift left
                self.cycle_left |= sign_bit >> 14; // Wrap sign bit to LSB
            }
            SPECIAL_REGISTER_CYCLE_RIGHT => {
                // Right cycle with wrap-around
                let low_bit = masked_value & 0x1; // Capture LSB
                self.cycle_right = (masked_value >> 1) | (low_bit << 14); // Move LSB to MSB
            }
            SPECIAL_REGISTER_SHIFT => {
                // Arithmetic shift right (preserve sign bit)
                let sign_bit = masked_value & 0o40000; // Capture sign bit (bit 14)
                self.shift_reg = (masked_value >> 1) | sign_bit; // Shift and maintain sign
            }
            SPECIAL_REGISTER_EDIT_OP => {
                // Extract bits 7-13 for operation code (mask 0o177 = 7 bits)
                self.edit_op = (masked_value >> 7) & 0o177;
            }
            _ => {
                error!("Invalid EditRegisters write at: 0o{:o}", address);
            }
        }
    }
}
