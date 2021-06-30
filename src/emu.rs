use std::{fs::File, io::Write, sync::{Arc, Mutex, mpsc::{Receiver, Sender}}, thread, time::Duration, usize};

use rand::{Rng, prelude::ThreadRng};

use self::display::Change;

mod memory;
mod timer;
pub mod display;
pub mod keyboard;

/// All available instruction types for the chip-8 cpu
#[derive(Debug)]
pub enum InstructionType {
    /// Calls machine code routine
    Call,
    /// Clears the screen
    DispClr,
    /// Returns from subroutine
    FlowRet,
    /// Jumps to address
    FlowJmp,
    /// Calls subroutine
    FlowCall,
    /// Skips if register is equal to literal
    CondEqL,
    /// Skips if register is not equal to literal
    CondNoEqL,
    /// Skips if register is equal to register
    CondEqRg,
    /// Sets register
    RegConst,
    /// Adds to register
    RegAdd,
    /// Sets one register to the value of another register
    Assign,
    /// Bitwise OR
    BitOr,
    /// Bitwise AND
    BitAnd,
    /// Bitwise XOR
    BitXor,
    /// Addition
    MathAdd,
    /// Subtraction
    MathSub,
    /// Bitwise Shift right, store least significant bit of initial value in other register
    BitShiftR,
    /// Store Vy-Vx in Vx
    InvertSub,
    /// Bitwise Shift left, store most significant bit of initial value in other register
    BitShiftL,
    /// Skips if register is not equal to register
    CondNoEqRg,
    /// Sets the memory pointer I
    SetPoint,
    /// Jumps to address + V0
    FlowJmpV0,
    /// Random generation
    RNG,
    /// Draw
    DispDraw,
    /// Skip if key is pressed
    CondKey,
    /// Skip if key is not pressed
    CondNotKey,
    /// Store the value of the delay timer in a register
    DelTimrGet,
    /// Wait for a key press and store key
    WaitKey,
    /// Set the delay timer
    DelTimrSet,
    /// Set the delay timer
    SndTimrSet,
    /// Add register to pointer (no status register change)
    PointAdd,
    /// Set the pointer to the location of a character
    PointChar,
    /// Store the BCD representation of register at I
    BCDStore,
    /// Saves register to memory
    RegDmp,
    /// Loads regsiter from memory
    RegLoad,
    /// No operation
    StopExecution,
}

/// A representation of a single decoded instruction containing the instruction type and the instructions parameters
#[derive(Debug)]
pub struct Instruction {
    /// Instruction type
    insttype: InstructionType,
    /// An address Parameter
    nnn: Option<u16>,
    /// A one byte constant
    nn: Option<u8>,
    /// A half byte constant
    n: Option<u8>,
    /// The first register identifier
    x: Option<u8>,
    /// The second register identifier
    y: Option<u8>,
}

impl Instruction {
    fn param<P>(param: Option<P>) -> Result<P,()> {
        param.ok_or(())
    }
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

        ram.load(&DEFAULT_FONTPACK, &DEFAULT_FONTPACK_LOCATION);
        ram.load(&rom, &0x200);

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
            [0x0, 0x0, 0xE, 0x0] => Instruction {
                insttype: InstructionType::DispClr,
                nnn: None,
                nn: None,
                n: None,
                x: None,
                y: None,
            },
            // Return from subroutine
            [0x0, 0x0, 0xE, 0xE] => Instruction {
                insttype: InstructionType::FlowRet,
                nnn: None,
                nn: None,
                n: None,
                x: None,
                y: None,
            },
            // Call machine code routine
            [0x0, _, _, _] => Instruction {
                insttype: InstructionType::Call,
                nnn: Some(inst & 0xFFF),
                nn: None,
                n: None,
                x: None,
                y: None,
            },
            // Jump to address
            [0x1, _, _, _] => Instruction {
                insttype: InstructionType::FlowJmp,
                nnn: Some(inst & 0xFFF),
                nn: None,
                n: None,
                x: None,
                y: None,
            },
            // Call subroutine
            [0x2, _, _, _] => Instruction {
                insttype: InstructionType::FlowCall,
                nnn: Some(inst & 0xFFF),
                nn: None,
                n: None,
                x: None,
                y: None,
            },
            // Skip if register is equal to literal
            [0x3, _, _, _] => Instruction {
                insttype: InstructionType::CondEqL,
                nnn: None,
                nn: Some((inst & 0xFF) as u8),
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // Skip if register is not equal to literal
            [0x4, _, _, _] => Instruction {
                insttype: InstructionType::CondNoEqL,
                nnn: None,
                nn: Some((inst & 0xFF) as u8),
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // Skip if register is equal to register
            [0x5, _, _, 0x0] => Instruction {
                insttype: InstructionType::CondEqRg,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // Set register to literal
            [0x6, _, _, _] => Instruction {
                insttype: InstructionType::RegConst,
                nnn: None,
                nn: Some((inst & 0xFF) as u8),
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // Add literal to register
            [0x7, _, _, _] => Instruction {
                insttype: InstructionType::RegAdd,
                nnn: None,
                nn: Some((inst & 0xFF) as u8),
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // Set register to other register
            [0x8, _, _, 0x0] => Instruction {
                insttype: InstructionType::Assign,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // Bitwise OR
            [0x8, _, _, 0x1] => Instruction {
                insttype: InstructionType::BitOr,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // Bitwise AND
            [0x8, _, _, 0x2] => Instruction {
                insttype: InstructionType::BitAnd,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // Bitwise XOR
            [0x8, _, _, 0x3] => Instruction {
                insttype: InstructionType::BitXor,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // Register add
            [0x8, _, _, 0x4] => Instruction {
                insttype: InstructionType::MathAdd,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // Register subtract
            [0x8, _, _, 0x5] => Instruction {
                insttype: InstructionType::MathSub,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // Bitwise shift right
            [0x8, _, _, 0x6] => Instruction {
                insttype: InstructionType::BitShiftR,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // Store Vy-Vx in Vx
            [0x8, _, _, 0xE7] => Instruction {
                insttype: InstructionType::InvertSub,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // Bitwise shift left
            [0x8, _, _, 0xE] => Instruction {
                insttype: InstructionType::BitShiftL,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // Skips if register is not equal to register
            [0x9, _, _, 0x0] => Instruction {
                insttype: InstructionType::CondNoEqRg,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // Set I (the memory pointer)
            [0xA, _, _, _] => Instruction {
                insttype: InstructionType::SetPoint,
                nnn: Some((inst & 0xFFF) as u16),
                nn: None,
                n: None,
                x: None,
                y: None,
            },
            // Jump to address plus V0
            [0xB, _, _, _] => Instruction {
                insttype: InstructionType::FlowJmpV0,
                nnn: Some((inst & 0xFFF) as u16),
                nn: None,
                n: None,
                x: None,
                y: None,
            },
            // random number generation
            [0xC, _, _, _] => Instruction {
                insttype: InstructionType::RNG,
                nnn: None,
                nn: Some((inst & 0xFF) as u8),
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // draw sprite
            [0xD, _, _, _] => Instruction {
                insttype: InstructionType::DispDraw,
                nnn: None,
                nn: None,
                n: Some((inst & 0xF) as u8),
                x: Some((inst >> 8 & 0xF) as u8),
                y: Some((inst >> 4 & 0xF) as u8),
            },
            // key press skip
            [0xE, _, 0x9, 0xE] => Instruction {
                insttype: InstructionType::CondKey,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // no key press skip
            [0xE, _, 0xA, 0x1] => Instruction {
                insttype: InstructionType::CondNotKey,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // store value of timer in register
            [0xF, _, 0x0, 0x7] => Instruction {
                insttype: InstructionType::DelTimrGet,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // wait for key press and store the key in register
            [0xF, _, 0x0, 0xA] => Instruction {
                insttype: InstructionType::WaitKey,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // set timer from register value
            [0xF, _, 0x1, 0x5] => Instruction {
                insttype: InstructionType::DelTimrSet,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // set sound timer from register value
            [0xF, _, 0x1, 0x8] => Instruction {
                insttype: InstructionType::SndTimrSet,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // add to memory pointer (I)
            [0xF, _, 0x1, 0xE] => Instruction {
                insttype: InstructionType::PointAdd,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // let memory pointer point to character of default font
            [0xF, _, 0x2, 0x9] => Instruction {
                insttype: InstructionType::PointChar,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // store BCD of register in memory at memory pointer (I)
            [0xF, _, 0x3, 0x3] => Instruction {
                insttype: InstructionType::BCDStore,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // store register in memory at memory pointer (I)
            [0xF, _, 0x5, 0x5] => Instruction {
                insttype: InstructionType::RegDmp,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // load register from memory at memory pointer (I)
            [0xF, _, 0x6, 0x5] => Instruction {
                insttype: InstructionType::RegLoad,
                nnn: None,
                nn: None,
                n: None,
                x: Some((inst >> 8 & 0xF) as u8),
                y: None,
            },
            // No operation
            _ => Instruction {
                insttype: InstructionType::StopExecution,
                nnn: None,
                nn: None,
                n: None,
                x: None,
                y: None,
            }
        }
    }

    /// Execute the instruction
    fn execute(&mut self, inst: &Instruction) -> Result<(), ()> {
        self.keyboard.recieve_key()?;

        match inst.insttype {
            // Calls machine code routine
            InstructionType::Call => {
                //eprint!("Not implemented: {:?}", inst);
                Ok(())
            },
            // Clears the screen
            InstructionType::DispClr => {
                self.display.clear();
                Ok(())
            },
            // Returns from subroutine
            InstructionType::FlowRet => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
                self.stack[self.sp as usize] = 0x0;
                Ok(())
            },
            // Jumps to address
            InstructionType::FlowJmp => {
                self.pc = Instruction::param(inst.nnn)?;
                Ok(())
            },
            // Calls subroutine
            InstructionType::FlowCall => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = Instruction::param(inst.nnn)?;
                Ok(())
            },
            // Skips if register is equal to literal
            InstructionType::CondEqL => {
                if self.reg[Instruction::param(inst.x)? as usize] == Instruction::param(inst.nn)? {
                    self.pc += 2;
                }
                Ok(())
            },
            // Skips if register is not equal to literal
            InstructionType::CondNoEqL => {
                if self.reg[Instruction::param(inst.x)? as usize] != Instruction::param(inst.nn)? {
                    self.pc += 2;
                }
                Ok(())
            },
            // Skips if register is equal to register
            InstructionType::CondEqRg => {
                if self.reg[Instruction::param(inst.x)? as usize] == self.reg[Instruction::param(inst.y)? as usize] {
                    self.pc += 2;
                }
                Ok(())
            },
            // Sets register
            InstructionType::RegConst => {
                self.reg[Instruction::param(inst.x)? as usize] = Instruction::param(inst.nn)?;
                Ok(())
            },
            // Adds to register
            InstructionType::RegAdd => {
                self.reg[Instruction::param(inst.x)? as usize] += Instruction::param(inst.nn)?;
                Ok(())
            },
            // Sets one register to the value of another register
            InstructionType::Assign => {
                self.reg[Instruction::param(inst.x)? as usize] = self.reg[Instruction::param(inst.y)? as usize];
                Ok(())
            },
            // Bitwise OR
            InstructionType::BitOr => {
                self.reg[Instruction::param(inst.x)? as usize] |= self.reg[Instruction::param(inst.y)? as usize];
                Ok(())
            },
            // Bitwise AND
            InstructionType::BitAnd => {
                self.reg[Instruction::param(inst.x)? as usize] &= self.reg[Instruction::param(inst.y)? as usize];
                Ok(())
            },
            // Bitwise XOR
            InstructionType::BitXor => {
                self.reg[Instruction::param(inst.x)? as usize] ^= self.reg[Instruction::param(inst.y)? as usize];
                Ok(())
            },
            // Addition
            InstructionType::MathAdd => {
                let x = Instruction::param(inst.x)? as usize;
                let result = self.reg[x] as u16 + self.reg[Instruction::param(inst.y)? as usize] as u16;

                self.reg[0xF] = (result >> 8) as u8;
                self.reg[x] = result as u8;
                Ok(())
            },
            // Subtraction
            InstructionType::MathSub => {
                let x = Instruction::param(inst.x)? as usize;
                let y = self.reg[Instruction::param(inst.y)? as usize];
                if y > self.reg[x] {
                    self.reg[0xF] = 1
                } else {
                    self.reg[0xF] = 0
                }
                self.reg[x] -= y;
                Ok(())
            },
            // Bitwise Shift right, store least significant bit of initial value in other register
            InstructionType::BitShiftR => {
                let x = Instruction::param(inst.x)? as usize;
                self.reg[0xF] = self.reg[x] & 0b1;
                self.reg[x] >>= 1;
                Ok(())
            },
            // Store Vy-Vx in Vx
            InstructionType::InvertSub => {
                let x = Instruction::param(inst.x)? as usize;
                let y = self.reg[Instruction::param(inst.y)? as usize];
                if self.reg[x] > y {
                    self.reg[0xF] = 1
                } else {
                    self.reg[0xF] = 0
                }
                self.reg[x] = y - self.reg[x];
                Ok(())
            },
            // Bitwise Shift left, store most significant bit of initial value in other register
            InstructionType::BitShiftL => {
                let x = Instruction::param(inst.x)? as usize;
                self.reg[0xF] = self.reg[x] >> 7;
                self.reg[x] <<= 1;
                Ok(())
            },
            // Skips if register is not equal to register
            InstructionType::CondNoEqRg => {
                if self.reg[Instruction::param(inst.x)? as usize] != self.reg[Instruction::param(inst.y)? as usize] {
                    self.pc += 2;
                }
                Ok(())
            },
            // Sets the memory pointer I
            InstructionType::SetPoint => {
                self.i = Instruction::param(inst.nnn)?;
                Ok(())
            },
            // Jump to address + V0
            InstructionType::FlowJmpV0 => {
                self.pc = Instruction::param(inst.nnn)? + self.reg[0] as u16;
                Ok(())
            },
            // Random generation
            InstructionType::RNG => {
                self.reg[Instruction::param(inst.x)? as usize] = ThreadRng::default().gen_range(0..255) & Instruction::param(inst.nn)?;
                Ok(())
            },
            // Draw
            InstructionType::DispDraw => {
                let x = self.reg[Instruction::param(inst.x)? as usize];
                let y = self.reg[Instruction::param(inst.y)? as usize];
                let n = Instruction::param(inst.n)? as u16;
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
            InstructionType::CondKey => {
                if self.keyboard.is_key_pressed(self.reg[Instruction::param(inst.x)? as usize]) {
                    self.pc += 2;
                }
                Ok(())
            },
            // Skip if key is not pressed
            InstructionType::CondNotKey => {
                if !self.keyboard.is_key_pressed(self.reg[Instruction::param(inst.x)? as usize]) {
                    self.pc += 2;
                }
                Ok(())
            },
            // Store the value of the delay timer in a register
            InstructionType::DelTimrGet => {
                let dtime_access = match self.time.dtime.lock() {
                    Ok(guard) => guard,
                    Err(_) => return Err(()),
                };
                self.reg[Instruction::param(inst.x)? as usize] = *dtime_access;
                Ok(())
            },
            // Wait for a key press and store key
            InstructionType::WaitKey => {
                let key = self.keyboard.wait_for_key()?;
                self.reg[Instruction::param(inst.x)? as usize] = key;                

                Ok(())
            },
            // Set the delay timer
            InstructionType::DelTimrSet => {
                let mut dtime_access = match self.time.dtime.lock() {
                    Ok(guard) => guard,
                    Err(_) => return Err(()),
                };
                *dtime_access = self.reg[Instruction::param(inst.x)? as usize];
                Ok(())
            },
            // Set the delay timer
            InstructionType::SndTimrSet => {
                let mut stime_access = match self.time.stime.lock() {
                    Ok(guard) => guard,
                    Err(_) => return Err(()),
                };
                *stime_access = self.reg[Instruction::param(inst.x)? as usize];
                Ok(())
            },
            // Add register to pointer (no status register change)
            InstructionType::PointAdd => {
                self.i += self.reg[Instruction::param(inst.x)? as usize] as u16;
                Ok(())
            },
            // Set the pointer to the location of a character
            InstructionType::PointChar => {
                self.i = DEFAULT_FONTPACK_LOCATION as u16 + self.reg[Instruction::param(inst.x)? as usize] as u16 * 5;
                Ok(())
            },
            // Store the BCD representation of register at I
            InstructionType::BCDStore => {
                let mut x = self.reg[Instruction::param(inst.x)? as usize];
                const BASE: u8 = 10;

                for i in 0u16..=2 {
                    self.ram[self.i+2-i] = x % BASE;
                    x /= BASE;
                }

                Ok(())
            },
            // Saves register to memory
            InstructionType::RegDmp => {
                let x = Instruction::param(inst.x)? as u16;
                for reg_id in 0x0..=x {
                    self.ram[self.i+reg_id] = self.reg[reg_id as usize];
                }

                Ok(())
            },
            // Loads regsiter from memory
            InstructionType::RegLoad => {
                let x = Instruction::param(inst.x)? as u16;
                for reg_id in 0x0..=x {
                    self.reg[reg_id as usize] = self.ram[self.i+reg_id];
                }

                Ok(())
            },
            // No operation
            InstructionType::StopExecution => {
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