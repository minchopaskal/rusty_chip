// Chip-8's display
pub const DISPLAY_WIDTH : u32 = 64;
pub const DISPLAY_HEIGHT : u32 = 32;

// Our display
pub(crate) const PIXEL_SIZE : u32 = 10;
pub const WIDTH :u32 = DISPLAY_WIDTH * PIXEL_SIZE;
pub const HEIGHT : u32 = DISPLAY_HEIGHT * PIXEL_SIZE;
