use std::time::Duration;

#[allow(unused_imports)]
use bevy::diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::time::FixedTimestep;
use bevy::{prelude::*, window::WindowResizeConstraints};
use bevy_egui::EguiPlugin;
use bevy_pixel_buffer::prelude::*;
use config::{DISPLAY_WIDTH, PIXEL_SIZE, DISPLAY_HEIGHT, WIDTH, HEIGHT, DELTA_S};
use resources::beep::BeepResource;
use resources::chip8::Chip8;
use resources::config::ConfigResource;
use resources::timer::DrawTimer;
use systems::{audio_setup, keyboard, ui, emulator};

mod config;
mod resources;
mod systems;

fn main() -> std::io::Result<()> {
    let args : Vec<String> = std::env::args().collect();
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
            window: WindowDescriptor {
                title: "Chip-8 Rust Emulator".to_string(),
                width: WIDTH as f32,
                height: HEIGHT as f32,
                resize_constraints: WindowResizeConstraints {
                    min_width: WIDTH as f32,
                    min_height: HEIGHT as f32,
                    ..default()
                },
                fit_canvas_to_parent: true,
                ..default()
            },
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .insert_resource(Chip8::new(600, debug))
        .insert_resource(BeepResource::default())
        .insert_resource(ConfigResource {
            debug_ui: debug,
            show_grid: false,
            trace: false,
            circle_pixels: false,
            reduce_flicker: false,
        })
        .insert_resource(DrawTimer { timer: Timer::new(Duration::from_secs_f64(1.0/120.0), TimerMode::Repeating)})
        .add_plugins(PixelBufferPlugins)
        .add_startup_system(
            PixelBufferBuilder::new()
                .with_size(pixel_buffer_size)
                .with_render(false)
                .setup()
        )
        .add_startup_system(audio_setup::setup_audio_system)
        .add_system(keyboard::keyboard_system)
        .add_system(ui::ui_system.label("ui"))
        .add_system_set(
            SystemSet::new()
                // We run the emulation at max 2000Hz. Actual clock speed is controlled by UI.
                .before("ui")
                .with_run_criteria(FixedTimestep::step(DELTA_S))
                .with_system(emulator::emulator_system),
        )
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();

    Ok(())
}