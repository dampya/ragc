// Table to map AGC values to 7-segment display values
#[allow(dead_code)]
const SEVEN_SEG_TABLE: [u8; 11] = [
    0x3F, 0x06, 0x5B, 0x4F, 0x66, 0x6D, 0x7D, 0x07, 0x7F, 0x6F, 0x00,
];

// Function to get the 7-segment encoding for a given AGC value
#[allow(dead_code)]
pub fn get_7seg(agc_val: u8) -> u8 {
    match agc_val {
        0 => SEVEN_SEG_TABLE[10],
        21 => SEVEN_SEG_TABLE[0],
        3 => SEVEN_SEG_TABLE[1],
        25 => SEVEN_SEG_TABLE[2],
        27 => SEVEN_SEG_TABLE[3],
        15 => SEVEN_SEG_TABLE[4],
        30 => SEVEN_SEG_TABLE[5],
        28 => SEVEN_SEG_TABLE[6],
        19 => SEVEN_SEG_TABLE[7],
        29 => SEVEN_SEG_TABLE[8],
        31 => SEVEN_SEG_TABLE[9],
        _ => SEVEN_SEG_TABLE[10], // Default case, returns the 'blank' display
    }
}

// Function to combine two 7-segment values (c and d) into a 16-bit value
#[allow(dead_code)]
pub fn get_7seg_value(c: u8, d: u8) -> u16 {
    let mut res: u16 = get_7seg(c) as u16; // Get 7-segment value for c
    res = res << 8 | get_7seg(d) as u16; // Combine with 7-segment value for d
    res
}
