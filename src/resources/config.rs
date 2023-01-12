use bevy::prelude::Resource;

/// Various configurations for the emulation.
/// 
/// `debug` is set byt the command line argument `debug`
/// `show_grid` draws a grid over the pixels. Only enabled when `circle_pixels` is false
/// `trace` leaves a trace after a pixel is erased. This is one way to reduce flicker.
/// `circle_pixels` draws CHIP-8's pixels as circles.
/// `reduce_flicker` tries to reduce the flicker by not updating the screen if a sprite was just erased.
/// 
/// @note That `reduce_flicker` and `trace` do not work together.
#[derive(Resource)]
pub struct ConfigResource {
    pub debug_ui: bool,
    pub show_grid: bool,
    pub trace: bool,
    pub circle_pixels: bool,
    pub reduce_flicker: bool,
}