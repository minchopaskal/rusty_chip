use std::time::Duration;

use bevy::time::common_conditions::on_timer;
use bevy::{prelude::*, window::WindowResizeConstraints};
use bevy_egui::EguiPlugin;
use bevy_pixel_buffer::prelude::*;
use config::{DELTA_S, DISPLAY_HEIGHT, DISPLAY_WIDTH, HEIGHT, PIXEL_SIZE, WIDTH};
use resources::chip8::Chip8;
use resources::config::ConfigResource;
use resources::timer::DrawTimer;
use systems::{audio, emulator, keyboard, ui};

mod config;
mod resources;
mod systems;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut debug = false;
    if args.len() > 1 {
        if args[1] != "debug" {
            panic!("Uncrecognised command line argument: {}!", args[1]);
        }
        debug = true;
    }

    let pixel_buffer_size = PixelBufferSize {
        size: UVec2::new(DISPLAY_WIDTH * PIXEL_SIZE, DISPLAY_HEIGHT * PIXEL_SIZE),
        pixel_size: UVec2::new(1, 1),
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Chip-8 Rust Emulator".to_string(),
                resolution: (WIDTH as f32, HEIGHT as f32).into(),
                resize_constraints: WindowResizeConstraints {
                    min_width: WIDTH as f32,
                    min_height: HEIGHT as f32,
                    ..default()
                },
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .insert_resource(Chip8::new(600, debug))
        .insert_resource(ConfigResource {
            debug_ui: debug,
            show_grid: false,
            trace: false,
            circle_pixels: false,
            reduce_flicker: false,
        })
        .insert_resource(DrawTimer {
            timer: Timer::new(Duration::from_secs_f64(1.0 / 120.0), TimerMode::Repeating),
        })
        .add_plugins(PixelBufferPlugins)
        .add_systems(
            Startup,
            (
                PixelBufferBuilder::new()
                    .with_size(pixel_buffer_size)
                    .with_render(false)
                    .setup(),
                audio::setup_audio_system,
            ),
        )
        .add_systems(
            Update,
            (
                keyboard::keyboard_system,
                ui::ui_system.in_set(ui::UiSet),
                emulator::emulator_system
                    .before(ui::UiSet)
                    .run_if(on_timer(Duration::from_secs_f64(DELTA_S))),
            ),
        )
        .run();

    Ok(())
}
