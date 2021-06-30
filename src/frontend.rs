use std::{sync::{Arc, Mutex, mpsc::{Receiver, Sender}}};

use crate::emu::display::Change;

#[cfg(feature = "tui")]
mod term;

//const FPS: u8 = 60;
//const FRAME_DELAY: Duration = Duration::from_nanos((1_000_000_000f64/FPS as f64) as u64);

pub fn init(display_changes: Receiver<Change>, keyboard_sender: Sender<[u8; 4]>, beep: Arc<Mutex<bool>>) {
	#[cfg(feature = "tui")]
	term::init(display_changes, keyboard_sender.clone(), beep);
}

pub fn exit() {
	#[cfg(feature = "tui")]
	term::exit();
}
