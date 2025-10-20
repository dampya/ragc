use crate::constants;
use crate::memory::MemoryType;

/// Manages AGC timing systems and interrupt flags
/// Handles three distinct timer types with different behaviors
pub struct Clocks {
    counter: u32,         // Master counter for timing reference
    pub mct_counter: u16, // Memory Cycle Time counter
    rupt_counter: u32,    // Interrupt service counter
    interrupt_flags: u8,  // Bitmask of pending interrupts

    timer1: u32, // 14-bit timer (T1)
    timer3: u16, // 15-bit timer (T3)
    timer4: u16, // 15-bit timer (T4) - generates periodic interrupt
}

/// Identifies which timer to configure
pub enum ClockType {
    TIMER1, // 14-bit overflow timer
    TIMER3, // 15-bit general purpose
    TIMER4, // 15-bit interrupt generator
}

impl Clocks {
    pub fn new() -> Self {
        Self {
            rupt_counter: 1,
            interrupt_flags: 0,
            counter: 0,
            mct_counter: 0,
            timer1: 0,
            timer3: 0,
            timer4: 0,
        }
    }

    /// Merge new interrupt flags into existing state
    pub fn update_interrupt_flags(&mut self, flags: u8) {
        self.interrupt_flags |= flags;
        if self.interrupt_flags == 0x3 {
            self.interrupt_flags = 0x0; // Clear both flags
            self.rupt_counter = 0; // Reset service counter
        }
    }

    /// Update Timer4 and check for overflow condition
    pub fn process_timer4(&mut self) -> u16 {
        self.timer4 = (self.timer4 + 1) & 0o77777; // 15-bit mask
        if self.timer4 == 0o40000 {
            // Trigger at half-range
            self.timer4 = 0;
            return 1 << constants::registers::INTERRUPT_TIMER4;
        }
        0
    }

    /// Signal external downlink interrupt (channel-specific)
    pub fn trigger_interrupt(&mut self) -> u16 {
        1 << constants::registers::INTERRUPT_DOWNLINK
    }

    /// Set timer values with hardware-appropriate masking
    pub fn set_time_value(&mut self, clock_type: ClockType, value: u16) {
        match clock_type {
            ClockType::TIMER1 => self.timer1 = value as u32, // 14-bit implicit
            ClockType::TIMER3 => self.timer3 = value & 0o77777, // 15-bit mask
            ClockType::TIMER4 => self.timer4 = value & 0o77777, // 15-bit mask
        }
    }

    pub fn get_counter_value(&self) -> u32 {
        self.counter
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.timer1 = 0;
        self.timer3 = 0;
        self.timer4 = 0;
    }
}

/// Memory-mapped interface for timer registers
impl MemoryType for Clocks {
    fn read(&self, _bank: usize, address: usize) -> u16 {
        match address {
            // Timer1 returns 14 bits (mask 0o37777 = 16,383)
            constants::timers::TIMER_1_ADDRESS => (self.timer1 & 0o37777) as u16,
            constants::timers::TIMER_3_ADDRESS => self.timer3,
            constants::timers::TIMER_4_ADDRESS => self.timer4,
            _ => 0,
        }
    }

    fn write(&mut self, _bank: usize, address: usize, value: u16) {
        match address {
            constants::timers::TIMER_1_ADDRESS => self.set_time_value(ClockType::TIMER1, value),
            constants::timers::TIMER_3_ADDRESS => self.set_time_value(ClockType::TIMER3, value),
            constants::timers::TIMER_4_ADDRESS => self.set_time_value(ClockType::TIMER4, value),
            _ => {}
        }
    }
}
