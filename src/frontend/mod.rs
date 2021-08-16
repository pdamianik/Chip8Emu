use crate::emu::Chip8Emu;

use self::term::TermController;

#[cfg(feature = "tui")]
mod term;

//const FPS: u8 = 60;
//const FRAME_DELAY: Duration = Duration::from_nanos((1_000_000_000f64/FPS as f64) as u64);

trait FrontendController: Drop { }

pub struct Frontend {
	controllers: Vec<Box<dyn FrontendController>>,
}

impl Frontend {
	pub fn new(emulator: &mut Chip8Emu) -> Self {
		Self {
			controllers: vec![
				#[cfg(feature = "tui")]
				Box::new(TermController::new(emulator.get_screen_changes(), emulator.new_keyboard_driver(), emulator.is_beeping())),
			],
		}
	}
}

