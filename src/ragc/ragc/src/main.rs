extern crate clap;
use crossbeam_channel::bounded; // Inter-thread communication
use ctrlc; // exit using cntrl-c
use env_logger;
use log::error;

// Internal project modules
use ragc_binaries;
use ragc_core::{cpu, memory}; // Core emulation components
use ragc_peripherals;

// ROM configuration constants
pub const NUM_ROM_BANKS: usize = 36;
pub const WORDS_PER_ROM: usize = 1024;

/// Configures command-line interface using clap
fn get_cli_config<'a>() -> clap::ArgMatches<'a> {
    let description = "Apollo Guidance Computer emulator implementation in Rust";
    clap::App::new("Rust AGC Emulator (RAGC)")
        .version("0.1")
        .about(description)
        .subcommand(
            clap::SubCommand::with_name("retread50")
                .help("Execute using RETREAD50 (Apollo 11 CM pre-launch)"),
        )
        .subcommand(
            clap::SubCommand::with_name("luminary99")
                .help("Run with LUMINARY99 ROM setup (Apollo 11 LM)"),
        )
        .subcommand(
            clap::SubCommand::with_name("comanche55")
                .help("Start with COMANCHE55 ROM image (Apollo 11 CM)"),
        )
        .get_matches()
}

/// Main entry point for AGC emulator
fn main() {
    env_logger::init();

    // Set up Ctrl-C handler with channel communication
    let (signal_sender, signal_receiver) = bounded(1);
    let handler_result = ctrlc::set_handler(move || {
        if signal_sender.is_full() {
            std::process::exit(-1); // Emergency exit if channel blocked
        }
        let _send_result = signal_sender.send(()); // Send shutdown signal
    });

    if let Err(e) = handler_result {
        error!("Signal handler failed: {:?}", e);
        return;
    }

    // Parse command-line arguments
    let cli_matches = get_cli_config();

    // Load appropriate ROM image
    let rom_data = match cli_matches.subcommand_name() {
        Some("retread50") => *ragc_binaries::RETREAD50_ROPE,
        _ => {
            error!("Invalid ROM specified");
            return;
        }
        Some("luminary99") => *ragc_binaries::LUMINARY99_ROPE,
        _ => {
            error!("Invalid ROM specified");
            return;
        }
        Some("comanche55") => *ragc_binaries::COMANCHE55_ROPE,
        _ => {
            error!("Invalid ROM specified");
            return;
        }
    };

    // Initialize hardware components
    let mut queue_instance = heapless::spsc::Queue::new();
    let (rupt_line, _) = queue_instance.split(); // RUPT line communication

    let mut display_unit = ragc_peripherals::dsky::DskyDisplay::new();

    let mut rupt_handler = ragc_peripherals::downrupt::DownruptPeriph::new();

    // Configure memory map with ROM and peripherals
    let memory_map =
        memory::MemoryMap::new(&rom_data, &mut rupt_handler, &mut display_unit, rupt_line);

    // Create and initialize CPU core
    let mut agc_cpu = cpu::Cpu::new(memory_map);
    agc_cpu.reset(); // Perform AGC cold start

    // Main emulation loop
    let mut cycle_timer = std::time::Instant::now();
    loop {
        if !signal_receiver.is_empty() {
            break;
        }

        // Timing control for cycle-accurate emulation
        let elapsed_time = cycle_timer.elapsed();
        if elapsed_time.as_millis() == 0 {
            // Prevent busy-waiting at high speeds
            std::thread::sleep(std::time::Duration::from_micros(5000));
            continue;
        }

        // Calculate target cycles based on AGC clock speed (11.7µs/cycle)
        let target_cycles = (elapsed_time.as_micros() as f64 / 11.7) as i64;
        let mut executed_cycles = 0;

        // Execute instructions until catching up with real time
        while executed_cycles < target_cycles {
            executed_cycles += agc_cpu.step() as i64;
        }

        // Reset timing for next frame
        cycle_timer = std::time::Instant::now();
    }
}
