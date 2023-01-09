use std::time::Duration;

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

const NUM_PIXELS : usize = (DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize;

#[derive(PartialEq)]
enum State {
    Paused,
    Running,
}

#[derive(Resource)]
pub struct Chip8 {
    ram: [u8; 4096],
    stack: [u16; 16],
    display: [DisplayPixel; NUM_PIXELS],
    pc: u16,
    index_register : u16,
    stack_ptr: usize,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16],

    timer_clock: Timer,
    timer_60hz: Timer,

    state: State,

    pub clock_hz: u64,
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

    fn execute(&mut self, instr: u16) {
        let itype = (instr & 0xF000) >> 12;
        let x = (instr & 0x0F00) >> 8;
        let y = (instr & 0x00F0) >> 4;
        let b4 = instr & 0x000F;
        let b8 = instr & 0x00FF;
        let b12 = instr & 0x0FFF;

        println!("Execute: 0x{:04x}", instr);
        
        match itype {
            0 => {
                if b8 == 0xE0 {
                    self.display.fill(DisplayPixel::default());
                } else if b8 == 0xEE {
                    let stack_top = self.stack[self.stack_ptr];
                    self.stack_ptr -= 1;
                    self.pc = stack_top;
                } else {
                    panic!("Unrecognised instruction: 0x{:04x}", instr);
                }
            },
            1 => {
                self.pc = b12 as u16;
            },
            6 => {
                self.registers[x as usize] = b8 as u8;
            }
            7 => {
                self.registers[x as usize] += b8 as u8;
            }
            0xA => {
                self.index_register = b12 as u16;
            }
            0xD => {
                // Assume DISPLAY_* are powers of 2. 
                // Eqiv. to (X % DISPLAY_*)
                let x = (self.registers[x as usize] & (DISPLAY_WIDTH-1) as u8) as usize;
                let y = (self.registers[y as usize] & (DISPLAY_HEIGHT-1) as u8) as usize;
                let rows = b4 as usize; 
                self.registers[0xF] = 0;
                for i in 0..rows {
                    let row = self.ram[self.index_register as usize + i];
                    for j in 0..8 {
                        let color = if (row & (1 << ( 8 - j - 1))) > 0 { 1 } else { 0 };
                        let idx = (y + i) * DISPLAY_WIDTH as usize + (x + j);
                        let current = self.display[idx].0;
                        self.display[idx].0 ^= color;

                        if current > 0 && color == 0 {
                            self.registers[0xF] = 1;
                        }
                    }
                }
            }
            _ => {
                panic!("Unrecognised instruction: 0x{:04x}", instr);
            }
        }
    }
}

// PUBLIC
impl Chip8 {
    pub fn new(data: &Vec<u8>, clock_hz : u64) -> Chip8 {
        const START_OFFSET : usize = 512;
        const SECOND_IN_NS : u64 = 1000000000;

        let mut res = Chip8 {
            ram : [0; 4096],
            stack: [0; 16],
            display : [DisplayPixel::default(); NUM_PIXELS],
            pc: START_OFFSET as u16,
            index_register: 0,
            stack_ptr: 0,
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            clock_hz: clock_hz,
            timer_clock: Timer::new(Duration::from_nanos(SECOND_IN_NS / clock_hz as u64), TimerMode::Repeating),
            timer_60hz: Timer::new(Duration::from_nanos(SECOND_IN_NS / 60), TimerMode::Repeating),

            #[cfg(debug_assertions)]
            state: State::Paused,

            #[cfg(not(debug_assertions))]
            state: State::Running,
        };

        // TODO: copy font into memory 050–09F
        
        // Copy program data into memory
        res.ram[START_OFFSET..(START_OFFSET + data.len())].copy_from_slice(&data);

        res
    }

    pub fn display(&self) -> &[DisplayPixel; NUM_PIXELS] {
        &self.display
    }

    pub fn paused(&self) -> bool {
        self.state == State::Paused
    }

    pub fn pause(&mut self) {
        self.state = State::Paused;
    }

    pub fn run(&mut self) {
        self.state = State::Running;
    }

    pub fn step(&mut self, delta: Duration) -> bool {
        self.timer_clock.tick(delta);
        self.timer_60hz.tick(delta);

        if self.state == State::Paused || self.timer_clock.just_finished() {
            let instr = self.fetch();
            self.execute(instr);
        }

        // TODO: this is not entirely correct. Pausing 
        if self.state == State::Paused || self.timer_60hz.just_finished() {
            if self.delay_timer > 0 { self.delay_timer -= 1; }
            if self.sound_timer > 0 { self.sound_timer -= 1; }
        }

        // Return weather we should play a sound.
        // Don't play sound when paused as it might be unpleasant.
        self.state == State::Running && self.sound_timer > 0
    }

    pub fn change_clock(&mut self, clock_hz: u64) {
        if clock_hz == self.clock_hz {
            return;
        }

        self.clock_hz = clock_hz;
        self.timer_clock = Timer::new(Duration::from_nanos(1000000000/clock_hz), TimerMode::Repeating);
    }
}
