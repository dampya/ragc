use dsky_protocol::agc::generate_dsky_packet;

use crossbeam_channel::{unbounded, Receiver, Sender};
use std::io::Write;
use std::net::TcpListener;

use ragc_core::memory::periph::IoPeriph;

pub struct DownruptPeriph {
    tx: Sender<[u8; 4]>,
    word_order: bool, // Tracks current word order for CHAN13 read behavior
}

// Thread responsible for forwarding DSKY packets over TCP to 127.0.0.1:19800
fn downrupt_thread(rx: Receiver<[u8; 4]>, addr: &str) {
    let listener = TcpListener::bind(addr).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut xa) => loop {
                let msg = match rx.recv() {
                    Ok(x) => x,
                    _ => break,
                };

                match xa.write_all(&msg) {
                    Ok(_) => {}
                    _ => break,
                }
            },
            _ => {}
        };
    }
}

impl DownruptPeriph {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();

        // Spawn thread to handle outgoing TCP communication
        std::thread::spawn(move || downrupt_thread(rx, "127.0.0.1:19800"));
        DownruptPeriph {
            tx,
            word_order: false,
        }
    }
}

impl IoPeriph for DownruptPeriph {
    fn read(&self, channel_idx: usize) -> u16 {
        match channel_idx {
            ragc_core::constants::ports::CHANNEL_CHAN13 => {
                // CHAN13 read returns a control bit based on word_order
                if self.word_order {
                    1 << 6
                } else {
                    0o00000
                }
            }
            // Return max value for these channels (typical AGC pattern)
            ragc_core::constants::ports::CHANNEL_CHAN30
            | ragc_core::constants::ports::CHANNEL_CHAN31
            | ragc_core::constants::ports::CHANNEL_CHAN32
            | ragc_core::constants::ports::CHANNEL_CHAN33
            | ragc_core::constants::ports::CHANNEL_CHAN34
            | ragc_core::constants::ports::CHANNEL_CHAN35 => 0o77777,
            _ => 0o00000,
        }
    }

    fn write(&mut self, channel_idx: usize, value: u16) {
        match channel_idx {
            ragc_core::constants::ports::CHANNEL_CHAN13 => {
                // Toggle word_order flag based on bit 6
                self.word_order = value & (1 << 6) != 0o00000;
            }
            ragc_core::constants::ports::CHANNEL_CHAN34
            | ragc_core::constants::ports::CHANNEL_CHAN35 => {
                // Generate and send DSKY packet over channel
                let packet = generate_dsky_packet(channel_idx, value);
                self.tx.send(packet).unwrap();
            }
            _ => {}
        }
    }

    fn is_interrupt(&mut self) -> u16 {
        0 // This peripheral doesn't generate interrupts
    }
}
