use crate::instructions::{Instructions, Mnemonic};
use log::error;

/// Decode extended-format instructions
fn decoder_extended(mut i: Instructions) -> Result<Instructions, &'static str> {
    let opbits = i.get_opcode(); // Extract the opcode bits

    match opbits {
        // Opcode 0 is a special case with 3-bit extrabits
        0 => {
            // Extract extrabits from bits 9 to 11 (0x0E00)
            let exb: u8 = ((i.data & 0x0E00) >> 9) as u8;
            i.extrabits = Some(exb);

            // Match specific extrabit values to extended opcodes
            match i.extrabits {
                Some(0) => i.mnem = Mnemonic::READ,
                Some(1) => {
                    i.mnem = Mnemonic::WRITE;
                    i.mct = 2; // This instruction takes two memory cycles
                }
                Some(2) => i.mnem = Mnemonic::RAND,
                Some(3) => i.mnem = Mnemonic::WAND,
                Some(4) => i.mnem = Mnemonic::ROR,
                Some(5) => i.mnem = Mnemonic::WOR,
                Some(6) => i.mnem = Mnemonic::RXOR,
                Some(7) => i.mnem = Mnemonic::EDRUPT,
                _ => {
                    error!(
                        "Invalid Extrabits Encoding for {}: {:?}",
                        opbits, i.extrabits
                    );
                    i.extrabits = None;
                    return Err("Invalid Extrabits Encoding");
                }
            }
            return Ok(i);
        }

        // Opcodes 1, 2, and 6 use 2-bit extrabits from bits 10-11
        1 => {
            let exb: u8 = ((i.data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);
        }

        2 => {
            let exb: u8 = ((i.data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);

            match i.extrabits {
                Some(2) => i.mnem = Mnemonic::AUG,
                Some(3) => i.mnem = Mnemonic::DIM,
                _ => {
                    error!(
                        "Invalid Extrabits Encoding for {}: {:?}",
                        opbits, i.extrabits
                    );
                    i.extrabits = None;
                    return Err("Invalid Extrabits Encoding");
                }
            }
            return Ok(i);
        }

        3 => i.mnem = Mnemonic::DCA,
        4 => i.mnem = Mnemonic::DCS,
        5 => i.mnem = Mnemonic::INDEX,

        6 => {
            let exb: u8 = ((i.data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);
            // SU if extrabits are 0, otherwise BZMF
            i.mnem = if exb == 0 {
                Mnemonic::SU
            } else {
                Mnemonic::BZMF
            };
        }

        7 => i.mnem = Mnemonic::MP,

        _ => {
            error!(
                "Invalid value found. We didn't properly mask the opcode bits. {}",
                opbits
            );
            return Err("Invalid Opcode Size");
        }
    }

    Ok(i)
}

/// Decode simple-format instructions
fn decoder_simple(mut i: Instructions) -> Result<Instructions, &'static str> {
    let opbits = i.get_opcode(); // Extract the opcode bits

    match opbits {
        // Opcode 0 handles several special control instructions
        0 => {
            i.mnem = match i.data & 0xFFF {
                3 => Mnemonic::RELINT,
                4 => Mnemonic::INHINT,
                6 => Mnemonic::EXTEND,
                _ => Mnemonic::TC,
            };
        }

        1 => {
            let exb: u8 = ((i.data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);

            match i.extrabits {
                Some(0) => i.mnem = Mnemonic::CCS,
                Some(1..=3) => i.mnem = Mnemonic::TCF,
                _ => {
                    error!(
                        "Invalid Extrabits Encoding for {}: {:?}",
                        opbits, i.extrabits
                    );
                    i.extrabits = None;
                    return Err("Invalid Extrabits Encoding");
                }
            }
        }

        2 => {
            let exb: u8 = ((i.data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);

            match i.extrabits {
                Some(0) => i.mnem = Mnemonic::DAS,
                Some(1) => i.mnem = Mnemonic::LXCH,
                Some(2) => i.mnem = Mnemonic::INCR,
                Some(3) => i.mnem = Mnemonic::ADS,
                _ => {
                    error!(
                        "Invalid Extrabits Encoding for {}: {:?}",
                        opbits, i.extrabits
                    );
                    i.extrabits = None;
                    return Err("Invalid Extrabits Encoding");
                }
            }
        }

        3 => {
            i.mnem = Mnemonic::CA;
            i.mct = 2;
        }

        4 => {
            i.mnem = Mnemonic::CS;
            i.mct = 2;
        }

        5 => {
            let exb: u8 = ((i.data & 0x0C00) >> 10) as u8;
            i.extrabits = Some(exb);

            match i.extrabits {
                Some(0) => {
                    // Check for special RESUME instruction pattern
                    if i.data & 0o07777 == 0o00017 {
                        i.mnem = Mnemonic::RESUME;
                    } else {
                        i.mnem = Mnemonic::INDEX;
                    }
                }
                Some(1) => i.mnem = Mnemonic::DXCH,
                Some(2) => {
                    i.mnem = Mnemonic::TS;
                    i.mct = 2;
                }
                Some(3) => i.mnem = Mnemonic::XCH,
                _ => {
                    error!(
                        "Invalid Extrabits Encoding for {}: {:?}",
                        opbits, i.extrabits
                    );
                    i.extrabits = None;
                    return Err("Invaid Extrabits Encoding");
                }
            }
        }

        6 => {
            i.mnem = Mnemonic::AD;
            i.mct = 2;
        }

        7 => i.mnem = Mnemonic::MASK,

        _ => {
            error!(
                "Invalid value found. We didn't properly mask the opcode bits. {}",
                opbits
            );
            return Err("Invalid Opcode Size");
        }
    }

    Ok(i)
}

/// Main decoder function that selects between extended and simple decoders
pub fn decoder(pc: u16, data: u16) -> Result<Instructions, &'static str> {
    let i = Instructions {
        pc,
        data,
        mnem: Mnemonic::INVALID, // Initial placeholder
        extrabits: None,
        mct: 1, // Default memory cycle count
    };

    // Dispatch based on instruction type
    if i.is_extended() {
        decoder_extended(i)
    } else {
        decoder_simple(i)
    }
}
