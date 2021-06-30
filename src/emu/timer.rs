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

 use std::{sync::{Arc,Mutex}, thread::{spawn, sleep}, time::Duration};


/// Timers
pub struct Timer {
    /// delay timer
    pub dtime: Arc<Mutex<u8>>,
    /// sound timer
    pub stime: Arc<Mutex<u8>>,
    /// is beeping
    pub beep: Arc<Mutex<bool>>,
}

impl Timer {
    /// Initialize the timer
    pub fn new() -> Self {
        Self {
            dtime: Arc::new(Mutex::new(0)),
            stime: Arc::new(Mutex::new(0)),
            beep: Arc::new(Mutex::new(false)),
        }
    }

    pub fn tick(&mut self) {
        let mut dtime_access = self.dtime.lock().unwrap();

        if *dtime_access > 0 {
            *dtime_access -= 1;
        }

        let mut stime_access = self.stime.lock().unwrap();
        let mut beep_access = self.beep.lock().unwrap();

        if *stime_access > 0 {
            *stime_access -= 1;
            *beep_access = true;
        } else {
            *beep_access = false;
        }
    }

    pub fn start(mut self, delay: Duration) {
        spawn(move || {
            loop {
                self.tick();
                sleep(delay);
            }
        });
    }
}

impl Clone for Timer {
    fn clone(&self) -> Self {
        Self {
            dtime: self.dtime.clone(),
            stime: self.stime.clone(),
            beep: self.beep.clone(),
        }
    }
}
