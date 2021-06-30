use core::panic;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread::spawn;
mod emu;
mod frontend;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // load rom
    if args.len() == 0 {
        panic!("No rom path provided");
    }

    let mut file = File::open(&args[0]).expect("Couldn't load rom");
    let mut rom = [0x0u8; 0xE00];
    file.read(&mut rom).unwrap();

    let mut emulator = emu::Chip8Emu::new(rom);

    #[cfg(feature = "tui")]
    frontend::init(emulator.get_screen_changes(), init_keyboard_proxy(emulator.new_keyboard_driver()), emulator.is_beeping());

    emulator.run();

    #[cfg(feature = "tui")]
    frontend::exit();
}

fn init_keyboard_proxy(sender: Sender<u8>) -> Sender<[u8; 4]> {
    let (tx, rx) = channel::<[u8; 4]>();
    spawn(move || {
        loop {
            match rx.recv() {
                Ok(data) => {
                    match data {
                        [0x58, 0x0, 0x0, 0x0] |
                        [0x78, 0x0, 0x0, 0x0] => {
                            sender.send(0x0).unwrap();
                            ()
                        },
                        [0x31, 0x0, 0x0, 0x0] => {
                            sender.send(0x1).unwrap();
                            ()
                        },
                        [0x32, 0x0, 0x0, 0x0] => {
                            sender.send(0x2).unwrap();
                            ()
                        },
                        [0x33, 0x0, 0x0, 0x0] => {
                            sender.send(0x3).unwrap();
                            ()
                        },
                        [0x51, 0x0, 0x0, 0x0] |
                        [0x71, 0x0, 0x0, 0x0] => {
                            sender.send(0x4).unwrap();
                            ()
                        },
                        [0x57, 0x0, 0x0, 0x0] |
                        [0x77, 0x0, 0x0, 0x0] => {
                            sender.send(0x5).unwrap();
                            ()
                        },
                        [0x45, 0x0, 0x0, 0x0] |
                        [0x65, 0x0, 0x0, 0x0] => {
                            sender.send(0x6).unwrap();
                            ()
                        },
                        [0x41, 0x0, 0x0, 0x0] | 
                        [0x61, 0x0, 0x0, 0x0] => {
                            sender.send(0x7).unwrap();
                            ()
                        },
                        [0x53, 0x0, 0x0, 0x0] |
                        [0x73, 0x0, 0x0, 0x0] => {
                            sender.send(0x8).unwrap();
                            ()
                        },
                        [0x44, 0x0, 0x0, 0x0] |
                        [0x64, 0x0, 0x0, 0x0] => {
                            sender.send(0x9).unwrap();
                            ()
                        },
                        [0x59, 0x0, 0x0, 0x0] |
                        [0x79, 0x0, 0x0, 0x0] => {
                            sender.send(0xA).unwrap();
                            ()
                        },
                        [0x43, 0x0, 0x0, 0x0] |
                        [0x63, 0x0, 0x0, 0x0] => {
                            sender.send(0xB).unwrap();
                            ()
                        },
                        [0x34, 0x0, 0x0, 0x0] => {
                            sender.send(0xC).unwrap();
                            ()
                        },
                        [0x52, 0x0, 0x0, 0x0] |
                        [0x72, 0x0, 0x0, 0x0] => {
                            sender.send(0xD).unwrap();
                            ()
                        },
                        [0x46, 0x0, 0x0, 0x0] |
                        [0x66, 0x0, 0x0, 0x0] => {
                            sender.send(0xE).unwrap();
                            ()
                        },
                        [0x56, 0x0, 0x0, 0x0] |
                        [0x76, 0x0, 0x0, 0x0] => {
                            sender.send(0xF).unwrap();
                            ()
                        },
                        [0x1b, 0x0, 0x0, 0x0] => {
                            frontend::exit();
                            exit(0)
                        },
                        _ => (),
                    }
                },
                Err(_) => (),
            }
        }
    });
    tx
}
