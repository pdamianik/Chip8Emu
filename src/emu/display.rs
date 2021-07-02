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

 use std::{fs::File, io::Write, sync::{
		mpsc::{
			channel,
			Sender,
			Receiver,
		}
	}, usize};

/// A sprite consisting of binary data and location.
/// A `1` in the binary data represents a change in color at that position
/// whereas a `0` represents no change (like an xor with the existing data)
#[derive(Debug)]
pub struct Sprite<'a> {
	pub data: &'a [u8],
	pub x: u8,
	pub y: u8,
}

/// An enum that contains all the possible commands for the view to process
#[derive(Debug)]
pub enum DisplayCmd {
	/// A change in the display buffer due to a sprite being drawn to the display buffer.
	/// The change consist of a data field containing the new look of the buffer at the given location
	/// and the x and y coordinate of the change.
	Change(Vec<u8>, u8, u8),
	/// A command that indicates that the display should be cleared
	Clear,
}

/// A representation of the screen of the emulator and its changes.
pub struct Display {
	/// This buffer contains the image to be displayed. It stores the screens state in binary form.
	/// The screen is 64 pixels wide and 32 pixels high so the screen data gets stored as 32 64 bit rows with each
	/// bit representing 1 pixel. The bits themselfs just represent two different colors at a given location
	buffer: [u64; 32],
	changes: Vec<Sender<DisplayCmd>>,
	#[cfg(debug_assertions)]
	logfile: File,
}

impl Display {
	/// constructs a new display
	pub fn new() -> Self {
		Self {
			buffer: [0x0; 32],
			changes: Vec::new(),
			#[cfg(debug_assertions)]
			logfile: File::create("display.log").unwrap(),
		}
	}

	pub fn clear(&mut self) {
		self.buffer = [0x0; 32];
		for tx in self.changes.iter() {
			match tx.send(DisplayCmd::Clear) {
				Ok(_) => (),
				Err(_) => (),
			}
		}
	}

	pub fn draw(&mut self, sprite: Sprite) -> Result<bool,()> {
		let mut updated = false;
		let Sprite { data, x, y} = sprite;
		let mut changes = Vec::new();

		#[cfg(debug_assertions)]
		writeln!(self.logfile, "Sprite: {:?}", sprite).unwrap();

		let left = x <= 56;
		#[cfg(debug_assertions)]
		writeln!(self.logfile, "left {:?}", left).unwrap();

		for (index, row) in data.iter().enumerate() {
			if y as usize + index >= 32 {
				break;
			}

			let buf_row;
			if left {
				buf_row = (*row as u64) << (56 - x); // stretch the current row to full width
			} else {
				buf_row = (*row as u64) >> (x - 56); // stretch the current row to full width
			}
			updated |= self.buffer[y as usize + index] & buf_row > 0; // check if the buffer gets updated
			self.buffer[y as usize + index] ^= buf_row; // write changes to buffer
			if left {
				changes.push((self.buffer[y as usize + index] >> (56 - x) & 0xFF) as u8); // get result as change
			} else {
				changes.push((self.buffer[y as usize + index] << (x - 56) & 0xFF) as u8); // get result as change
			}
		}

		for tx in self.changes.iter() {
			match tx.send(DisplayCmd::Change(changes.clone(), x, y)) {
				Ok(_) => (),
				Err(_) => (),
			}
		}

		Ok(updated)
	}

	pub fn get_changes_pipe(&mut self) -> Receiver<DisplayCmd> {
		let (tx,rx) = channel::<DisplayCmd>();
		self.changes.push(tx);
		rx
	}
}