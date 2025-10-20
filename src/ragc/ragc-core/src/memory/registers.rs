use crate::constants;
use crate::memory::MemoryType;

/// Struct representing AGC (Apollo Guidance Computer) registers
pub struct Registers {
    registers: [u16; 32],     // Array of 16-bit general-purpose registers
    pub fixed_bank: usize,    // Currently selected fixed memory bank
    pub erasable_bank: usize, // Currently selected erasable memory bank
}

impl Registers {
    /// Constructs a new AgcRegs instance with all registers and banks initialized to zero
    pub fn new() -> Self {
        Self {
            registers: [0; 32],
            fixed_bank: 0,
            erasable_bank: 0,
        }
    }
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.registers = [0; 32];
        self.fixed_bank = 0;
        self.erasable_bank = 0;
    }

    /// Updates the special bank-selection registers with the current fixed and erasable bank values
    fn refresh_bank_registers(&mut self) {
        let erasable_value = ((self.erasable_bank & 0x7) << 8) as u16; // Only lower 3 bits are used, shifted to bits 8–10
        let fixed_value = ((self.fixed_bank & 0x1F) << 10) as u16; // Only lower 5 bits are used, shifted to bits 10–14

        // Combined value includes fixed bank bits only
        let combined_bank_value = (erasable_value >> 8) | fixed_value;

        self.registers[constants::registers::REGISTER_ERASABLE_BANK] = erasable_value;
        self.registers[constants::registers::REGISTER_FIXED_BANK] = fixed_value;
        self.registers[constants::registers::REGISTER_COMBINED_BANK] = combined_bank_value;
    }
}

impl MemoryType for Registers {
    /// Reads a value from a register with appropriate masking for specific registers
    fn read(&self, _bank_index: usize, address_offset: usize) -> u16 {
        match address_offset {
            // ACCUMULATOR and MULTIPLIER are returned directly
            constants::registers::REGISTER_ACCUMULATOR
            | constants::registers::REGISTER_MULTIPLIER => self.registers[address_offset],

            // REGISTER_ZERO is always masked to 12 bits
            constants::registers::REGISTER_ZERO => self.registers[address_offset] & 0o7777,

            constants::registers::REGISTER_NULL => 0o00000,

            _ => self.registers[address_offset] & 0o77777,
        }
    }

    /// Writes a value to a register, with special handling for bank-selection registers
    fn write(&mut self, _bank_index: usize, address_offset: usize, new_value: u16) {
        match address_offset {
            // Combined bank register: extract and update both erasable and fixed banks
            constants::registers::REGISTER_COMBINED_BANK => {
                self.erasable_bank = (new_value & 0x7) as usize; // Lower 3 bits
                self.fixed_bank = ((new_value & 0x7C00) >> 10) as usize; // Bits 10–14
                self.refresh_bank_registers();
                return;
            }

            // Fixed bank register: update and refresh only fixed bank
            constants::registers::REGISTER_FIXED_BANK => {
                self.fixed_bank = ((new_value & 0x7C00) >> 10) as usize;
                self.refresh_bank_registers();
                return;
            }

            // Erasable bank register: update and refresh only erasable bank
            constants::registers::REGISTER_ERASABLE_BANK => {
                self.erasable_bank = ((new_value & 0x0700) >> 8) as usize;
                self.refresh_bank_registers();
                return;
            }

            constants::registers::REGISTER_ZERO => {
                self.registers[address_offset] = new_value & 0o7777;
            }

            constants::registers::REGISTER_NULL => {
                return;
            }
            _ => {
                self.registers[address_offset] = new_value & 0o77777;
            }
        }
        self.registers[address_offset] = new_value;
    }
}
