/*
    Chip8Emu - a Chip-8 emulator
    Copyright (C) 2021  Philip Damianik

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

 use std::{sync::{Arc, Mutex, mpsc::{Receiver, Sender}}};

use crate::emu::display::DisplayCmd;

#[cfg(feature = "tui")]
mod term;

//const FPS: u8 = 60;
//const FRAME_DELAY: Duration = Duration::from_nanos((1_000_000_000f64/FPS as f64) as u64);

pub fn init(display_changes: Receiver<DisplayCmd>, keyboard_sender: Sender<[u8; 4]>, beep: Arc<Mutex<bool>>) {
	#[cfg(feature = "tui")]
	term::init(display_changes, keyboard_sender.clone(), beep);
}

pub fn exit() {
	#[cfg(feature = "tui")]
	term::exit();
}
