use std::cmp;
use std::ops::RangeInclusive;
use std::time::Duration;
use rand::{thread_rng, Rng};

use bevy::prelude::Resource;
use bevy::time::{Timer, TimerMode};

use crate::config::*;

#[derive(Default, Clone, Copy)]
pub struct DisplayPixel(pub u8);

const FONT : [u8; 5 * 16] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];
const FONT_RANGE : RangeInclusive<usize> = 0x50..=0x9F;

const NUM_PIXELS : usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize;

#[derive(PartialEq)]
enum ConsoleState {
    Paused,
    Running,
}

#[derive(PartialEq, Clone, Copy)]
pub enum KeyState {
    Released,
    Pressed,
    JustReleased,
}

#[derive(Resource)]
pub struct Chip8 {
    ram: [u8; RAM_SIZE],
    stack: [u16; STACK_SIZE],
    framebuffer: [DisplayPixel; NUM_PIXELS],
    pc: u16,
    index_register : u16,
    stack_ptr: usize,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16],

    timer_clock: Timer,
    timer_60hz: Timer,

    state: ConsoleState,
    rom_size: usize,

    pub input: [KeyState; NUM_KEYS],
    pub clock_hz: u64,
    pub super_chip: bool,
    pub debug: bool,
}

// PRIVATE
impl Chip8 {
    fn fetch(&mut self) -> u16 {
        let fst = self.ram[self.pc as usize] as u16;
        self.pc += 1;
        let snd = self.ram[self.pc as usize] as u16;
        self.pc += 1;
        
        (fst << 8) | snd
    }

    fn display(&mut self, x: u16, y: u16, n: u16) {
        // Assume DISPLAY_* are powers of 2. 
        // Eqiv. to (X % DISPLAY_*)
        let x = (self.registers[x as usize] & (DISPLAY_WIDTH-1) as u8) as usize;
        let y = (self.registers[y as usize] & (DISPLAY_HEIGHT-1) as u8) as usize;
        let rows = n as usize; 
        self.registers[0xF] = 0;
        for i in 0..rows {
            let row = self.ram[self.index_register as usize + i];
            for j in 0..8 {
                let color = if (row & (1 << ( 8 - j - 1))) > 0 { 1 } else { 0 };
                let idx = cmp::min(y + i, DISPLAY_HEIGHT as usize - 1) * DISPLAY_WIDTH as usize + cmp::min(x + j, DISPLAY_WIDTH as usize - 1);
                let current = self.framebuffer[idx].0;
                self.framebuffer[idx].0 ^= color;

                if current > 0 && color == 0 {
                    self.registers[0xF] = 1;
                }
            }
        }
    }

    fn execute(&mut self, instr: u16) {
        let itype = (instr & 0xF000) >> 12;
        let x = (instr & 0x0F00) >> 8;
        let y = (instr & 0x00F0) >> 4;
        let b4 = instr & 0x000F;
        let b8 = instr & 0x00FF;
        let b12 = instr & 0x0FFF;

        if self.debug {
            println!("Execute: 0x{:04x}", instr);
        }
        
        match itype {
            0 => {
                if b8 == 0xE0 {
                    self.framebuffer.fill(DisplayPixel::default());
                } else if b12 == 0x0EE {
                    let stack_top = self.stack[self.stack_ptr-1];
                    self.stack_ptr -= 1;
                    self.pc = stack_top;
                } else {
                    panic!("Unsupported instruction 0x{:04x}!", instr);
                }
            },
            1 => {
                self.pc = b12 as u16;
            },
            2 => {
                self.stack[self.stack_ptr] = self.pc;
                self.stack_ptr += 1;
                self.pc = b12;
            },
            3 => {
                if self.registers[x as usize] == b8 as u8 {
                    self.pc += 2;
                }
            },
            4 => {
                if self.registers[x as usize] != b8 as u8 {
                    self.pc += 2;
                }
            },
            5 => {
                assert!(b4 == 0);
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.pc += 2;
                }
            },
            6 => {
                self.registers[x as usize] = b8 as u8;
            },
            7 => { // Add
                // We cast to u16 to prevent overflow.
                let vx = self.registers[x as usize] as u16;
                let res = vx + b8;
                self.registers[x as usize] = res as u8;
            },
            8 => {
                match b4 {
                    0 => {
                        self.registers[x as usize] = self.registers[y as usize];
                    },
                    1 => {
                        self.registers[x as usize] = self.registers[x as usize] | self.registers[y as usize];
                    },
                    2 => {
                        self.registers[x as usize] = self.registers[x as usize] & self.registers[y as usize];
                    },
                    3 => {
                        self.registers[x as usize] = self.registers[x as usize] ^ self.registers[y as usize];
                    },
                    4 => {
                        let vx = self.registers[x as usize] as u16;
                        let vy =  self.registers[y as usize] as u16;
                        let res = vx + vy;
                        self.registers[x as usize] = res as u8;
                        self.registers[0xF] = if res > 255 { 1 } else { 0 }
                    },
                    5 => {
                        let vx = self.registers[x as usize] as i16;
                        let vy =  self.registers[y as usize] as i16;
                        self.registers[x as usize] = (vx - vy) as u8;
                        self.registers[0xF] = if vx > vy { 1 } else { 0 }
                    },
                    6 => {
                        if !self.super_chip {
                            self.registers[x as usize] = self.registers[y as usize];
                        }
                        self.registers[0xF] = self.registers[x as usize] & 1;
                        self.registers[x as usize] >>= 1;
                    },
                    7 => {
                        let vx = self.registers[x as usize] as i16;
                        let vy =  self.registers[y as usize] as i16;
                        self.registers[x as usize] = (vy - vx) as u8;
                        self.registers[0xF] = if vy > vx { 1 } else { 0 }
                    },
                    0xE => {
                        if !self.super_chip {
                            self.registers[x as usize] = self.registers[y as usize];
                        }
                        self.registers[0xF] = self.registers[x as usize] & 0x80;
                        self.registers[x as usize] <<= 1;
                    },
                    _ => {
                        panic!("Unrecognised instruction: 0x{:04x}", instr);
                    }
                }
            },
            9 => {
                assert!(b4 == 0);
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.pc += 2;
                }
            },
            0xA => {
                self.index_register = b12 as u16;
            },
            0xB => {
                if self.super_chip {
                    self.pc = self.registers[x as usize] as u16 + b12;
                } else {
                    self.pc = self.registers[0] as u16 + b12;
                }
            },
            0xC => {
                let num : u8 = thread_rng().gen();
                self.registers[x as usize] = num & b8 as u8;
            }
            0xD => {
                self.display(x, y, b4);
            },
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
            },
            0xF => {
                match b8 {
                    0x07 => {
                        self.registers[x as usize] = self.delay_timer;
                    },
                    0x15 => {
                        self.delay_timer = self.registers[x as usize];
                    },
                    0x18 => {
                        self.sound_timer = self.registers[x as usize];
                    },
                    0x1E => {
                        self.index_register += self.registers[x as usize] as u16;
                        // "overflow" outside of addressing range
                        if self.index_register >= 4096 {
                            self.registers[0xF] = 1;
                        }
                    },
                    0x0A => { // Get Key
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
                    },
                    0x29 => { // Font character
                        let char = (self.registers[x as usize] & 0xF) as usize;
                        assert!(char <= 0xF);
                        // Each character sprite is represented by 5 bytes.
                        let addr = self.ram[FONT_RANGE.start() + 5 * char] as u16;
                        self.index_register = addr;
                    },
                    0x33 => {
                        let mut num = self.registers[x as usize];
                        if num == 0 {
                            self.ram[self.index_register as usize] = 0;
                        } else {
                            if num < 10 {
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
                    },
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
                    },
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
            },
        }

        // Clear out the keyboard state if the 0xFx0A instruction was
        // not called. This will prevent it from catching old input.
        for i in 0..NUM_KEYS {
            if self.input[i] == KeyState::JustReleased {
                self.input[i] = KeyState::Released;
            }
        }

    }
}

const SECOND_IN_NS : u64 = 1000000000;

// PUBLIC
pub struct StepResult {
    pub drawn: bool, // weather or not we executed a draw instruction
    pub beep: bool, // weather or not sound_timer > 0
}

const START_PC : usize = 512;
impl Chip8 {
    pub fn new(data: &Vec<u8>, clock_hz : u64) -> Chip8 {

        let mut res = Chip8 {
            ram : [0; 4096],
            stack: [0; 16],
            framebuffer : [DisplayPixel::default(); NUM_PIXELS],
            pc: START_PC as u16,
            index_register: 0,
            stack_ptr: 0,
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            clock_hz: clock_hz,
            timer_clock: Timer::new(Duration::from_nanos(SECOND_IN_NS / clock_hz as u64), TimerMode::Repeating),
            timer_60hz: Timer::new(Duration::from_nanos(SECOND_IN_NS / 60), TimerMode::Repeating),
            super_chip: true,
            input: [KeyState::Released; 16],
            rom_size: 0,
            debug: false,

            #[cfg(debug_assertions)]
            state: ConsoleState::Paused,
            #[cfg(not(debug_assertions))]
            state: ConsoleState::Running,
        };

        // Copy font into memory 050–09F
        res.ram[FONT_RANGE].copy_from_slice(&FONT);
        
        // Copy program data into memory
        res.ram[START_PC..(START_PC + data.len())].copy_from_slice(&data);

        res.rom_size = data.len();

        res
    }

    pub fn framebuffer(&self) -> &[DisplayPixel; NUM_PIXELS] {
        &self.framebuffer
    }

    pub fn ram(&self) -> &[u8; RAM_SIZE] {
        &self.ram
    }

    pub fn stack(&self) -> &[u16; STACK_SIZE] {
        &self.stack
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn rom_sz(&self) -> usize {
        self.rom_size
    }

    pub fn paused(&self) -> bool {
        self.state == ConsoleState::Paused
    }

    pub fn pause(&mut self) {
        self.state = ConsoleState::Paused;
    }

    pub fn run(&mut self) {
        self.state = ConsoleState::Running;
    }

    pub fn step(&mut self, delta: Duration) -> StepResult {
        self.timer_clock.tick(delta);
        self.timer_60hz.tick(delta);

        let mut drawn = false;
        if self.state == ConsoleState::Paused || self.timer_clock.just_finished() {
            let instr = self.fetch();
            if instr & 0xF000 == 0xD000 {
                drawn = true;
            }
            self.execute(instr);
        }

        if self.state == ConsoleState::Paused || self.timer_60hz.just_finished() {
            if self.delay_timer > 0 { self.delay_timer -= 1; }
            if self.sound_timer > 0 { self.sound_timer -= 1; }
        }

        // Don't play sound when paused as it might be unpleasant.
        StepResult { drawn: drawn, beep: self.state == ConsoleState::Running && self.sound_timer > 0 }
    }

    pub fn change_clock(&mut self, clock_hz: u64) {
        if clock_hz == self.clock_hz {
            return;
        }

        self.clock_hz = clock_hz;
        self.timer_clock = Timer::new(Duration::from_nanos(SECOND_IN_NS/clock_hz), TimerMode::Repeating);
    }
}

#[derive(Resource)]
pub struct PlayingSound(pub bool);
