pub mod instructions;

// Import trait implementations for CPU instruction categories
pub use instructions::Arithmatic;
pub use instructions::ControlFlow;
pub use instructions::Interrupt;
pub use instructions::Io;
pub use instructions::LoadStore;

// Bit masks and constants for instruction decoding
const MASK: u16 = 0o7777; // 12-bit mask for basic instruction fields
const MASK_RAM: u16 = 0o1777; // 10-bit mask for RAM addresses (1024 words)
const OPCODE_MASK: u16 = 0o7; // Mask for 3-bit primary opcode
const OPCODE_OFFSET: u16 = 12; // Bit position of primary opcode
const OPCODE_EXTEND: u16 = 0o100000; // Bit pattern for extended instruction prefix

/// Enum representing AGC instruction mnemonics
/// Note: Not all instructions are implemented in this emulation
pub enum Mnemonic {
    AD,     // Add
    ADS,    // Add to Storage
    AUG,    // Augment
    BZF,    // Branch Zero to Fixed
    BZMF,   // Branch Zero Minus to Fixed
    CA,     // Clear and Add
    CS,     // Clear and Subtract
    CCS,    // Count, Compare, Skip
    DAS,    // Double Add to Storage
    DCA,    // Double Clear and Add
    DCS,    // Double Clear and Subtract
    DIM,    // Double Increment
    DV,     // Divide
    DXCH,   // Double Exchange
    EDRUPT, // Enable Interrupt
    EXTEND, // Extended instruction prefix
    INCR,   // Increment
    INDEX,  // Index
    INHINT, // Inhibit Interrupts
    LXCH,   // Exchange with L register
    MASK,   // Mask
    MP,     // Multiply
    MSU,    // Multiply and Subtract
    QXCH,   // Exchange with Q register
    RAND,   // Read and Disable
    READ,   // Read
    RELINT, // Release Interrupt
    RESUME, // Resume
    ROR,    // Rotate Right
    RXOR,   // Read and XOR
    SU,     // Subtract
    TC,     // Transfer Control
    TCF,    // Transfer Control Fixed
    TS,     // Transfer to Storage
    WAND,   // Write and AND
    WOR,    // Write or OR
    WRITE,  // Write
    XCH,    // Exchange
    INVALID,
}

/// Structure representing a decoded AGC instruction
pub struct Instructions {
    pub pc: u16,               // Program counter value for this instruction
    pub mnem: Mnemonic,        // Mnemonic representation
    pub data: u16,             // Raw instruction word
    pub extrabits: Option<u8>, // Additional bits for special instructions
    pub mct: u8,               // Memory Cycle Time (MCT) count
}

impl Instructions {
    pub fn new() -> Instructions {
        Instructions {
            pc: 0o00000,
            data: 0o00000,
            mnem: Mnemonic::INVALID,
            extrabits: None,
            mct: 1,
        }
    }

    /// Extract 3-bit primary opcode (bits 12-14)
    pub fn get_opcode(&self) -> u8 {
        ((self.data >> OPCODE_OFFSET) & OPCODE_MASK) as u8
    }

    /// Get 12-bit data field (bits 0-11)
    pub fn get_data(&self) -> u16 {
        (self.data & MASK) as u16
    }

    /// Get memory address from 12-bit field (for fixed memory)
    pub fn get_address(&self) -> usize {
        (self.data & MASK) as usize
    }

    /// Get RAM address from 10-bit field (bits 0-9)
    pub fn get_address_ram(&self) -> usize {
        (self.data & MASK_RAM) as usize
    }

    /// Check if instruction uses EXTEND prefix (bit 15 set)
    pub fn is_extended(&self) -> bool {
        (self.data & OPCODE_EXTEND) == OPCODE_EXTEND
    }
}
