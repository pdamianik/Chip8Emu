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

 use std::{fs::File, io::Write, sync::{Arc, Mutex, mpsc::{Receiver, Sender}}, time::Duration, usize};

use rand::{Rng, prelude::ThreadRng};

use self::display::Change;

mod memory;
mod timer;
pub mod display;
pub mod keyboard;

// types for parameters
/// A 12-Bit address
type NNN = u16;
/// A 8-Bit constant
type NN = u8;
/// A 4-Bit constant
type N = u8;
/// The 4-Bit identifier V[0] to V[F] of a register
type X = u8;
/// The 4-Bit identifier V[0] to V[F] of a second register
type Y = u8;

/// All available instruction types for the chip-8 cpu
#[derive(Debug)]
pub enum Instruction {
    /// Calls machine code routine
    Call(NNN),
    /// Clears the screen
    DispClr,
    /// Returns from subroutine
    FlowRet,
    /// Jumps to address
    FlowJmp(NNN),
    /// Calls subroutine
    FlowCall(NNN),
    /// Skips if register is equal to literal
    CondEqL(X, NN),
    /// Skips if register is not equal to literal
    CondNoEqL(X, NN),
    /// Skips if register is equal to register
    CondEqRg(X, Y),
    /// Sets register
    RegConst(X, NN),
    /// Adds to register
    RegAdd(X, NN),
    /// Sets one register to the value of another register
    Assign(X, Y),
    /// Bitwise OR
    BitOr(X, Y),
    /// Bitwise AND
    BitAnd(X, Y),
    /// Bitwise XOR
    BitXor(X, Y),
    /// Addition
    MathAdd(X, Y),
    /// Subtraction
    MathSub(X, Y),
    /// Bitwise Shift right, store least significant bit of initial value in other register
    BitShiftR(X, Y),
    /// Store Vy-Vx in Vx
    InvertSub(X, Y),
    /// Bitwise Shift left, store most significant bit of initial value in other register
    BitShiftL(X, Y),
    /// Skips if register is not equal to register
    CondNoEqRg(X, Y),
    /// Sets the memory pointer I
    SetPoint(NNN),
    /// Jumps to address + V0
    FlowJmpV0(NNN),
    /// Random generation
    RNG(X, NN),
    /// Draw
    DispDraw(X, Y, N),
    /// Skip if key is pressed
    CondKey(X),
    /// Skip if key is not pressed
    CondNotKey(X),
    /// Store the value of the delay timer in a register
    DelTimrGet(X),
    /// Wait for a key press and store key
    WaitKey(X),
    /// Set the delay timer
    DelTimrSet(X),
    /// Set the delay timer
    SndTimrSet(X),
    /// Add register to pointer (no status register change)
    PointAdd(X),
    /// Set the pointer to the location of a character
    PointChar(X),
    /// Store the BCD representation of register at I
    BCDStore(X),
    /// Saves register to memory
    RegDmp(X),
    /// Loads regsiter from memory
    RegLoad(X),
    /// No operation
    StopExecution,
}

// Constants
/// instructions per second
const EXEC_SPEED: u64 = 700;
/// delay between instructions
const EXEC_DELAY: Duration = Duration::from_nanos(1_000_000_000u64/EXEC_SPEED);
/// ticks per second
const TICK_SPEED: u64 = 60;
/// delay between ticks
const TICK_DELAY: Duration = Duration::from_nanos(1_000_000_000u64/TICK_SPEED);

/// the default font
const DEFAULT_FONTPACK: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// The default font location
const DEFAULT_FONTPACK_LOCATION: usize = 0x50;

/// CPU
pub struct Chip8Emu {
    /// RAM
    ram: memory::Memory,
    /// Timers
    time: timer::Timer,
    /// Display
    display: display::Display,
    /// Keyboard
    keyboard: keyboard::Keyboard,
    /// Programm counter
    pc: u16,
    /// Stack
    stack: [u16; 0xFF],
    /// Stack pointer
    sp: u8,
    /// Registers V0 to VF (VF is the status flag register)
    reg: [u8; 0x10],
    /// Address pointer
    i: u16,
    /// Log
    #[cfg(debug_assertions)]
    log: File,
}

/// The main emulator
impl Chip8Emu {
    /// Instanciate the emulator
    pub fn new(rom: [u8; 0xE00]) -> Self {
        let mut ram = memory::Memory::new();

        ram.load(&DEFAULT_FONTPACK, DEFAULT_FONTPACK_LOCATION);
        ram.load(&rom, 0x200);

        Self {
            ram, // RAM
            time: timer::Timer::new(), // Timer
            display: display::Display::new(), // Display
            keyboard: keyboard::Keyboard::new(), // Keyboard
            pc: 0x200, // Programm counter
            stack: [0x0u16; 0xFF], // Stack
            sp: 0x0, // Stack pointer
            reg: [0x0; 0x10], // Registers V0 to VF (VF is the status flag register)
            i: 0x0, // Address pointer
            #[cfg(debug_assertions)]
            log: File::create("emu.log").unwrap(),
        }
    }

    pub fn run(&mut self) {
        self.time.clone().start(TICK_DELAY);
        
        loop {
            match self.step() {
                Ok(_) => /*thread::sleep(EXEC_DELAY)*/(),
                Err(_) => break,
            };
        }
    }

    pub fn step(&mut self) -> Result<(),()> {
        #[cfg(debug_assertions)]
        writeln!(self.log, "{:?}", self.reg).unwrap();
        let instruction = self.fetch();
        #[cfg(debug_assertions)]
        writeln!(self.log, "{:#04x?}", instruction).unwrap();
        let instruction = Self::decode(&instruction);
        #[cfg(debug_assertions)]
        writeln!(self.log, "{:?}", instruction).unwrap();
        self.execute(&instruction)
    }

    /// Fetch the next instruction from memory
    fn fetch(&mut self) -> u16 {
        let inst1 = (self.ram[self.pc] as u16) << 8; // fetch the first part of the instructions
        let inst2 = self.ram[self.pc+1] as u16; // fetch the second part of the instruction
        self.pc += 2; // two instructions got fetched
        inst1 + inst2 // Return the two instructions
    }

    /// Decode the instruction
    fn decode(inst: &u16) -> Instruction {
        let nibs = [
            inst >> 12,
            inst >> 8 & 0xF,
            inst >> 4 & 0xF,
            inst & 0xF,
        ];
        match nibs {
            // Clear the display
            [0x0, 0x0, 0xE, 0x0] => Instruction::DispClr,
            // Return from subroutine
            [0x0, 0x0, 0xE, 0xE] => Instruction::FlowRet,
            // Call machine code routine
            [0x0, _, _, _] => Instruction::Call(inst & 0xFFF),
            // Jump to address
            [0x1, _, _, _] => Instruction::FlowJmp(inst & 0xFFF),
            // Call subroutine
            [0x2, _, _, _] => Instruction::FlowCall(inst & 0xFFF),
            // Skip if register is equal to literal
            [0x3, _, _, _] => Instruction::CondEqL((inst >> 8 & 0xF) as u8, (inst & 0xFF) as u8),
            // Skip if register is not equal to literal
            [0x4, _, _, _] => Instruction::CondNoEqL((inst >> 8 & 0xF) as u8, (inst & 0xFF) as u8),
            // Skip if register is equal to register
            [0x5, _, _, 0x0] => Instruction::CondEqRg((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8),
            // Set register to literal
            [0x6, _, _, _] => Instruction::RegConst((inst >> 8 & 0xF) as u8, (inst & 0xFF) as u8),
            // Add literal to register
            [0x7, _, _, _] => Instruction::RegAdd((inst >> 8 & 0xF) as u8, (inst & 0xFF) as u8),
            // Set register to other register
            [0x8, _, _, 0x0] => Instruction::Assign((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8),
            // Bitwise OR
            [0x8, _, _, 0x1] => Instruction::BitOr((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8),
            // Bitwise AND
            [0x8, _, _, 0x2] => Instruction::BitAnd((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8),
            // Bitwise XOR
            [0x8, _, _, 0x3] => Instruction::BitXor((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8),
            // Register add
            [0x8, _, _, 0x4] => Instruction::MathAdd((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8),
            // Register subtract
            [0x8, _, _, 0x5] => Instruction::MathSub((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8),
            // Bitwise shift right
            [0x8, _, _, 0x6] => Instruction::BitShiftR((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8),
            // Store Vy-Vx in Vx
            [0x8, _, _, 0xE7] => Instruction::InvertSub((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8),
            // Bitwise shift left
            [0x8, _, _, 0xE] => Instruction::BitShiftL((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8),
            // Skips if register is not equal to register
            [0x9, _, _, 0x0] => Instruction::CondNoEqRg((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8),
            // Set I (the memory pointer)
            [0xA, _, _, _] => Instruction::SetPoint(inst & 0xFFF),
            // Jump to address plus V0
            [0xB, _, _, _] => Instruction::FlowJmpV0(inst & 0xFFF),
            // random number generation
            [0xC, _, _, _] => Instruction::RNG((inst >> 8 & 0xF) as u8, (inst & 0xFF) as u8),
            // draw sprite
            [0xD, _, _, _] => Instruction::DispDraw((inst >> 8 & 0xF) as u8, (inst >> 4 & 0xF) as u8, (inst & 0xF) as u8),
            // key press skip
            [0xE, _, 0x9, 0xE] => Instruction::CondKey((inst >> 8 & 0xF) as u8),
            // no key press skip
            [0xE, _, 0xA, 0x1] => Instruction::CondNotKey((inst >> 8 & 0xF) as u8),
            // store value of timer in register
            [0xF, _, 0x0, 0x7] => Instruction::DelTimrGet((inst >> 8 & 0xF) as u8),
            // wait for key press and store the key in register
            [0xF, _, 0x0, 0xA] => Instruction::WaitKey((inst >> 8 & 0xF) as u8),
            // set timer from register value
            [0xF, _, 0x1, 0x5] => Instruction::DelTimrSet((inst >> 8 & 0xF) as u8),
            // set sound timer from register value
            [0xF, _, 0x1, 0x8] => Instruction::SndTimrSet((inst >> 8 & 0xF) as u8),
            // add to memory pointer (I)
            [0xF, _, 0x1, 0xE] => Instruction::PointAdd((inst >> 8 & 0xF) as u8),
            // let memory pointer point to character of default font
            [0xF, _, 0x2, 0x9] => Instruction::PointChar((inst >> 8 & 0xF) as u8),
            // store BCD of register in memory at memory pointer (I)
            [0xF, _, 0x3, 0x3] => Instruction::BCDStore((inst >> 8 & 0xF) as u8),
            // store register V0 to Vx in memory at memory pointer (I)
            [0xF, _, 0x5, 0x5] => Instruction::RegDmp((inst >> 8 & 0xF) as u8),
            // load register V0 to Vx from memory at memory pointer (I)
            [0xF, _, 0x6, 0x5] => Instruction::RegLoad((inst >> 8 & 0xF) as u8),
            // No operation
            _ => Instruction::StopExecution
        }
    }

    /// Execute the instruction
    fn execute(&mut self, inst: &Instruction) -> Result<(), ()> {
        self.keyboard.recieve_key()?;

        match inst {
            // Calls machine code routine
            Instruction::Call(nnn) => {
                #[cfg(debug_assertions)]
                writeln!(self.log, "WARNING: Call to unknown internal platform dependent code at {}", nnn).unwrap();
                Ok(())
            },
            // Clears the screen
            Instruction::DispClr => {
                self.display.clear();
                Ok(())
            },
            // Returns from subroutine
            Instruction::FlowRet => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
                self.stack[self.sp as usize] = 0x0;
                Ok(())
            },
            // Jumps to address
            Instruction::FlowJmp(nnn) => {
                self.pc = *nnn;
                Ok(())
            },
            // Calls subroutine
            Instruction::FlowCall(nnn) => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = *nnn;
                Ok(())
            },
            // Skips if register is equal to literal
            Instruction::CondEqL(x, nn) => {
                if self.reg[*x as usize] == *nn {
                    self.pc += 2;
                }
                Ok(())
            },
            // Skips if register is not equal to literal
            Instruction::CondNoEqL(x, nn) => {
                if self.reg[*x as usize] != *nn {
                    self.pc += 2;
                }
                Ok(())
            },
            // Skips if register is equal to register
            Instruction::CondEqRg(x, y) => {
                if self.reg[*x as usize] == self.reg[*y as usize] {
                    self.pc += 2;
                }
                Ok(())
            },
            // Sets register
            Instruction::RegConst(x, nn) => {
                self.reg[*x as usize] = *nn;
                Ok(())
            },
            // Adds to register
            Instruction::RegAdd(x, nn) => {
                self.reg[*x as usize] += *nn;
                Ok(())
            },
            // Sets one register to the value of another register
            Instruction::Assign(x, y) => {
                self.reg[*x as usize] = self.reg[*y as usize];
                Ok(())
            },
            // Bitwise OR
            Instruction::BitOr(x, y) => {
                self.reg[*x as usize] |= self.reg[*y as usize];
                Ok(())
            },
            // Bitwise AND
            Instruction::BitAnd(x, y) => {
                self.reg[*x as usize] &= self.reg[*y as usize];
                Ok(())
            },
            // Bitwise XOR
            Instruction::BitXor(x, y) => {
                self.reg[*x as usize] ^= self.reg[*y as usize];
                Ok(())
            },
            // Addition
            Instruction::MathAdd(x, y) => {
                let result = self.reg[*x as usize] as u16 + self.reg[*y as usize] as u16;

                self.reg[0xF] = (result >> 8) as u8;
                self.reg[*x as usize] = result as u8;
                Ok(())
            },
            // Subtraction
            Instruction::MathSub(x, y) => {
                if self.reg[*y as usize] > self.reg[*x as usize] {
                    self.reg[0xF] = 1
                } else {
                    self.reg[0xF] = 0
                }
                self.reg[*x as usize] -= self.reg[*y as usize];
                Ok(())
            },
            // Bitwise Shift right, store least significant bit of initial value in VF
            Instruction::BitShiftR(x, _y) => {
                self.reg[0xF] = self.reg[*x as usize] & 0b1;
                self.reg[*x as usize] >>= 1;
                Ok(())
            },
            // Store Vy-Vx in Vx
            Instruction::InvertSub(x, y) => {
                if self.reg[*x as usize] > self.reg[*y as usize] {
                    self.reg[0xF] = 1
                } else {
                    self.reg[0xF] = 0
                }
                self.reg[*x as usize] = y - self.reg[*x as usize];
                Ok(())
            },
            // Bitwise Shift left, store most significant bit of initial value in VF
            Instruction::BitShiftL(x, _y) => {
                self.reg[0xF] = self.reg[*x as usize] >> 7;
                self.reg[*x as usize] <<= 1;
                Ok(())
            },
            // Skips if register is not equal to register
            Instruction::CondNoEqRg(x, y) => {
                if self.reg[*x as usize] != self.reg[*y as usize] {
                    self.pc += 2;
                }
                Ok(())
            },
            // Sets the memory pointer I
            Instruction::SetPoint(nnn) => {
                self.i = *nnn;
                Ok(())
            },
            // Jump to address + V0
            Instruction::FlowJmpV0(nnn) => {
                self.pc = *nnn + self.reg[0] as u16;
                Ok(())
            },
            // Random generation
            Instruction::RNG(x, nn) => {
                self.reg[*x as usize] = ThreadRng::default().gen_range(0..255) & *nn;
                Ok(())
            },
            // Draw
            Instruction::DispDraw(x, y, n) => {
                let x = self.reg[*x as usize];
                let y = self.reg[*y as usize];
                let n = *n as u16;
                let data = &self.ram[self.i..=self.i+n-1];

                let sprite = display::Sprite {
                    x,
                    y,
                    data,
                };

                match self.display.draw(sprite) {
                    Ok(change) => {
                        self.reg[0xF] = change as u8;
                        Ok(())
                    },
                    Err(()) => Err(())
                }
            },
            // Skip if key is pressed
            Instruction::CondKey(x) => {
                if self.keyboard.is_key_pressed(self.reg[*x as usize]) {
                    self.pc += 2;
                }
                Ok(())
            },
            // Skip if key is not pressed
            Instruction::CondNotKey(x) => {
                if !self.keyboard.is_key_pressed(self.reg[*x as usize]) {
                    self.pc += 2;
                }
                Ok(())
            },
            // Store the value of the delay timer in a register
            Instruction::DelTimrGet(x) => {
                let dtime_access = match self.time.dtime.lock() {
                    Ok(guard) => guard,
                    Err(_) => return Err(()),
                };
                self.reg[*x as usize] = *dtime_access;
                Ok(())
            },
            // Wait for a key press and store key
            Instruction::WaitKey(x) => {
                let key = self.keyboard.wait_for_key()?;
                self.reg[*x as usize] = key;                

                Ok(())
            },
            // Set the delay timer
            Instruction::DelTimrSet(x) => {
                let mut dtime_access = match self.time.dtime.lock() {
                    Ok(guard) => guard,
                    Err(_) => return Err(()),
                };
                *dtime_access = self.reg[*x as usize];
                Ok(())
            },
            // Set the delay timer
            Instruction::SndTimrSet(x) => {
                let mut stime_access = match self.time.stime.lock() {
                    Ok(guard) => guard,
                    Err(_) => return Err(()),
                };
                *stime_access = self.reg[*x as usize];
                Ok(())
            },
            // Add register to pointer (no status register change)
            Instruction::PointAdd(x) => {
                self.i += self.reg[*x as usize] as u16;
                Ok(())
            },
            // Set the pointer to the location of a character
            Instruction::PointChar(x) => {
                self.i = DEFAULT_FONTPACK_LOCATION as u16 + self.reg[*x as usize] as u16 * 5;
                Ok(())
            },
            // Store the BCD representation of register at I
            Instruction::BCDStore(x) => {
                let mut x = self.reg[*x as usize];
                const BASE: u8 = 10;

                for i in 0u16..=2 {
                    self.ram[self.i+2-i] = x % BASE;
                    x /= BASE;
                }

                Ok(())
            },
            // Saves register to memory
            Instruction::RegDmp(x) => {
                for reg_id in 0x0..=(*x as u16) {
                    self.ram[self.i+reg_id] = self.reg[reg_id as usize];
                }

                Ok(())
            },
            // Loads regsiter from memory
            Instruction::RegLoad(x) => {
                for reg_id in 0x0..=(*x as u16) {
                    self.reg[reg_id as usize] = self.ram[self.i+reg_id];
                }

                Ok(())
            },
            // No operation
            Instruction::StopExecution => {
                Err(())
            },
        }
    }

    pub fn is_beeping(&self) -> Arc<Mutex<bool>> {
        self.time.beep.clone()
    }

    pub fn get_screen_changes(&mut self) -> Receiver<Change> {
        self.display.get_changes_pipe()
    }

    pub fn new_keyboard_driver(&mut self) -> Sender<u8> {
        self.keyboard.new_driver()
    }
}