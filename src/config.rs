use std::ops::Range;

// Chip-8's Config
// Display
pub const DISPLAY_WIDTH : u32 = 64;
pub const DISPLAY_HEIGHT : u32 = 32;
pub const PIXEL_SIZE : u32 = 10;
// Memory sizes
pub const RAM_SIZE : usize = 4096;
pub const STACK_SIZE : usize = 16;
pub const REGISTER_COUNT : usize = 16;
// Input
pub const NUM_KEYS : usize = 16;

// Our display
pub const WIDTH :u32 = 1366; // DISPLAY_WIDTH * PIXEL_SIZE;
pub const HEIGHT : u32 = 768; // DISPLAY_HEIGHT * PIXEL_SIZE;

// Emulation clock
pub const DELTA_S : f64 = 1.0 / 2000.0;

// Starting programm address in Chip8's RAM.
pub const START_PC : usize = 0x200;

// Address range for Chip8's font
pub const FONT_RANGE : Range<usize> = 0x50..0xA0;