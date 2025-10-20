use super::mods::IoPeriph;
use crate::constants::ports;
use crate::utils::Option;

use log::{debug, error, warn};

/// Manages AGC I/O channel addressing and peripheral routing
pub struct IoController<'a> {
    port_map: [u16; 256],                   // Memory-mapped I/O channels (0o00-0o77)
    downlink: Option<&'a mut dyn IoPeriph>, // Telemetry interface
    display: Option<&'a mut dyn IoPeriph>,  // DSKY interface
}

impl<'a> IoController<'a> {
    /// Creates controller with connected peripherals
    /// Initializes calibration channels to max values
    pub fn new(downlink_periph: &'a mut dyn IoPeriph, display_unit: &'a mut dyn IoPeriph) -> Self {
        let mut controller = Self {
            port_map: [0; 256],
            downlink: Option::Value(downlink_periph),
            display: Option::Value(display_unit),
        };
        // Initialize calibration channels (0o30-0o33)
        controller.port_map[0o30] = 0o37777; // 14-bit max (T4 cal)
        controller.port_map[0o31] = 0o77777; // 15-bit max
        controller.port_map[0o32] = 0o77777;
        controller.port_map[0o33] = 0o77777;
        controller
    }

    /// Creates controller without attached peripherals
    pub fn empty() -> Self {
        let mut controller = Self {
            port_map: [0; 256],
            downlink: Option::Empty,
            display: Option::Empty,
        };
        controller.port_map[0o30] = 0o37777;
        controller.port_map[0o31] = 0o77777;
        controller.port_map[0o32] = 0o77777;
        controller.port_map[0o33] = 0o77777;
        controller
    }

    /// Handles read operations for special I/O channels
    pub fn read_port(&mut self, port: usize) -> u16 {
        debug!("Reading from I/O port: 0o{:o}", port);
        match port {
            // Inertial measurement unit channels
            ports::CHANNEL_LOSCALAR | ports::CHANNEL_HISCALAR => 0,

            // Reaction control system jets
            ports::CHANNEL_PYJETS | ports::CHANNEL_ROLLJETS => self.port_map[port],

            // Display unit interface
            ports::CHANNEL_DSKY => {
                warn!("Unexpected read from display unit interface");
                0
            }

            // Alarm system output
            ports::CHANNEL_DSALMOUT => self.port_map[ports::CHANNEL_DSALMOUT],

            // Custom channel filters
            ports::CHANNEL_CHAN12 => self.port_map[ports::CHANNEL_CHAN12],
            ports::CHANNEL_CHAN13 => self.port_map[ports::CHANNEL_CHAN13] & 0x47CF, // Mask gyro bits
            ports::CHANNEL_CHAN14 => self.port_map[ports::CHANNEL_CHAN14],

            // Display keyboard input
            ports::CHANNEL_MNKEYIN => match &self.display {
                Option::Value(unit) => unit.read(port),
                Option::Empty => 0o00000,
            },

            // Navigation keyboard (unimplemented)
            ports::CHANNEL_NAVKEYIN => 0,

            // Hardware status channels
            ports::CHANNEL_CHAN31 => 0o77777, // Always ready status

            // Combined display data
            ports::CHANNEL_CHAN32 => {
                let display_data = match &self.display {
                    Option::Value(unit) => unit.read(port),
                    Option::Empty => 0o77777,
                };
                display_data | (self.port_map[0o32] & 0o57777) // Merge with backup data
            }

            // Downlink status
            ports::CHANNEL_CHAN33 => 0o77777,

            // Telemetry downlink
            ports::CHANNEL_CHAN34 | ports::CHANNEL_CHAN35 => match &self.downlink {
                Option::Value(periph) => periph.read(port),
                Option::Empty => 0o77777,
            },

            // Secondary display interface
            0o163 => match &self.display {
                Option::Value(unit) => unit.read(port),
                Option::Empty => 0o77777,
            },

            _ => {
                error!("Unknown I/O port access: {:o}", port);
                self.port_map[port]
            }
        }
    }

    /// Handles write operations with peripheral routing
    pub fn write_port(&mut self, port: usize, value: u16) {
        debug!("Writing to I/O port: {:x} with value {:x}", port, value);

        // Mirror writes to attached peripherals
        if let Option::Value(unit) = &mut self.display {
            unit.write(port, value);
        }
        if let Option::Value(periph) = &mut self.downlink {
            periph.write(port, value);
        }

        match port {
            ports::CHANNEL_DSALMOUT => self.port_map[ports::CHANNEL_DSALMOUT] = value,
            ports::CHANNEL_CHAN13 => self.port_map[ports::CHANNEL_CHAN13] = value,
            ports::CHANNEL_CHAN32 => warn!("Write attempt to read-only port CHAN32"),
            _ => self.port_map[port] = value,
        }
    }

    /// Aggregates interrupt flags from all peripherals
    pub fn get_interrupt_status(&mut self) -> u16 {
        let mut interrupt_status = 0;

        if let Option::Value(unit) = &mut self.display {
            interrupt_status |= unit.is_interrupt();
        }
        if let Option::Value(periph) = &mut self.downlink {
            interrupt_status |= periph.is_interrupt();
        }

        interrupt_status
    }
}
