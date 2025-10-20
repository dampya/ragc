use crate::constants::ports;
use crate::constants::registers::*;
use crate::decoder::decoder;
use crate::instructions::{Arithmatic, ControlFlow, Interrupt, Io, LoadStore};
use crate::instructions::{Instructions, Mnemonic};
use crate::memory::MemoryMap;
use crate::utils::{add_s15, adjust_overflow, extend_sign_bits};

/// Enum for representing the unprogrammed sequence instructions
#[allow(dead_code)]
pub enum UnprogSequence {
    PINC,
    PCDU,
    MINC,
    MCDU,
    DINC,
    SHINC,
    SHANC,
    INOTRD,
    INOTLD,
    FETCH,
    STORE,
    GOJ,
    TCSAJ,
    RUPT,
}

/// Enum for representing overflow state
#[allow(dead_code)]
pub enum Overflow {
    None,
    Positive,
    Negative,
}

/// Trait that defines the behavior for unprogrammed GOJ instruction
trait UnprogInstruction {
    fn handle_goj(&mut self) -> u16;
}

/// Struct representing the CPU and its state
#[allow(dead_code)]
pub struct Cpu<'a> {
    mem: MemoryMap<'a>,      // Memory mapping
    pub ir: u16,             // Instruction register
    pub idx_val: u16,        // Indexed value for addressing
    pub ec_flag: bool,       // Extend flag
    pub total_cycles: usize, // Total cycles executed
    mct_counter: f64,        // Master control timing counter
    timer_counter: u8,       // Timer counter

    pub gint: bool,     // Global interrupt enable
    pub is_irupt: bool, // Interrupt active status

    unprog: heapless::Deque<UnprogSequence, 8>, // Queue for unprogrammed instructions
    pub rupt: u16,                              // Interrupt request bits

    nightwatch: u16,        // Nightwatch memory counter
    nightwatch_cycles: u32, // Nightwatch cycle count

    tc_count: u32,     // TC instruction count
    non_tc_count: u32, // Non-TC instruction count

    ruptlock_count: i32, // Interrupt lock count
}

impl<'a> UnprogInstruction for Cpu<'a> {
    /// GOJ: Zero specific IO channels and reset flags
    fn handle_goj(&mut self) -> u16 {
        self.write_io(ports::CHANNEL_PYJETS, 0);
        self.write_io(ports::CHANNEL_ROLLJETS, 0);
        self.write_io(ports::CHANNEL_DSKY, 0);
        self.write_io(ports::CHANNEL_DSALMOUT, 0);
        self.write_io(ports::CHANNEL_CHAN12, 0);
        self.write_io(ports::CHANNEL_CHAN13, 0);
        self.write_io(ports::CHANNEL_CHAN14, 0);
        self.write_io(ports::CHANNEL_CHAN34, 0);
        self.write_io(ports::CHANNEL_CHAN35, 0);

        let val = self.read_io(ports::CHANNEL_CHAN33);
        self.write_io(ports::CHANNEL_CHAN33, val & 0o75777);

        self.gint = false;
        self.is_irupt = false;

        self.tc_count = 0;
        self.non_tc_count = 0;

        self.restart();

        2 // Cycle cost
    }
}

impl<'a> Cpu<'a> {
    /// Combines the IR and index for instruction calculation
    fn calculate_instr_data(&self) -> u16 {
        let mut inst_data = add_s15(self.ir, self.idx_val);
        if self.ec_flag {
            inst_data |= 0x8000;
        }
        inst_data
    }

    /// Creates a new CPU instance with default values
    pub fn new(memmap: MemoryMap) -> Cpu {
        let mut cpu = Cpu {
            mem: memmap,
            ir: 0x0,
            ec_flag: false,
            idx_val: 0x0,
            unprog: heapless::Deque::new(),

            total_cycles: 0,
            mct_counter: 0.0,
            timer_counter: 0,

            gint: false,
            is_irupt: false,
            rupt: 1 << INTERRUPT_DOWNLINK,

            nightwatch: 0,
            nightwatch_cycles: 0,
            tc_count: 0,
            non_tc_count: 0,
            ruptlock_count: 0,
        };

        cpu.reset();
        cpu
    }

    /// Reset CPU to startup state
    pub fn reset(&mut self) {
        self.update_pc(0x800);
        self.gint = false;
    }

    /// Restart CPU: similar to reset but modifies IO
    fn restart(&mut self) {
        self.update_pc(0x800);
        self.gint = false;

        let io_val = self.read_io(0o163);
        self.write_io(0o163, 0o200 | io_val);
    }

    /// Sets program counter and fetches next instruction
    pub fn update_pc(&mut self, val: u16) {
        self.write(REGISTER_COUNTER, val);
        self.ir = self.read(val as usize);
    }

    /// Fix/refresh known registers during editing
    pub fn check_editing(&mut self, k: usize) {
        match k {
            0o20 | 0o21 | 0o22 | 0o23 => {
                let val = self.read_s15(k);
                self.write_s15(k, val);
            }
            _ => {}
        }
    }

    // Memory read/write functions, including sign extension handling

    pub fn read(&mut self, idx: usize) -> u16 {
        if idx == 0o067 {
            self.nightwatch += 1;
        }
        self.mem.read(idx)
    }

    pub fn read_s16(&mut self, idx: usize) -> u16 {
        match idx {
            REGISTER_ACCUMULATOR | REGISTER_MULTIPLIER => self.read(idx),
            _ => extend_sign_bits(self.read(idx)),
        }
    }

    pub fn read_s15(&mut self, idx: usize) -> u16 {
        match idx {
            REGISTER_ACCUMULATOR | REGISTER_MULTIPLIER => adjust_overflow(self.read(idx)) & 0x7FFF,
            _ => self.read(idx) & 0x7FFF,
        }
    }

    pub fn write_s16(&mut self, idx: usize, value: u16) {
        match idx {
            REGISTER_ACCUMULATOR | REGISTER_MULTIPLIER => self.write(idx, value),
            _ => self.write(idx, adjust_overflow(value) & 0o77777),
        }
    }

    pub fn write_s15(&mut self, idx: usize, value: u16) {
        match idx {
            REGISTER_ACCUMULATOR | REGISTER_MULTIPLIER => self.write(idx, extend_sign_bits(value)),
            _ => self.write(idx, value & 0o77777),
        }
    }

    pub fn write(&mut self, idx: usize, val: u16) {
        if idx == 0o067 {
            self.nightwatch += 1;
        }
        self.mem.write(idx, val)
    }

    /// Read double precision (32-bit equivalent) value from memory
    #[allow(dead_code)]
    pub fn read_dp(&mut self, idx: usize) -> u32 {
        let upper: u32 = self.read_s15(idx) as u32;
        let lower: u32 = self.read_s15(idx + 1) as u32;

        match (upper & 0o40000) == (lower & 0o40000) {
            true => (upper << 14) | (lower & 0o37777),
            false => {
                let mut res = if lower & 0o40000 == 0o40000 {
                    let mut val: u32 = upper << 14;
                    val += lower | 0o3777740000;
                    val
                } else {
                    let mut val: u32 = (upper + 1) << 14;
                    val += lower - 1;
                    val
                };

                if res & 0o4000000000 == 0o4000000000 {
                    res += 1;
                }
                res & 0o3777777777
            }
        }
    }

    /// Write double precision (32-bit equivalent) value to memory
    pub fn write_dp(&mut self, idx: usize, val: u32) {
        let upper = ((val >> 14) & 0o77777) as u16;
        let lower = (val & 0o37777) as u16 | (upper & 0o40000);

        self.write_s15(idx, upper);
        self.write_s15(idx + 1, lower);
    }

    // IO functions
    pub fn read_io(&mut self, idx: usize) -> u16 {
        self.mem.read_io(idx)
    }

    pub fn write_io(&mut self, idx: usize, val: u16) {
        self.mem.write_io(idx, val)
    }

    // Interrupt and overflow handling

    fn is_overflow(&mut self) -> bool {
        let a = self.read(REGISTER_ACCUMULATOR);
        a & 0xC000 != 0xC000 && a & 0xC000 != 0x0000
    }

    fn interrupt_disabled(&mut self) -> bool {
        self.ec_flag || !self.gint || self.is_irupt || self.is_overflow()
    }

    fn interrupt_pending(&self) -> bool {
        self.rupt != 0
    }

    fn handle_interrupt(&mut self) {
        for i in 0..10 {
            let mask = 1 << i;
            if self.rupt & mask != 0 {
                self.gint = false;
                let val = self.read(REGISTER_COUNTER) + 1;
                self.write(REGISTER_COUNTER_BACKUP, val);
                self.write(REGISTER_INSTRUCTION, self.calculate_instr_data());
                self.idx_val = 0;

                let new_pc = 0x800 + (i * 4);
                self.update_pc(new_pc);

                self.rupt ^= mask;
                break;
            }
        }
    }

    /// Execute the instruction and return cycle count
    pub fn execute(&mut self, inst: &Instructions) -> u16 {
        match inst.mnem {
            Mnemonic::TC | Mnemonic::TCF => {
                self.non_tc_count = 0;
                self.tc_count += 1;
            }
            _ => {
                self.tc_count = 0;
                self.non_tc_count += 1;
            }
        }

        let cycles = match inst.mnem {
            Mnemonic::AD => self.ad(inst),
            Mnemonic::ADS => self.ads(inst),
            Mnemonic::BZF => self.bzf(inst),
            Mnemonic::CA => self.ca(inst),
            Mnemonic::CS => self.cs(inst),
            Mnemonic::DCA => self.dca(inst),
            Mnemonic::DCS => self.dcs(inst),
            Mnemonic::DIM => self.dim(inst),
            Mnemonic::DV => self.dv(inst),
            Mnemonic::EXTEND => {
                self.ec_flag = true;
                self.idx_val = 0x0;
                1
            }
            Mnemonic::INCR => self.incr(inst),
            Mnemonic::INHINT => self.inhint(inst),
            Mnemonic::LXCH => self.lxch(inst),
            Mnemonic::MP => self.mp(inst),
            Mnemonic::QXCH => self.qxch(inst),
            Mnemonic::RELINT => self.relint(inst),
            Mnemonic::RESUME => self.resume(inst),
            Mnemonic::ROR => self.ror(inst),
            Mnemonic::RAND => self.rand(inst),
            Mnemonic::READ => self.read_instr(inst),
            Mnemonic::RXOR => self.rxor(inst),
            Mnemonic::SU => self.su(inst),
            Mnemonic::TC => self.tc(inst),
            Mnemonic::TCF => self.tcf(inst),
            Mnemonic::WAND => self.wand(inst),
            Mnemonic::WOR => self.wor(inst),
            Mnemonic::WRITE => self.write_instr(inst),
            Mnemonic::XCH => self.xch(inst),
            _ => {
                self.ec_flag = false;
                self.idx_val = 0x0;
                0
            }
        };
        cycles
    }

    fn update_cycles(&mut self, cycles: u16) {
        self.mct_counter += cycles as f64 * 12.0;
        self.total_cycles += cycles as usize;
    }

    /// Step through unprogrammed instruction
    fn step_unprogrammed(&mut self) -> u16 {
        let instr = self.unprog.pop_front().unwrap();
        let cycles = match instr {
            UnprogSequence::GOJ
            | UnprogSequence::TCSAJ
            | UnprogSequence::STORE
            | UnprogSequence::FETCH
            | UnprogSequence::RUPT => 2,
            _ => 1,
        };

        self.update_cycles(cycles);

        match instr {
            UnprogSequence::GOJ => {
                self.handle_goj();
                return cycles;
            }
            _ => {}
        };

        if !self.interrupt_disabled() {
            self.rupt |= self.mem.check_interrupts();
            if self.interrupt_pending() {
                self.handle_interrupt();
                self.is_irupt = true;
            }
        }

        cycles
    }

    /// Step through normal instruction execution
    fn step_programmed(&mut self) -> u16 {
        if !self.interrupt_disabled() && self.interrupt_pending() {
            self.handle_interrupt();
            self.is_irupt = true;
            return 0;
        }

        let inst_data = self.calculate_instr_data();
        let addr: usize = (self.read(REGISTER_COUNTER) & 0xFFFF) as usize;
        let i = decoder(addr as u16, inst_data).unwrap();
        let next_pc = ((addr + 1) & 0xFFFF) as u16;
        self.update_pc(next_pc);

        self.idx_val = 0;

        if self.ec_flag {
            if !matches!(i.mnem, Mnemonic::INDEX) {
                self.ec_flag = false;
            }
        }

        let cycles = self.execute(&i);
        self.update_cycles(cycles);
        cycles
    }

    /// CPU execution cycle handler
    pub fn step(&mut self) -> u16 {
        if self.unprog.len() > 0 {
            self.step_unprogrammed()
        } else {
            self.step_programmed()
        }
    }
}

// Tests for the CPU
#[cfg(feature = "std")]
#[cfg(test)]
mod cpu_tests {
    use crate::cpu;
    use crate::instructions::tests::init_agc;

    #[test]
    fn cpu_test_reset_light() {
        let mut cpu = init_agc();
        let dur = std::time::Duration::from_secs(5);
        std::thread::sleep(dur);
        println!("Restarting AGC");
        cpu.restart();
        std::thread::sleep(dur);
    }
}
