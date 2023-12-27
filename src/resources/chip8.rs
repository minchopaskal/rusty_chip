use rand::{thread_rng, Rng};
use std::cmp;
use std::time::Duration;

use bevy::prelude::Resource;
use bevy::time::{Timer, TimerMode};

use crate::config::{
    DISPLAY_HEIGHT, DISPLAY_WIDTH, FONT, FONT_RANGE, NUM_KEYS, RAM_SIZE, REGISTER_COUNT,
    STACK_SIZE, START_PC,
};

/// CHIP-8 display pixel's representation.
#[derive(Default, Clone, Copy)]
pub struct DisplayPixel(pub u8);

const NUM_PIXELS: usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize;

/// CHIP-8 state. If paused the user can step through
/// the instructions one by one.
#[derive(PartialEq)]
enum ConsoleState {
    Paused,
    Running,
}

/// CHIP-8 key state.
///
/// There is a peculiarity in the instruction FX0A(Get key).
/// It busy waits until a key is pressed, but only detects it
/// after a key was pressed and then released.
/// Thus we need the JustReleased state.
#[derive(PartialEq, Clone, Copy)]
pub enum KeyState {
    Released,
    Pressed,
    JustReleased,
}

/// CHIP-8's state.
///
/// Nothing fancy. Most of it you find on every CHIP-8
/// emulator tutorial. Added stuff is for user-friendliness.
#[derive(Resource)]
pub struct Chip8 {
    ram: [u8; RAM_SIZE],
    stack: [u16; STACK_SIZE],
    framebuffer: [DisplayPixel; NUM_PIXELS],
    pc: u16,
    index_register: u16,
    stack_ptr: usize,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16],

    timer_clock: Timer,
    timer_60hz: Timer,

    state: ConsoleState,
    rom_size: usize,
    reset: bool,
    debug: bool,
    trace: bool,
    reduce_flicker: bool,

    pub input: [KeyState; NUM_KEYS],
    pub clock_hz: u64,
    pub super_chip: bool,
}

impl Chip8 {
    /// Read the next instruction and increase the program counter.
    fn fetch(&mut self) -> u16 {
        let fst = self.ram[self.pc as usize] as u16;
        self.pc += 1;
        let snd = self.ram[self.pc as usize] as u16;
        self.pc += 1;

        (fst << 8) | snd
    }

    /// Draw the sprite specified by the instruction.
    ///
    /// # Returns true if the display should be updated. False, otherwise.
    ///
    /// If `self.reduce_flicker` is true it checks if we are just erasing a sprite
    /// (i.e all the pixels that are changed were 1 to 0 flips) and if that is the case
    /// we don't update the display.
    /// `self.reduce_flicker` being false means we always want to update the display after
    /// the instruction was called.
    fn display(&mut self, x: u16, y: u16, n: u16) -> bool {
        // Assume DISPLAY_* are powers of 2.
        // Eqiv. to (X % DISPLAY_*)
        let x = (self.registers[x as usize] & (DISPLAY_WIDTH - 1) as u8) as usize;
        let y = (self.registers[y as usize] & (DISPLAY_HEIGHT - 1) as u8) as usize;
        let rows = n as usize;
        self.registers[0xF] = 0;
        let mut drawn = false;
        for i in 0..rows {
            if y + i >= DISPLAY_HEIGHT as usize {
                break;
            }

            let row = self.ram[self.index_register as usize + i];

            for j in 0..8 {
                let color = if (row & (0x80 >> j)) > 0 { 1 } else { 0 };
                let idx =
                    (y + i) * DISPLAY_WIDTH as usize + cmp::min(x + j, DISPLAY_WIDTH as usize - 1);
                let old = self.framebuffer[idx].0;

                if old == 255 {
                    self.framebuffer[idx].0 = if color == 0 {
                        255
                    } else if self.trace {
                        128
                    } else {
                        0
                    }
                } else if color == 1 {
                    self.framebuffer[idx].0 = 255;
                    drawn = true;
                }

                if old == 255 && self.framebuffer[idx].0 < 255 {
                    self.registers[0xF] = 1;
                }
            }
        }

        !self.reduce_flicker || drawn
    }

    /// Parse an instruction.
    fn execute(&mut self, instr: u16) -> bool {
        let itype = (instr & 0xF000) >> 12;
        let x = (instr & 0x0F00) >> 8;
        let y = (instr & 0x00F0) >> 4;
        let b4 = instr & 0x000F;
        let b8 = instr & 0x00FF;
        let b12 = instr & 0x0FFF;

        if self.debug {
            println!("Execute: 0x{:04x}", instr);
        }

        let mut drawn = false;
        match itype {
            0 => {
                if b8 == 0xE0 {
                    self.framebuffer.fill(DisplayPixel::default());
                } else if b12 == 0x0EE {
                    self.stack_ptr -= 1;
                    self.pc = self.stack[self.stack_ptr];
                    self.stack[self.stack_ptr] = 0;
                } else {
                    panic!("Unsupported instruction 0x{:04x}!", instr);
                }
            }
            1 => {
                self.pc = b12;
            }
            2 => {
                self.stack[self.stack_ptr] = self.pc;
                self.stack_ptr += 1;
                self.pc = b12;
            }
            3 => {
                if self.registers[x as usize] == b8 as u8 {
                    self.pc += 2;
                }
            }
            4 => {
                if self.registers[x as usize] != b8 as u8 {
                    self.pc += 2;
                }
            }
            5 => {
                assert!(b4 == 0);
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.pc += 2;
                }
            }
            6 => {
                self.registers[x as usize] = b8 as u8;
            }
            7 => {
                // Add
                // We cast to u16 to prevent overflow.
                let vx = self.registers[x as usize] as u16;
                let res = vx + b8;
                self.registers[x as usize] = res as u8;
            }
            8 => match b4 {
                0 => {
                    self.registers[x as usize] = self.registers[y as usize];
                }
                1 => {
                    self.registers[x as usize] |= self.registers[y as usize];
                    if !self.super_chip {
                        self.registers[0xF] = 0;
                    }
                }
                2 => {
                    self.registers[x as usize] &= self.registers[y as usize];
                    if !self.super_chip {
                        self.registers[0xF] = 0;
                    }
                }
                3 => {
                    self.registers[x as usize] ^= self.registers[y as usize];
                    if !self.super_chip {
                        self.registers[0xF] = 0;
                    }
                }
                4 => {
                    let vx = self.registers[x as usize] as u16;
                    let vy = self.registers[y as usize] as u16;
                    let res = vx + vy;
                    self.registers[x as usize] = res as u8;
                    self.registers[0xF] = if res > 255 { 1 } else { 0 }
                }
                5 => {
                    let vx = self.registers[x as usize] as i16;
                    let vy = self.registers[y as usize] as i16;
                    self.registers[x as usize] = (vx - vy) as u8;
                    self.registers[0xF] = if vx > vy { 1 } else { 0 }
                }
                6 => {
                    if !self.super_chip {
                        self.registers[x as usize] = self.registers[y as usize];
                    }
                    let vf = self.registers[x as usize] & 0x01;
                    self.registers[x as usize] >>= 1;
                    self.registers[0xF] = vf;
                }
                7 => {
                    let vx = self.registers[x as usize] as i16;
                    let vy = self.registers[y as usize] as i16;
                    self.registers[x as usize] = (vy - vx) as u8;
                    self.registers[0xF] = if vy > vx { 1 } else { 0 }
                }
                0xE => {
                    if !self.super_chip {
                        self.registers[x as usize] = self.registers[y as usize];
                    }
                    let vf = (self.registers[x as usize] & 0x80) >> 7;
                    self.registers[x as usize] <<= 1;
                    self.registers[0xF] = vf;
                }
                _ => {
                    panic!("Unrecognised instruction: 0x{:04x}", instr);
                }
            },
            9 => {
                assert!(b4 == 0);
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.pc += 2;
                }
            }
            0xA => {
                self.index_register = b12;
            }
            0xB => {
                if self.super_chip {
                    self.pc = self.registers[x as usize] as u16 + b12;
                } else {
                    self.pc = self.registers[0] as u16 + b12;
                }
            }
            0xC => {
                let num: u8 = thread_rng().gen();
                self.registers[x as usize] = num & b8 as u8;
            }
            0xD => {
                drawn = self.display(x, y, b4);
            }
            0xE => {
                if b8 == 0x9E {
                    if self.input[self.registers[x as usize] as usize] == KeyState::Pressed {
                        self.pc += 2;
                    }
                } else if b8 == 0xA1 {
                    if self.input[self.registers[x as usize] as usize] != KeyState::Pressed {
                        self.pc += 2;
                    }
                } else {
                    panic!("Unrecognised instruction: 0x{:04x}", instr);
                }
            }
            0xF => {
                match b8 {
                    0x07 => {
                        self.registers[x as usize] = self.delay_timer;
                    }
                    0x15 => {
                        self.delay_timer = self.registers[x as usize];
                    }
                    0x18 => {
                        self.sound_timer = self.registers[x as usize];
                    }
                    0x1E => {
                        self.index_register += self.registers[x as usize] as u16;
                        // "overflow" outside of addressing range
                        if self.index_register >= 4096 {
                            self.registers[0xF] = 1;
                        }
                    }
                    0x0A => {
                        // Get Key
                        let mut found = false;
                        for i in 0..16 {
                            if self.input[i] == KeyState::JustReleased {
                                self.input[i] = KeyState::Released;
                                self.registers[x as usize] = i as u8;
                                found = true;
                            }
                        }
                        if !found {
                            self.pc -= 2;
                        }
                    }
                    0x29 => {
                        // Font character
                        let char = (self.registers[x as usize] & 0xF) as usize;
                        assert!(char <= 0xF);
                        // Each character sprite is represented by 5 bytes.
                        let addr = self.ram[FONT_RANGE.start + 5 * char] as u16;
                        self.index_register = addr;
                    }
                    0x33 => {
                        let mut num = self.registers[x as usize];
                        if num == 0 {
                            self.ram[self.index_register as usize] = 0;
                        } else if num < 10 {
                            self.ram[self.index_register as usize] = num;
                        } else if num < 100 {
                            self.ram[self.index_register as usize + 1] = num % 10;
                            self.ram[self.index_register as usize] = num / 10;
                        } else {
                            self.ram[self.index_register as usize + 2] = num % 10;
                            num /= 10;
                            self.ram[self.index_register as usize + 1] = num % 10;
                            self.ram[self.index_register as usize] = num / 10;
                        }
                    }
                    0x55 => {
                        let mut local = self.index_register;
                        let &mut i;
                        if !self.super_chip {
                            i = &mut self.index_register;
                        } else {
                            i = &mut local;
                        }

                        for j in 0..=x as usize {
                            self.ram[*i as usize] = self.registers[j];
                            *i += 1;
                        }
                    }
                    0x65 => {
                        let mut local = self.index_register;
                        let &mut i;
                        if !self.super_chip {
                            i = &mut self.index_register;
                        } else {
                            i = &mut local;
                        }

                        for j in 0..=x as usize {
                            self.registers[j] = self.ram[*i as usize];
                            *i += 1;
                        }
                    }
                    _ => {
                        panic!("Unrecognised instruction: 0x{:04x}", instr);
                    }
                }
            }
            _ => {
                panic!("Unrecognised instruction: 0x{:04x}", instr);
            }
        }

        // Clear out the keyboard state if the 0xFx0A instruction was
        // not called. This will prevent it from catching old input.
        for i in 0..NUM_KEYS {
            if self.input[i] == KeyState::JustReleased {
                self.input[i] = KeyState::Released;
            }
        }

        drawn
    }
}

const SECOND_IN_NS: u64 = 1000000000;

/// Result of calling `Chip8::step()`
///
/// `drawn` means we should update the screen
/// `beep` means we should play the beep sound
pub struct StepResult {
    pub drawn: bool, // weather or not we executed a draw instruction
    pub beep: bool,  // weather or not sound_timer > 0
}

impl Chip8 {
    pub fn new(clock_hz: u64, debug: bool) -> Chip8 {
        let mut res = Chip8 {
            ram: [0; RAM_SIZE],
            stack: [0; STACK_SIZE],
            framebuffer: [DisplayPixel::default(); NUM_PIXELS],
            pc: START_PC as u16,
            index_register: 0,
            stack_ptr: 0,
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; REGISTER_COUNT],
            clock_hz,
            timer_clock: Timer::new(
                Duration::from_nanos(SECOND_IN_NS / clock_hz),
                TimerMode::Repeating,
            ),
            timer_60hz: Timer::new(
                Duration::from_nanos(SECOND_IN_NS / 60),
                TimerMode::Repeating,
            ),
            super_chip: true,
            input: [KeyState::Released; NUM_KEYS],
            rom_size: 0,
            reset: true,
            debug,
            trace: false,
            reduce_flicker: false,

            state: ConsoleState::Paused,
            //#[cfg(not(debug_assertions))]
        };

        // Copy font into memory 050–09F
        res.ram[FONT_RANGE].copy_from_slice(&FONT);

        res
    }

    /// Load a ROM into CHIP-8's RAM.
    pub fn insert_cartridge(&mut self, data: &Vec<u8>) {
        self.reset();

        // Copy program data into memory
        self.ram[START_PC..(START_PC + data.len())].copy_from_slice(data);

        self.rom_size = data.len();
    }

    /// Reset all the state. A new ROM should be loaded.
    pub fn reset(&mut self) {
        *self = Chip8::new(self.clock_hz, self.debug);
        self.reset = true;
    }

    pub fn is_reset(&mut self) -> bool {
        let res = self.reset;
        self.reset = false;
        res
    }

    pub fn framebuffer(&self) -> &[DisplayPixel; NUM_PIXELS] {
        &self.framebuffer
    }

    pub fn framebuffer_mut(&mut self) -> &mut [DisplayPixel; NUM_PIXELS] {
        &mut self.framebuffer
    }

    pub fn ram(&self) -> &[u8] {
        &self.ram
    }

    pub fn stack(&self) -> &[u16; STACK_SIZE] {
        &self.stack
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn sp(&self) -> usize {
        self.stack_ptr
    }

    pub fn rom_sz(&self) -> usize {
        self.rom_size
    }

    pub fn registers(&self) -> &[u8; REGISTER_COUNT] {
        &self.registers
    }

    pub fn paused(&self) -> bool {
        self.state == ConsoleState::Paused
    }

    pub fn set_trace(&mut self, trace: bool) {
        self.trace = trace;
    }

    pub fn set_reduce_flicker(&mut self, reduce: bool) {
        self.reduce_flicker = reduce;
    }

    pub fn pause(&mut self) {
        self.state = ConsoleState::Paused;
    }

    pub fn run(&mut self) {
        self.state = ConsoleState::Running;
    }

    /// Fetch, decode and execute the next instruction.
    ///
    /// Since we call this function more times than the cpu clock
    /// we have a timer to check if it's time to actually process
    /// the next instruction.
    pub fn step(&mut self, delta: Duration) -> StepResult {
        self.timer_clock.tick(delta);
        self.timer_60hz.tick(delta);

        let mut drawn = false;
        if self.state == ConsoleState::Paused || self.timer_clock.just_finished() {
            let instr = self.fetch();
            drawn = self.execute(instr);
        }

        if self.state == ConsoleState::Paused || self.timer_60hz.just_finished() {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
        }

        // Don't play sound when paused as it might be unpleasant.
        StepResult {
            drawn,
            beep: self.state == ConsoleState::Running && self.sound_timer > 0,
        }
    }

    /// Change the CPU clock.
    ///
    /// Some games may need a higher clock speed, others may be
    /// more playable at lower than the default.
    pub fn change_clock(&mut self, clock_hz: u64) {
        if clock_hz == self.clock_hz {
            return;
        }

        self.clock_hz = clock_hz;
        self.timer_clock = Timer::new(
            Duration::from_nanos(SECOND_IN_NS / clock_hz),
            TimerMode::Repeating,
        );
    }
}
