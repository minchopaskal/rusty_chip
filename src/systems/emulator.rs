use bevy::prelude::{Res, ResMut};
use bevy::time::Time;
use bevy_pixel_buffer::prelude::*;

use crate::config::*;
use crate::resources::chip8::*;

pub fn emulator_system(mut pb: QueryPixelBuffer, mut chip8_resource: ResMut<Chip8>, time: Res<Time>) {
    if !chip8_resource.paused() {
        chip8_resource.as_mut().step(time.delta());
    }

    let framebuffer = chip8_resource.display();

    pb.frame().per_pixel(|coord, _| {
        let x = coord.x / PIXEL_SIZE;
        let y = coord.y / PIXEL_SIZE;
        let idx : usize = (y * DISPLAY_WIDTH + x) as usize;

        let is_grid = coord.x % PIXEL_SIZE == 0 || coord.y % PIXEL_SIZE == 0;
        
        // TODO: try to draw circles instead of rectangle pixels.
        if framebuffer[idx].0>0  && !is_grid {
            Pixel::WHITE
        } else {
            Pixel::BLACK
        }
    });
}