use std::{cmp::min, io::{self, Error, ErrorKind::Interrupted, Read, Write, stdin, stdout}, sync::{Arc, Mutex, mpsc::{Receiver, Sender}}, thread::{self, sleep}, time::Duration};
use crate::emu::display::DisplayCmd;

mod consoleapi;

#[inline(always)]
fn console_init() {
	
	use consoleapi::*;
	use std::os::windows::io::AsRawHandle;

	
	let h_stdin = stdin().as_raw_handle();
	let h_stdout = stdout().as_raw_handle();

	unsafe {

		let mode: LPDWORD = &mut 0;
		if GetConsoleMode(h_stdin, mode) == 0 {
			//panic!("Failed to get the mode of the stdin console: {}", GetLastError());
		}

		let mode: DWORD = *mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT) | (/*ENABLE_WINDOW_INPUT | */ENABLE_VIRTUAL_TERMINAL_INPUT);
		if SetConsoleMode(h_stdin, mode) == 0 {
			//panic!("Failed to set the mode of the stdin console: {}", GetLastError());
		}

		let mode: LPDWORD = &mut 0;
		if GetConsoleMode(h_stdout, mode) == 0 {
			//panic!("Failed to get the mode of the stdout console: {}", GetLastError());
		}

		let mode: DWORD = *mode & !(ENABLE_WRAP_AT_EOL_OUTPUT) | (ENABLE_VIRTUAL_TERMINAL_PROCESSING);
		if SetConsoleMode(h_stdout, mode) == 0 {
			//panic!("Failed to set the mode of the stdout console: {}", GetLastError());
		}
	}

	print!("\x1b[?1049h\x1b[?25l\x1b]0;Chip-8 Emulator\x07\x1b[;H");

	render_ui();

	io::stdout().flush().unwrap();
}

#[inline(always)]
fn render_ui() {
	println!("\x1b[2J\u{250C}{}\u{2510}", "\u{2500}".repeat(128));
	for _ in 0..32 {
		println!("\u{2502}{}\u{2502}", " ".repeat(128));
	}
	println!("\u{2514}{}\u{2518}", "\u{2500}".repeat(128));
	io::stdout().flush().unwrap();
}

#[inline(always)]
fn render_change(change: DisplayCmd) {
	if let DisplayCmd::Change(data, x, y) = change {
		for (index, row) in data.iter().enumerate() {
			print!("\x1b[{};{}H", y+index as u8+2 as u8, x as u8*2+2);
			let mut mask = 0b1000_0000u8;
			let end = min(64 - x, 8);
			for _ in 0..end {
				if row & mask == 0 {
					print!("  ");
				} else {
					print!("\u{258D}\u{258D}");
				}
				mask >>= 1;
			}
		}
	};

	print!("\x1b[0;0H");
	io::stdout().flush().unwrap();
}

#[inline(always)]
fn render_changes(display_cmds: Receiver<DisplayCmd>) {
	thread::spawn(move || {
		loop {
			let cmd = match display_cmds.recv() {
				Ok(cmd) => cmd,
				Err(_) => break,
			};
			match cmd {
				DisplayCmd::Change(_, _, _) => render_change(cmd),
				DisplayCmd::Clear => (),
			}
		}
	});
}

#[inline(always)]
fn keyboard_init(sender: Sender<[u8; 4]>) {
	thread::spawn(move || {
		let stdin = stdin();
		let mut stdin = stdin.lock();

		loop {
			let mut buf = [0u8; 4];
			match stdin.read(&mut buf) {
				Ok(_) => {
					sender.send(buf).unwrap();
				},
    			Err(_) => {
					if Error::last_os_error().kind() == Interrupted {
						continue;
					}
					break;
				},
			}
		}
	});
}

#[inline(always)]
fn bell_init(beep: Arc<Mutex<bool>>) {
	thread::spawn(move || {
		loop {
			{
				let beep_access = beep.lock().unwrap();
				if *beep_access {
					print!("\x07");
					stdout().flush().unwrap();
				}
			}
			sleep(Duration::from_nanos((1_000_000_000f64/60f64) as u64))
		}
	});
}

#[inline(always)]
pub fn init(changes: Receiver<DisplayCmd>, keyboard_sender: Sender<[u8; 4]>, beep: Arc<Mutex<bool>>) {
	console_init();
	render_changes(changes);
	keyboard_init(keyboard_sender);
	bell_init(beep);
}

#[inline(always)]
fn console_exit() {
	print!("\x1b[?1049l\x1b[?25h");
}

pub fn exit() {
	#[cfg(any(windows, unix))]
	console_exit();
}
