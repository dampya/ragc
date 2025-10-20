use crate::constants::special_registers::*;
use crate::memory::MemoryType;
use heapless::spsc::Producer;
use log::{error, warn};

#[derive(Clone)]
pub struct SpecialRegisters {
    pub control_display: (u16, u16, u16),
    pub optical_sensors: (u16, u16),
    pub inertial_platform: (u16, u16, u16),
}

impl SpecialRegisters {
    /// Initializes all special registers to 0
    pub fn new(_interrupt_tx: Producer<u8, 8>) -> Self {
        Self {
            control_display: (0, 0, 0),
            optical_sensors: (0, 0),
            inertial_platform: (0, 0, 0),
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        // No-op: structure provided for interface completeness or future use
    }
}

impl MemoryType for SpecialRegisters {
    fn read(&self, memory_bank: usize, register_address: usize) -> u16 {
        // Special registers are only valid in bank 0
        if memory_bank != 0 {
            error!("Invalid memory bank access ({})", memory_bank);
            return 0;
        }

        match register_address {
            SPECIAL_REGISTER_CONTROL_DISPLAY_X => self.control_display.0,
            SPECIAL_REGISTER_CONTROL_DISPLAY_Y => self.control_display.1,
            SPECIAL_REGISTER_CONTROL_DISPLAY_Z => self.control_display.2,
            SPECIAL_REGISTER_OPTICAL_X => self.optical_sensors.0,
            SPECIAL_REGISTER_OPTICAL_Y => self.optical_sensors.1,
            SPECIAL_REGISTER_INERTIAL_X => self.inertial_platform.0,
            SPECIAL_REGISTER_INERTIAL_Y => self.inertial_platform.1,
            SPECIAL_REGISTER_INERTIAL_Z => self.inertial_platform.2,

            // Command registers are placeholders; currently return 0
            SPECIAL_REGISTER_CONTROL_X_CMD
            | SPECIAL_REGISTER_CONTROL_Y_CMD
            | SPECIAL_REGISTER_CONTROL_Z_CMD => 0,

            // Any unrecognized address logs an error
            _ => {
                error!(
                    "Invalid special register read at address: 0o{:o}",
                    register_address
                );
                0
            }
        }
    }

    fn write(&mut self, _memory_bank: usize, register_address: usize, _data: u16) {
        match register_address {
            // Writes to read-only registers are logged with a warning
            SPECIAL_REGISTER_CONTROL_DISPLAY_X
            | SPECIAL_REGISTER_CONTROL_DISPLAY_Y
            | SPECIAL_REGISTER_CONTROL_DISPLAY_Z
            | SPECIAL_REGISTER_OPTICAL_X
            | SPECIAL_REGISTER_OPTICAL_Y
            | SPECIAL_REGISTER_INERTIAL_X
            | SPECIAL_REGISTER_INERTIAL_Y
            | SPECIAL_REGISTER_INERTIAL_Z => {
                warn!("Write attempt to read-only: 0o{:o}", register_address);
            }

            // Invalid or unimplemented writes trigger an error
            _ => {
                error!("Unsupported special write: 0o{:o}", register_address);
            }
        }
    }
}
