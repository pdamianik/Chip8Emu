use std::sync::mpsc::{Receiver, Sender, TryRecvError::Empty, channel};

pub struct Keyboard {
	pressed_key: u8,
	keyboard_sender: Sender<u8>,
	keyboard_receiver: Receiver<u8>,
}

impl Keyboard {
	pub fn new() -> Self {
		let (tx, rx) = channel();
		Keyboard {
			pressed_key: 0x10,
			keyboard_sender: tx,
			keyboard_receiver: rx,
		}
	}

	pub fn recieve_key(&mut self) -> Result<(), ()> {
		let key = match self.keyboard_receiver.try_recv() {
			Ok(key) => key,
			Err(Empty) => 0x10,
			Err(_) => return Err(()),
		};
		
		self.pressed_key = key;
		return Ok(());
	}

	pub fn is_key_pressed(&self, key: u8) -> bool {
		self.pressed_key == key
	}

	pub fn wait_for_key(&mut self) -> Result<u8, ()> {
		if self.pressed_key > 0x10 {
			return Ok(self.pressed_key)
		}

		let key = match self.keyboard_receiver.recv() {
			Ok(key) => key,
			Err(_) => return Err(()),
		};

		self.pressed_key = key;
		return Ok(key)
	}

	pub fn new_driver(&mut self) -> Sender<u8> {
		self.keyboard_sender.clone()
	}
}
