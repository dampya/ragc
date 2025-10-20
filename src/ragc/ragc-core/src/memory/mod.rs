mod clock;
mod edit_registers;
pub mod io;
mod memory;
mod registers;
mod rom;
mod special_registers;

pub mod mods;
pub use io::IoController;

use self::mods::IoPeriph;
use crate::constants;
use crate::constants::address_space;
use heapless::spsc::Producer;
use log::error;

/// Core memory access interface for AGC components
trait MemoryType {
    fn read(&self, bank_idx: usize, bank_offset: usize) -> u16;
    fn write(&mut self, bank_idx: usize, bank_offset: usize, value: u16);
}

/// Central memory management unit implementing AGC address space
/// Handles banking and peripheral I/O through component routing
pub struct MemoryMap<'a> {
    ram: memory::Ram,                    // Eraseable memory (core rope simulator)
    rom: rom::ReadOnlyMemory<'a>,        // Fixed memory (core rope)
    io: io::IoController<'a>,            // I/O channel manager
    edit: edit_registers::EditRegisters, // Shift/cycle registers
    special: special_registers::SpecialRegisters, // Interrupt/control registers
    timers: clock::Clocks,               // Timing systems
    regs: registers::Registers,          // CPU registers
}

impl<'a> MemoryMap<'a> {
    /// Creates blank memory map for diagnostic purposes
    pub fn new_blank(rupt_tx: Producer<u8, 8>) -> MemoryMap {
        MemoryMap {
            ram: memory::Ram::new(),
            rom: rom::ReadOnlyMemory::empty(),
            io: io::IoController::empty(),
            edit: edit_registers::EditRegisters::new(),
            special: special_registers::SpecialRegisters::new(rupt_tx),
            timers: clock::Clocks::new(),
            regs: registers::Registers::new(),
        }
    }

    /// Creates operational memory map with loaded program
    pub fn new(
        program: &'a [[u16; constants::STORAGE_SEGMENT_SIZE]; constants::STORAGE_SEGMENTS],
        downrupt: &'a mut dyn IoPeriph, // Downlink peripheral
        dsky: &'a mut dyn IoPeriph,     // Display interface
        rupt_tx: Producer<u8, 8>,       // Interrupt channel
    ) -> MemoryMap<'a> {
        MemoryMap {
            ram: memory::Ram::new(),
            rom: rom::ReadOnlyMemory::new(program),
            edit: edit_registers::EditRegisters::new(),
            io: io::IoController::new(downrupt, dsky),
            special: special_registers::SpecialRegisters::new(rupt_tx),
            timers: clock::Clocks::new(),
            regs: registers::Registers::new(),
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.ram.reset();
        self.timers.reset();
    }

    pub fn fetch_clocks(&mut self) -> &mut clock::Clocks {
        &mut self.timers
    }

    /// Handles I/O channel writes with special register routing
    pub fn write_io(&mut self, idx: usize, value: u16) {
        match idx {
            constants::ports::CHANNEL_L => {
                // Link register
                self.regs
                    .write(0, constants::registers::REGISTER_LINK, value);
            }
            constants::ports::CHANNEL_Q => {
                // Multiplier quotient
                self.regs
                    .write(0, constants::registers::REGISTER_MULTIPLIER, value);
            }
            constants::ports::CHANNEL_CHAN34 => {
                // Downlink interrupt
                self.timers.update_interrupt_flags(1);
                self.io.write_port(idx, value);
            }
            constants::ports::CHANNEL_CHAN35 => {
                // Uplink interrupt
                self.timers.update_interrupt_flags(2);
                self.io.write_port(idx, value);
            }
            _ => {
                self.io.write_port(idx, value);
            }
        };
    }

    /// Handles I/O channel reads with timer value splitting
    pub fn read_io(&mut self, idx: usize) -> u16 {
        match idx {
            constants::ports::CHANNEL_L => self.regs.read(0, constants::registers::REGISTER_LINK),
            constants::ports::CHANNEL_Q => {
                self.regs.read(0, constants::registers::REGISTER_MULTIPLIER)
            }
            constants::ports::CHANNEL_HISCALAR => {
                // Timer high bits
                let result = self.timers.get_counter_value();
                ((result >> 14) & 0o37777) as u16 // Extract bits 14-27
            }
            constants::ports::CHANNEL_LOSCALAR => {
                // Timer low bits
                let result = self.timers.get_counter_value();
                (result & 0o37777) as u16 // Extract bits 0-13
            }
            _ => self.io.read_port(idx),
        }
    }

    /// Main memory write handler with bank switching
    pub fn write(&mut self, idx: usize, val: u16) {
        match idx {
            0o00..=0o17 => {
                // CPU registers
                self.regs.write(0, idx, val);
            }
            0o20..=0o23 => {
                // Edit registers
                self.edit.write(0, idx, val);
            }
            0o24..=0o31 => {
                // Timer registers
                self.timers.write(0, idx, val);
            }
            0o32..=0o60 => {
                // Special control registers
                self.special.write(0, idx, val);
            }
            address_space::VOLATILE_START..=address_space::VOLATILE_END => {
                // RAM
                // Handle erasable bank switching (bank 3 is switchable)
                if (idx >> 8) == 3 {
                    self.ram
                        .write(self.regs.erasable_bank, (idx & 0xff) as usize, val)
                } else {
                    self.ram.write(idx >> 8, (idx & 0xff) as usize, val)
                }
            }
            address_space::PERSISTENT_START..=address_space::PERSISTENT_END => {
                // ROM
                let bank_idx = idx >> 10;
                if bank_idx == 1 {
                    // Fixed-fixed bank switching
                    self.rom
                        .write(self.regs.fixed_bank, (idx & 0x3ff) as usize, val)
                } else {
                    self.rom.write(bank_idx, (idx & 0x3ff) as usize, val)
                }
            }
            _ => {
                error!("Unimplemented Memory Map Write (Addr: 0x{:x}", idx);
            }
        }
    }

    /// Main memory read handler with bank switching
    pub fn read(&self, idx: usize) -> u16 {
        let val = match idx {
            0o00..=0o17 => self.regs.read(0, (idx & 0xff) as usize), // CPU regs
            0o20..=0o23 => self.edit.read(0, idx),                   // Edit regs
            0o24..=0o31 => self.timers.read(0, idx),                 // Timers
            0o32..=0o60 => self.special.read(0, idx),                // Control regs
            address_space::VOLATILE_START..=address_space::VOLATILE_END => {
                // RAM
                // Handle erasable bank selection
                if (idx >> 8) == 3 {
                    self.ram
                        .read(self.regs.erasable_bank, (idx & 0xff) as usize)
                } else {
                    self.ram.read(idx >> 8, (idx & 0xff) as usize)
                }
            }
            address_space::PERSISTENT_START..=address_space::PERSISTENT_END => {
                // ROM
                // Handle fixed bank selection
                if (idx >> 10) == 1 {
                    self.rom.read(self.regs.fixed_bank, (idx & 0x3ff) as usize)
                } else {
                    self.rom.read(idx >> 10, (idx & 0x3ff) as usize)
                }
            }
            _ => {
                error!("Unimplemented Memory Map Read (Addr: 0x{:x}", idx);
                0
            }
        };
        val
    }

    /// Aggregate interrupt status from I/O subsystems
    pub fn check_interrupts(&mut self) -> u16 {
        self.io.get_interrupt_status()
    }
}
