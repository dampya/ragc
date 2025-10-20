// Represents a parsed DSKY (Apollo Guidance Computer) telemetry packet
pub struct Packet {
    _hw_packet: bool, // Reserved for future hardware packet
    io_addr: usize,   // 11-bit IO channel address
    io_value: u16,    // 15-bit data value
}

impl Packet {
    /// Constructs a Packet from raw DSKY bytes, `input` - 4-byte array from DSKY telemetry stream
    pub fn new(input: &[u8; 4]) -> Self {
        // TODO: Consider error handling instead of silent failure on invalid packets
        let parsed = parse_dsky_packet(*input);
        if let Some((addr_part, val_part)) = parsed {
            Packet {
                _hw_packet: false,
                io_addr: addr_part as usize,
                io_value: val_part,
            }
        } else {
            Packet {
                io_value: 0,
                io_addr: 0,
                _hw_packet: false,
            }
        }
    }

    /// Change the packet back to DSKY wire format
    pub fn serialize(&self) -> [u8; 4] {
        generate_dsky_packet(self.io_addr, self.io_value)
    }
}

/// Constructs DSKY packet using NASA-specified format:
/// [Header | Addr(8-11), Upper(12-14) | Middle(6-11) | Lower(0-5)]
pub fn generate_dsky_packet(addr: usize, data_val: u16) -> [u8; 4] {
    let header = (0x0 | ((addr >> 3) & 0x1F)) as u8;
    let upper_bits = 0x40 | ((addr & 0x7) << 3) as u8 | ((data_val >> 12) & 0x7) as u8;
    let middle = 0x80 | ((data_val >> 6) & 0x3F) as u8;
    let lower = 0xC0 | (data_val & 0x3F) as u8;
    [header, upper_bits, middle, lower]
}

/// Extracts address and value from DSKY packet bytes
pub fn parse_dsky_packet(packet: [u8; 4]) -> Option<(u16, u16)> {
    // TODO: Remove unreachable 5-byte case (input is fixed-size array)
    let (b0, b1, b2, b3) = match packet.len() {
        4 | 5 => (packet[0], packet[1], packet[2], packet[3]),
        _ => return None,
    };

    // Validate header bits
    let valid = (b0 & 0xC0 == 0x00) &&  // Header: 00
               (b1 & 0xC0 == 0x40) &&  // Upper:  01
               (b2 & 0xC0 == 0x80) &&  // Middle: 10
               (b3 & 0xC0 == 0xC0); // Lower:  11

    if !valid {
        return None;
    }

    // Reconstruct 15-bit value
    let combined = ((b1 as u16 & 0x07) << 12) | ((b2 as u16 & 0x3F) << 6) | (b3 as u16 & 0x3F);

    // Reconstruct 11-bit address
    let addr = ((b0 as u16 & 0x3F) << 3) | ((b1 as u16 >> 3) & 0x07);

    Some((addr, combined))
}
