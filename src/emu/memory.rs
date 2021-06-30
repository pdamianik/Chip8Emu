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

 use std::ops::{Index,
                IndexMut,
                Range,
                RangeFrom,
                RangeFull,
                RangeInclusive,
                RangeTo,
                RangeToInclusive};

/// 4k RAM Memory
#[derive(Debug)]
pub struct Memory {
    mem: [u8; Self::MEMORY_SIZE]
}

impl Memory {
    /// The default RAM size
    const MEMORY_SIZE: usize = 0x1000;

    /// Create a new Memory
    pub fn new() -> Self {
        Self {
            mem: [0x0; Self::MEMORY_SIZE],
        }
    }

    /// Loads a certain amount of data into RAM
    pub fn load(&mut self, data: &[u8], start: &usize) {
        for i in 0..data.len() as usize {
            self.mem[start+i] = data[i];
        }
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.mem[index as usize]
    }
}

impl Index<Range<u16>> for Memory {
    type Output = [u8];

    fn index(&self, range: Range<u16>) -> &Self::Output {
        &self.mem[(range.start as usize)..(range.end as usize)]
    }
}

impl Index<RangeFrom<u16>> for Memory {
    type Output = [u8];

    fn index(&self, range: RangeFrom<u16>) -> &Self::Output {
        &self.mem[range.start as usize..]
    }
}

impl Index<RangeFull> for Memory {
    type Output = [u8];

    fn index(&self, _range: RangeFull) -> &Self::Output {
        &self.mem[..]
    }
}

impl Index<RangeInclusive<u16>> for Memory {
    type Output = [u8];

    fn index(&self, range: RangeInclusive<u16>) -> &Self::Output {
        &self.mem[(*range.start() as usize)..=(*range.end() as usize)]
    }
}

impl Index<RangeTo<u16>> for Memory {
    type Output = [u8];

    fn index(&self, range: RangeTo<u16>) -> &Self::Output {
        &self.mem[..range.end as usize]
    }
}


impl Index<RangeToInclusive<u16>> for Memory {
    type Output = [u8];

    fn index(&self, range: RangeToInclusive<u16>) -> &Self::Output {
        &self.mem[..=range.end as usize]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.mem[index as usize]
    }
}

impl IndexMut<Range<u16>> for Memory {
    fn index_mut(&mut self, range: Range<u16>) -> &mut Self::Output {
        &mut self.mem[(range.start as usize)..(range.end as usize)]
    }
}

impl IndexMut<RangeFrom<u16>> for Memory {
    fn index_mut(&mut self, range: RangeFrom<u16>) -> &mut Self::Output {
        &mut self.mem[range.start as usize..]
    }
}

impl IndexMut<RangeFull> for Memory {
    fn index_mut(&mut self, _range: RangeFull) -> &mut Self::Output {
        &mut self.mem[..]
    }
}

impl IndexMut<RangeInclusive<u16>> for Memory {
    fn index_mut(&mut self, range: RangeInclusive<u16>) -> &mut Self::Output {
        &mut self.mem[(*range.start() as usize)..=(*range.end() as usize)]
    }
}

impl IndexMut<RangeTo<u16>> for Memory {
    fn index_mut(&mut self, range: RangeTo<u16>) -> &mut Self::Output {
        &mut self.mem[..range.end as usize]
    }
}


impl IndexMut<RangeToInclusive<u16>> for Memory {
    fn index_mut(&mut self, range: RangeToInclusive<u16>) -> &mut Self::Output {
        &mut self.mem[..=range.end as usize]
    }
}
