use std::time::Duration;

use bevy::prelude::*;

use bevy_pixel_buffer::prelude::*;

use crate::config::*;
use crate::resources::beep::BeepResource;
use crate::resources::chip8::*;

pub fn emulator_system(mut pb: QueryPixelBuffer, mut chip8_resource: ResMut<Chip8>, audio: Res<Audio>, beep_res: Res<BeepResource>) {
    let mut res = StepResult{ drawn: false, beep: false };
    
    if !chip8_resource.paused() {
        res = chip8_resource.as_mut().step(Duration::from_secs_f64(DELTA_S));
    }

    if res.beep {
        let sound = beep_res.sound.clone();
        audio.play(*sound);
    }

    if !res.drawn && !chip8_resource.is_reset() {
        return;
    }
 
    let framebuffer = chip8_resource.framebuffer();

    pb.frame().per_pixel_par(|coord, _| {
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