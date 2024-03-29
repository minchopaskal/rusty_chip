use std::time::Duration;

use bevy::prelude::*;

use bevy_pixel_buffer::prelude::*;
use rayon::prelude::*;

use crate::{
    config::{DELTA_S, DISPLAY_WIDTH, PIXEL_SIZE},
    resources::{
        chip8::{Chip8, StepResult},
        config::ConfigResource,
        timer::DrawTimer,
    },
    systems::audio::Beep,
};

/// Simple 10x10 matrix representing a circular pixel.
/// More efficient than calculating it.
const CIRCLE_MATRIX: [[u8; PIXEL_SIZE as usize]; PIXEL_SIZE as usize] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 1, 1, 1, 0, 0, 0],
    [0, 0, 1, 1, 1, 1, 1, 1, 0, 0],
    [0, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    [0, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    [0, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    [0, 1, 1, 1, 1, 1, 1, 1, 1, 0],
    [0, 0, 1, 1, 1, 1, 1, 1, 0, 0],
    [0, 0, 0, 1, 1, 1, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

/// Step through chip-8's instructions and display the result.
///
/// Drawing happens at 60FPS, unless the `ConfigResouce::reduce_flicker`
/// option is enabled. The latter case updates the framebuffer only when
/// a new change happens.
///
/// The emulator system is called `CHIP8_CPU_MAX_CLOCK_HZ` times per second,
/// where as the `Chip8::step` function is only called at the specified by
/// the user clock.
pub fn emulator_system(
    mut pb: QueryPixelBuffer,
    mut chip8_resource: ResMut<Chip8>,
    mut timer_resource: ResMut<DrawTimer>,
    cfg: ResMut<ConfigResource>,
    beep: Query<&AudioSink, With<Beep>>,
) {
    let delta = Duration::from_secs_f64(DELTA_S);

    let mut res = StepResult {
        drawn: false,
        beep: false,
    };

    if !chip8_resource.paused() {
        res = chip8_resource.as_mut().step(delta);
    }

    if res.beep {
        beep.single().play();
    }

    let force_draw = res.drawn || chip8_resource.is_reset();
    if !force_draw && !timer_resource.timer.tick(delta).finished() {
        return;
    }

    if cfg.reduce_flicker && !force_draw {
        return;
    }

    let framebuffer = chip8_resource.framebuffer();

    pb.frame().per_pixel_par(|coord, _| {
        let x = coord.x / PIXEL_SIZE;
        let y = coord.y / PIXEL_SIZE;
        let idx: usize = (y * DISPLAY_WIDTH + x) as usize;

        let is_grid = cfg.show_grid && ((x * PIXEL_SIZE == coord.x) || (y * PIXEL_SIZE == coord.y));
        let outside_circle = cfg.circle_pixels
            && CIRCLE_MATRIX[(coord.y - y * PIXEL_SIZE) as usize]
                [(coord.x - x * PIXEL_SIZE) as usize]
                == 0;

        if framebuffer[idx].0 > 0 && !is_grid && !outside_circle {
            let c = framebuffer[idx].0;
            if cfg.trace && c < 255 {
                Pixel {
                    r: c,
                    g: c,
                    b: c,
                    a: c,
                }
            } else if c == 255 {
                Pixel::WHITE
            } else {
                Pixel::BLACK
            }
        } else {
            Pixel::BLACK
        }
    });

    if cfg.trace {
        chip8_resource
            .framebuffer_mut()
            .par_iter_mut()
            .for_each(|c| {
                if c.0 > 0 && c.0 < 255 {
                    c.0 -= std::cmp::min(c.0, 5);
                }
            });
    }
}
