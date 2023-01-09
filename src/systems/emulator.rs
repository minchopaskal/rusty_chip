use bevy::prelude::{Res, ResMut};
use bevy::time::Time;
use bevy_pixels::PixelsResource;

use crate::config::*;
use crate::resources::chip8::*;

pub fn emulator_system(mut pixels_resource: ResMut<PixelsResource>, mut chip8_resource: ResMut<Chip8>, time: Res<Time>) {
    let frame = pixels_resource.pixels.get_frame_mut();
    
    chip8_resource.as_mut().step(time.delta());
    
    let framebuffer = chip8_resource.display();

    for i in 0..HEIGHT {
        for j in 0..WIDTH {
            let x = j / PIXEL_SIZE;
            let y = i / PIXEL_SIZE;
            let idx : usize = (y * DISPLAY_WIDTH + x) as usize;

            // TODO: try to draw circles instead of rectangle pixels.
            // Latter check draws a grid.
            let color: [u8; 4] = if framebuffer[idx].0>0 && i%PIXEL_SIZE>0 && j%PIXEL_SIZE>0 {
                [255, 255, 255, 255]
            } else {
                [0, 0, 0, 255]
            };

            let idx = ((i * WIDTH + j) * 4) as usize;
            frame[idx..idx+4].copy_from_slice(&color);
        }
    }
}