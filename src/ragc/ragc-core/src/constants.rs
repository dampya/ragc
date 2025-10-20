// Size and count of volatile memory segments (e.g., RAM)
pub const MEMORY_SEGMENTS: usize = 8;
pub const MEMORY_SEGMENT_SIZE: usize = 256;

// Size and count of non-volatile storage segments (e.g., ROM)
pub const STORAGE_SEGMENTS: usize = 36;
pub const STORAGE_SEGMENT_SIZE: usize = 1024;

pub mod address_space {
    // Sizes of memory regions in words
    pub const VOLATILE_SIZE: usize = 0x400;
    pub const PERSISTENT_SIZE: usize = 0xC00;

    // Address ranges for each memory region
    pub const VOLATILE_START: usize = 0o61;
    pub const VOLATILE_END: usize = VOLATILE_SIZE - 1;
    pub const PERSISTENT_START: usize = VOLATILE_SIZE;
    pub const PERSISTENT_END: usize = PERSISTENT_START + PERSISTENT_SIZE - 1;
}

pub mod ports {
    // Channel constants (I/O port identifiers)
    pub const CHANNEL_L: usize = 0o01;
    pub const CHANNEL_Q: usize = 0o02;
    pub const CHANNEL_HISCALAR: usize = 0o03;
    pub const CHANNEL_LOSCALAR: usize = 0o04;
    pub const CHANNEL_PYJETS: usize = 0o05;
    pub const CHANNEL_ROLLJETS: usize = 0o06;
    pub const CHANNEL_DSKY: usize = 0o10;
    pub const CHANNEL_DSALMOUT: usize = 0o11;
    pub const CHANNEL_CHAN12: usize = 0o12;
    pub const CHANNEL_CHAN13: usize = 0o13;
    pub const CHANNEL_CHAN14: usize = 0o14;
    pub const CHANNEL_MNKEYIN: usize = 0o15;
    pub const CHANNEL_NAVKEYIN: usize = 0o16;

    // Extended channels
    pub const CHANNEL_CHAN30: usize = 0o30;
    pub const CHANNEL_CHAN31: usize = 0o31;
    pub const CHANNEL_CHAN32: usize = 0o32;
    pub const CHANNEL_CHAN33: usize = 0o33;
    pub const CHANNEL_CHAN34: usize = 0o34;
    pub const CHANNEL_CHAN35: usize = 0o35;
}

pub mod registers {
    // Main processor registers
    pub const REGISTER_ACCUMULATOR: usize = 0x0;
    pub const REGISTER_LINK: usize = 0x1;
    pub const REGISTER_BUFFER: usize = 0x1; // Alias
    pub const REGISTER_MULTIPLIER: usize = 0x02;
    pub const REGISTER_RETURN: usize = 0x2;
    pub const REGISTER_ERASABLE_BANK: usize = 0x3;
    pub const REGISTER_FIXED_BANK: usize = 0x4;
    pub const REGISTER_ZERO: usize = 0x05;
    pub const REGISTER_COUNTER: usize = 0x05; // Alias
    pub const REGISTER_COMBINED_BANK: usize = 0x6;
    pub const REGISTER_NULL: usize = 0x7;

    // Backup registers
    pub const REGISTER_ACCUMULATOR_BACKUP: usize = 0x8;
    pub const REGISTER_BUFFER_BACKUP: usize = 0x9;
    pub const REGISTER_RETURN_BACKUP: usize = 0xA;
    pub const REGISTER_ERASABLE_BANK_BACKUP: usize = 0xB;
    pub const REGISTER_FIXED_BANK_BACKUP: usize = 0xC;
    pub const REGISTER_COUNTER_BACKUP: usize = 0xD;
    pub const REGISTER_COMBINED_BANK_BACKUP: usize = 0xE;

    pub const REGISTER_INSTRUCTION: usize = 0xF;
    pub const REGISTER_MAX: usize = 0x10;

    // Interrupt codes
    pub const INTERRUPT_RESET: u8 = 0x0;
    pub const INTERRUPT_TIMER3: u8 = 0x3;
    pub const INTERRUPT_TIMER4: u8 = 0x4;
    pub const INTERRUPT_KEYPRESS1: u8 = 0x5;
    pub const INTERRUPT_KEYPRESS2: u8 = 0x6;
    pub const INTERRUPT_UPLINK: u8 = 0x7;
    pub const INTERRUPT_DOWNLINK: u8 = 0x8;
    pub const INTERRUPT_RADAR: u8 = 0x9;
    pub const INTERRUPT_MANUAL: u8 = 0xA;

    // Timing values in system cycles (converted from Hz)
    pub const WATCHDOG_TIMEOUT: u32 = 1920000000 / 11700;
    pub const MONITOR_CYCLES: u32 = 15000000 / 11700;
    pub const INTERRUPT_LOCKOUT: i32 = 300000000 / 11700;
}

pub mod cycle_registers {
    // Cycle-related special instructions
    pub const SPECIAL_REGISTER_CYCLE_RIGHT: usize = 0o20;
    pub const SPECIAL_REGISTER_SHIFT: usize = 0o21;
    pub const SPECIAL_REGISTER_CYCLE_LEFT: usize = 0o22;
    pub const SPECIAL_REGISTER_EDIT_OP: usize = 0o23;
}

pub mod timers {
    // Timer hardware register addresses
    pub const TIMER_2_ADDRESS: usize = 0o24;
    pub const TIMER_1_ADDRESS: usize = 0o25;
    pub const TIMER_3_ADDRESS: usize = 0o26;
    pub const TIMER_4_ADDRESS: usize = 0o27;
}

pub mod special_registers {
    // Control display and navigation sensor registers
    pub const SPECIAL_REGISTER_CONTROL_DISPLAY_X: usize = 0o32;
    pub const SPECIAL_REGISTER_CONTROL_DISPLAY_Y: usize = 0o33;
    pub const SPECIAL_REGISTER_CONTROL_DISPLAY_Z: usize = 0o34;
    pub const SPECIAL_REGISTER_OPTICAL_Y: usize = 0o35;
    pub const SPECIAL_REGISTER_OPTICAL_X: usize = 0o36;
    pub const SPECIAL_REGISTER_INERTIAL_X: usize = 0o37;
    pub const SPECIAL_REGISTER_INERTIAL_Y: usize = 0o40;
    pub const SPECIAL_REGISTER_INERTIAL_Z: usize = 0o41;
    pub const SPECIAL_REGISTER_DATA_INPUT: usize = 0o45;
    pub const SPECIAL_REGISTER_NAV_RADAR: usize = 0o46;
    pub const SPECIAL_REGISTER_GYRO_CTRL: usize = 0o47;

    // Command/control registers
    pub const SPECIAL_REGISTER_CONTROL_X_CMD: usize = 0o50;
    pub const SPECIAL_REGISTER_CONTROL_Y_CMD: usize = 0o51;
    pub const SPECIAL_REGISTER_CONTROL_Z_CMD: usize = 0o52;
    pub const SPECIAL_REGISTER_OPTICAL_Y_CMD: usize = 0o53;
    pub const SPECIAL_REGISTER_OPTICAL_X_CMD: usize = 0o54;
    pub const SPECIAL_REGISTER_THRUST: usize = 0o55;
    pub const SPECIAL_REGISTER_MAINTENANCE: usize = 0o56;
    pub const SPECIAL_REGISTER_DATA_OUTPUT: usize = 0o57;
    pub const SPECIAL_REGISTER_ALTITUDE: usize = 0o60;
}
