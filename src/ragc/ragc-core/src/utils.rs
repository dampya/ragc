// Custom Option type with variants for empty and a wrapped value
pub enum Option<T> {
    Empty,
    Value(T),
}

// Adjusts overflow bits according to AGC-specific logic
pub fn adjust_overflow(input: u16) -> u16 {
    match input & 0xC000 {
        0x8000 => input | 0xC000, // Sign-extend negative values (11xxxx...)
        0x4000 => input & 0x3FFF, // Mask overflow for positive values (01xxxx...)
        _ => input,
    }
}

// Extends the sign bit in a 15-bit value to 16-bit representation
pub fn extend_sign_bits(value: u16) -> u16 {
    if (value & 0x4000) != 0 {
        value | 0x8000 // If sign bit is set, extend it to 16th bit
    } else {
        value & 0x7FFF
    }
}

// Converts a 1's complement number to 2's complement format (used in AGC)
#[allow(dead_code)]
pub fn convert_ones_to_twos_complement(value: u16) -> u16 {
    if value & 0x4000 != 0 {
        (value.wrapping_add(1)) & 0x7FFF // Add 1 and mask to 15 bits
    } else {
        value & 0x7FFF
    }
}

// 15-bit signed addition with overflow correction for AGC representation
pub fn add_s15(op1: u16, op2: u16) -> u16 {
    let mut sum = op1 as u32 + op2 as u32;
    if (sum & 0x8000) != 0 {
        sum = sum.wrapping_add(1); // Correct overflow when sign bit is set
    }
    (sum & 0x7FFF) as u16
}

// 16-bit signed addition with overflow correction
pub fn add_s16(left: u16, right: u16) -> u16 {
    let mut total = left as u32 + right as u32;
    if (total & 0xFFFF0000) != 0 {
        total += 1; // Overflow occurred; increment total
    }
    (total & 0xFFFF) as u16
}

// Double-width addition for 29-bit AGC-style values
pub fn double_width_add(num1: u32, num2: u32) -> u32 {
    let mut result = num1.wrapping_add(num2);
    if (result & 0xE0000000) != 0 {
        result = result.wrapping_add(1); // Correct overflow for 29-bit format
    }
    result
}

// Converts a signed 16-bit CPU value into AGC-style 1's complement format
pub fn translate_to_agc_format(cpu_value: i16) -> u16 {
    if cpu_value.is_negative() {
        !(cpu_value.abs() as u16) // Return 1's complement of abs value
    } else {
        cpu_value as u16
    }
}

// Converts a 1's complement AGC-style value to a standard CPU signed value
pub fn translate_from_agc_format(agc_value: u16) -> i16 {
    if (agc_value & 0x4000) != 0 {
        -(((!agc_value) & 0x3FFF) as i16)
    } else {
        agc_value as i16
    }
}

// Converts a 29-bit AGC double word to standard 32-bit CPU signed value
pub fn convert_agc_double_to_cpu(agc_double: u32) -> i32 {
    if (agc_double & 0x20000000) != 0 {
        -(((!agc_double) & 0x1FFFFFFF) as i32) // Handle negative values
    } else {
        agc_double as i32
    }
}

// Unit tests for conversion correctness
#[cfg(test)]
mod conversion_tests {
    use super::*;

    #[test]
    fn test_pos_overflow() {
        for value in 0x4000..0x7FFF {
            let result = adjust_overflow(value);
            assert_eq!(value & 0x3FFF, result);
        }
    }

    #[test]
    fn test_neg_overflow() {
        for value in 0x8000..0xBFFF {
            let result = adjust_overflow(value);
            assert_eq!(value | 0xC000, result);
        }
    }
}
