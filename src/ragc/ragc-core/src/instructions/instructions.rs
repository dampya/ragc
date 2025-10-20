use super::Instructions;
use crate::constants::registers::*;
use crate::cpu::Cpu;
use crate::utils;
use crate::utils::{adjust_overflow, extend_sign_bits};
use log::warn;

/// AGC arithmetic operations (ones' complement implementation)
pub trait Arithmatic {
    fn ad(&mut self, cmd: &Instructions) -> u16; // Add
    fn ads(&mut self, cmd: &Instructions) -> u16; // Add to storage
    fn mp(&mut self, cmd: &Instructions) -> u16; // Multiply
    fn su(&mut self, cmd: &Instructions) -> u16; // Subtract
    fn incr(&mut self, cmd: &Instructions) -> u16; // Increment
    fn dim(&mut self, cmd: &Instructions) -> u16; // Decrement if minus
    fn dv(&mut self, cmd: &Instructions) -> u16; // Divide
}

impl<'a> Arithmatic for Cpu<'a> {
    fn ad(&mut self, cmd: &Instructions) -> u16 {
        // Ones' complement addition with end-around carry
        let a = self.read_s16(REGISTER_ACCUMULATOR) as u16;
        let b = self.read_s16(cmd.get_address()) as u16;

        let mut c = a as u32 + b as u32;
        if c & 0xFFFF0000 != 0 {
            // Handle overflow
            c += 1; // AGC-style carry propagation
        }

        self.write_s16(REGISTER_ACCUMULATOR, (c & 0xFFFF) as u16);
        self.check_editing(cmd.get_address());
        2
    }

    fn ads(&mut self, cmd: &Instructions) -> u16 {
        // Add and store with carry handling
        let x = self.read_s16(REGISTER_ACCUMULATOR) as u32;
        let y = self.read_s16(cmd.get_address_ram());

        let mut z = x + y as u32;
        if z & 0xFFFF0000 != 0 {
            z += 1; // End-around carry
        }

        let res = (z & 0xFFFF) as u16;
        self.write_s16(REGISTER_ACCUMULATOR, res);
        self.write_s16(cmd.get_address_ram(), res);
        2
    }

    fn mp(&mut self, cmd: &Instructions) -> u16 {
        // Ones' complement multiplication
        let val1 = self.read_s15(REGISTER_ACCUMULATOR);
        let s1 = val1 & 0o40000; // Sign bit
        let mag1 = if s1 != 0 {
            (!val1) & 0o37777
        } else {
            val1 & 0o37777
        };

        let val2 = self.read_s15(cmd.get_address());
        let s2 = val2 & 0o40000;
        let mag2 = if s2 != 0 {
            (!val2) & 0o37777
        } else {
            val2 & 0o37777
        };

        let mut output = (mag1 as u32 * mag2 as u32) & 0o1777777777;
        if s2 != s1 {
            match output {
                // Handle special zero cases
                0o0000000000 | 0o1777777777 => {
                    output = if (mag1 | mag2) == 0 { 0 } else { 0o3777777777 }
                }
                _ => output = (!output) & 0o3777777777, // Invert for negative result
            }
        }
        self.write_dp(REGISTER_ACCUMULATOR, output);
        3 // Multiplication takes 3 cycles
    }

    fn incr(&mut self, cmd: &Instructions) -> u16 {
        // Circular increment for 15/16-bit registers
        let reg = cmd.get_address_ram();
        let curr = self.read(reg) as u32;

        let next = match reg {
            REGISTER_ACCUMULATOR | REGISTER_MULTIPLIER => match curr {
                // 16-bit handling
                0o077777 => curr & 0o177777, // Rollover from positive to negative
                0o177777 => 0o000001,        // Rollover from negative to positive
                _ => (curr + 1) & 0o177777,
            },
            _ => match curr {
                // 15-bit registers
                0o37777 => 0, // Positive overflow
                0o77777 => 1, // Negative overflow
                _ => (curr + 1) & 0o77777,
            },
        };

        self.write(reg, next as u16);
        2
    }

    fn su(&mut self, cmd: &Instructions) -> u16 {
        // Ones' complement subtraction (A - B = A + !B)
        let a = self.read_s16(REGISTER_ACCUMULATOR);
        let b = !self.read_s16(cmd.get_address_ram()); // Invert bits
        let mut c = a as u32 + b as u32;
        if c & 0xFFFF0000 != 0 {
            c += 1; // Carry handling
        }
        self.write_s16(REGISTER_ACCUMULATOR, (c & 0xFFFF) as u16);
        self.check_editing(cmd.get_address_ram());
        2
    }

    fn dim(&mut self, cmd: &Instructions) -> u16 {
        // Decrement if negative (AGC-specific instruction)
        let addr = cmd.get_address_ram();
        let val = self.read_s16(addr);

        match val {
            0o177777 | 0 => {} // No action on -0 or 0
            _ => {
                if val & 0o40000 != 0 {
                    // Negative value
                    self.write_s16(addr, val + 1);
                } else {
                    // Positive value
                    self.write_s16(addr, if val - 1 == 0 { 0o177777 } else { val - 1 });
                }
            }
        };
        2
    }

    fn dv(&mut self, cmd: &Instructions) -> u16 {
        // Division with null value handling (0 and -0)
        let nulls = [0o77777, 0]; // -0 and +0 representations
        let d = self.read_s15(cmd.get_address_ram());
        let num_high = self.read_s15(REGISTER_ACCUMULATOR);
        let num_low = self.read_s15(REGISTER_LINK);

        let s_div = d & 0o40000;
        let s_num = if nulls.contains(&num_high) {
            num_low & 0o40000 // Use low word's sign if high word is null
        } else {
            num_high & 0o40000
        };

        if nulls.contains(&num_high) && nulls.contains(&num_low) {
            self.write_s15(
                REGISTER_ACCUMULATOR,
                if s_num ^ s_div == 0 { 0o37777 } else { 0o40000 },
            );
        }
        6 // Division takes 6 cycles
    }
}

/// AGC control flow operations (branching/subroutines)
pub trait ControlFlow {
    fn tcf(&mut self, cmd: &Instructions) -> u16; // Unconditional jump
    fn bzf(&mut self, cmd: &Instructions) -> u16; // Branch if zero
    fn tc(&mut self, cmd: &Instructions) -> u16; // Subroutine call
}

impl<'a> ControlFlow for Cpu<'a> {
    fn bzf(&mut self, cmd: &Instructions) -> u16 {
        self.ec_flag = false; // Reset extended cycle flag

        // Check for ones' complement zero (both +0 and -0)
        let reg_a = self.read(REGISTER_ACCUMULATOR);
        match reg_a {
            0 | 0xFFFF => {
                // 0o00000 or 0o77777 in 15-bit terms
                let destination = cmd.get_data() & 0xFFF;

                // AGC memory protection: first 1KW is erasable
                if (destination & 0xC00) == 0x0 {
                    warn!("BZF jumping to non-fixed memory!");
                }

                self.write(REGISTER_COUNTER, destination);
                self.ir = self.read(destination as usize); // Pre-fetch
                1 // Cycle count for taken branch
            }
            _ => 2, // Not taken branch cycle count
        }
    }

    fn tcf(&mut self, cmd: &Instructions) -> u16 {
        // Absolute jump with no return
        let jump_target = cmd.get_data();
        self.update_pc(jump_target);
        self.ec_flag = false; // Clear extended instruction flag
        1 // Always 1 cycle
    }

    fn tc(&mut self, cmd: &Instructions) -> u16 {
        // Subroutine call (stores return address)
        let new_pc = cmd.get_data();
        let current_pc = self.read(REGISTER_COUNTER);

        self.update_pc(new_pc);
        self.write(REGISTER_RETURN, current_pc); // Store return
        self.ec_flag = false; // Reset instruction flag

        1 // Cycle count
    }
}

pub trait Interrupt {
    fn inhint(&mut self, cmd: &Instructions) -> u16; // Inhibit interrupts
    fn relint(&mut self, cmd: &Instructions) -> u16; // Release interrupts
    fn edrupt(&mut self, cmd: &Instructions) -> u16; // Emergency detected RUPT
    fn resume(&mut self, cmd: &Instructions) -> u16; // Return from interrupt
}

impl<'a> Interrupt for Cpu<'a> {
    fn inhint(&mut self, _cmd: &Instructions) -> u16 {
        self.gint = false; // Disable general interrupts
        1 // 1 MCT (machine cycle time)
    }

    fn relint(&mut self, _cmd: &Instructions) -> u16 {
        self.gint = true; // Re-enable interrupt processing
        1
    }

    fn edrupt(&mut self, _cmd: &Instructions) -> u16 {
        self.gint = false; // Disable during emergency
        3 // Takes longer due to priority handling
    }

    fn resume(&mut self, _cmd: &Instructions) -> u16 {
        // Restore pre-interrupt state from backup registers
        let shadow_pc = self.read(REGISTER_COUNTER_BACKUP) - 1; // Adjust for return
        self.write(REGISTER_COUNTER, shadow_pc);
        self.ir = self.read(REGISTER_INSTRUCTION); // Restore instruction
        self.idx_val = 0; // Clear index value

        // Reset interrupt flags
        self.gint = true; // Re-enable interrupts
        self.is_irupt = false; // Clear interrupt state

        2 // Resume takes 2 cycles
    }
}

#[cfg(test)]
mod interrupt_tests {
    #[test]
    fn test_interrupt_stubs() {
        // TODO: Implement actual interrupt tests
    }
}

pub trait Io {
    // I/O channel operations (AGC Block II architecture)
    fn ror(&mut self, cmd: &Instructions) -> u16; // Read OR
    fn rand(&mut self, cmd: &Instructions) -> u16; // Read AND
    fn wor(&mut self, cmd: &Instructions) -> u16; // Write OR
    fn wand(&mut self, cmd: &Instructions) -> u16; // Write AND
    fn read_instr(&mut self, cmd: &Instructions) -> u16; // Basic read
    fn write_instr(&mut self, cmd: &Instructions) -> u16; // Basic write
    fn rxor(&mut self, cmd: &Instructions) -> u16; // Read XOR
}

impl<'a> Io for Cpu<'a> {
    fn ror(&mut self, cmd: &Instructions) -> u16 {
        let port = cmd.get_data() & 0x1FF; // 9-bit I/O channel address
        let port_value = self.read_io(port as usize);

        match port {
            2 => {
                // Special handling for 16-bit accumulator
                let result = self.read_s16(REGISTER_ACCUMULATOR) | port_value;
                self.write_s16(REGISTER_ACCUMULATOR, result);
            }
            _ => {
                // Standard 15-bit I/O channels
                let masked_result = self.read_s15(REGISTER_ACCUMULATOR) | (port_value & 0x7FFF);
                self.write_s15(REGISTER_ACCUMULATOR, masked_result & 0x7FFF);
            }
        };
        2 // Fixed 2-cycle timing for I/O ops
    }

    fn rand(&mut self, cmd: &Instructions) -> u16 {
        let port = cmd.get_data() & 0x1FF;
        let port_value = self.read_io(port as usize);

        match port {
            2 => {
                let result = self.read_s16(REGISTER_ACCUMULATOR) & port_value;
                self.write_s16(REGISTER_ACCUMULATOR, result);
            }
            _ => {
                let masked_result = self.read_s15(REGISTER_ACCUMULATOR) & (port_value & 0x7FFF);
                self.write_s15(REGISTER_ACCUMULATOR, masked_result & 0x7FFF);
            }
        };
        2
    }

    fn rxor(&mut self, cmd: &Instructions) -> u16 {
        let port = cmd.get_data() & 0x1FF;
        let port_value = self.read_io(port as usize);

        match port {
            2 => {
                let result = self.read_s16(REGISTER_ACCUMULATOR) ^ port_value;
                self.write_s16(REGISTER_ACCUMULATOR, result);
            }
            _ => {
                let xor_result = self.read_s15(REGISTER_ACCUMULATOR) ^ (port_value & 0x7FFF);
                self.write_s15(REGISTER_ACCUMULATOR, xor_result & 0x7FFF);
            }
        };
        2
    }

    fn wor(&mut self, cmd: &Instructions) -> u16 {
        let port: usize = (cmd.get_data() & 0x1FF) as usize;
        let port_data = self.read_io(port);

        match port {
            2 => {
                // 16-bit read-modify-write
                let new_value = self.read_s16(REGISTER_ACCUMULATOR) | port_data;
                self.write_s16(REGISTER_ACCUMULATOR, new_value);
                self.write_io(port, new_value);
            }
            _ => {
                // 15-bit with sign handling
                let masked_value = self.read_s15(REGISTER_ACCUMULATOR) | (port_data & 0x7FFF);
                self.write_s15(REGISTER_ACCUMULATOR, masked_value);
                self.write_io(port, masked_value & 0x7FFF);
            }
        };
        2
    }

    fn wand(&mut self, cmd: &Instructions) -> u16 {
        let port: usize = (cmd.get_data() & 0x1FF) as usize;
        let port_data = self.read_io(port);

        match port {
            2 => {
                let new_value = self.read_s16(REGISTER_ACCUMULATOR) & port_data;
                self.write_s16(REGISTER_ACCUMULATOR, new_value);
                self.write_io(port, new_value);
            }
            _ => {
                let masked_value = self.read_s15(REGISTER_ACCUMULATOR) & (port_data & 0x7FFF);
                self.write_s15(REGISTER_ACCUMULATOR, masked_value);
                self.write_io(port, masked_value & 0x7FFF);
            }
        };
        2
    }

    fn read_instr(&mut self, cmd: &Instructions) -> u16 {
        let port = cmd.get_data() & 0x1FF;
        let input_data = match port {
            2 => self.read_io(port as usize), // Direct 16-bit read
            _ => extend_sign_bits(self.read_io(port as usize)), // Sign-extend 15-bit
        };
        self.write_s16(REGISTER_ACCUMULATOR, input_data);
        2
    }

    fn write_instr(&mut self, cmd: &Instructions) -> u16 {
        let port = cmd.get_data() & 0x1FF;
        let data = self.read_s16(REGISTER_ACCUMULATOR);
        match port {
            2 => {
                // Full 16-bit write
                self.write_io(port as usize, data);
            }
            _ => {
                // 15-bit write with overflow adjustment
                self.write_io(port as usize, adjust_overflow(data) & 0x7FFF);
            }
        }
        2
    }
}

/// Trait handling memory load/store and exchange operations
pub trait LoadStore {
    fn cs(&mut self, cmd: &Instructions) -> u16;
    fn ca(&mut self, cmd: &Instructions) -> u16;
    fn dcs(&mut self, cmd: &Instructions) -> u16;
    fn dca(&mut self, cmd: &Instructions) -> u16;
    fn xch(&mut self, cmd: &Instructions) -> u16;
    fn lxch(&mut self, cmd: &Instructions) -> u16;
    fn qxch(&mut self, cmd: &Instructions) -> u16;
}

impl<'a> LoadStore for Cpu<'a> {
    // Clear and Subtract - loads complement of memory into accumulator
    fn cs(&mut self, cmd: &Instructions) -> u16 {
        let location: usize = cmd.get_data() as usize;
        let mut inverted_value = self.read_s16(location);
        inverted_value = !inverted_value & 0xFFFF;
        self.write_s16(REGISTER_ACCUMULATOR, inverted_value);
        self.check_editing(cmd.get_address());
        2
    }

    // Double Clear and Subtract - handles two consecutive memory words
    fn dcs(&mut self, cmd: &Instructions) -> u16 {
        // Uses big-endian format: high word at lower address
        let base_addr = cmd.get_address() - 1;

        let negated_low = (!self.read_s16(base_addr + 1)) & 0xFFFF;
        self.write(REGISTER_LINK, negated_low);

        let negated_high = (!self.read_s16(base_addr)) & 0xFFFF;
        self.write(REGISTER_ACCUMULATOR, negated_high);

        self.check_editing(base_addr + 1);
        self.check_editing(base_addr);
        3
    }

    // Double Clear and Add - loads two memory words into A and L registers
    fn dca(&mut self, cmd: &Instructions) -> u16 {
        let base_address = cmd.get_address() - 1;

        let low_word = self.read_s16(base_address + 1);
        self.write_s16(REGISTER_LINK, low_word);

        let high_word = self.read_s16(base_address);
        self.write_s16(REGISTER_ACCUMULATOR, high_word);

        self.check_editing(base_address + 1);
        self.check_editing(base_address);
        3
    }

    // Exchange Link register with memory
    fn lxch(&mut self, cmd: &Instructions) -> u16 {
        let swap_addr = cmd.get_address_ram();

        let l_reg_value = self.read_s16(REGISTER_LINK);
        let mem_value = self.read_s16(swap_addr);

        self.write_s16(REGISTER_LINK, mem_value);
        self.write_s16(swap_addr, l_reg_value);
        2
    }

    // Clear and Add - loads memory value into accumulator
    fn ca(&mut self, cmd: &Instructions) -> u16 {
        let source: usize = cmd.get_data() as usize;
        let data = self.read_s16(source);
        self.write_s16(REGISTER_ACCUMULATOR, data);
        self.check_editing(source);
        2
    }

    // Exchange Q register (return address) with memory
    fn qxch(&mut self, cmd: &Instructions) -> u16 {
        let target_addr = cmd.get_address_ram();
        let temp_value = self.read_s16(target_addr);
        let lr_value = self.read_s16(REGISTER_RETURN);

        self.write_s16(target_addr, lr_value);
        self.write_s16(REGISTER_RETURN, temp_value);
        2
    }

    // Exchange accumulator with memory (with overflow adjustment)
    fn xch(&mut self, cmd: &Instructions) -> u16 {
        let exchange_addr = cmd.get_address_ram();
        let mem_data = self.read_s16(exchange_addr);
        let a_reg_value = self.read_s16(REGISTER_ACCUMULATOR);

        // Handle overflow correction when storing
        self.write_s16(exchange_addr, utils::adjust_overflow(a_reg_value));
        self.write_s16(REGISTER_ACCUMULATOR, mem_data);
        2
    }
}
