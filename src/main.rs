use std::io::Read;

use bevy::{prelude::*, window::WindowResizeConstraints};
use bevy_egui::EguiPlugin;
use bevy_pixel_buffer::prelude::*;

mod config;
mod resources;
mod systems;

use crate::config::*;
use crate::resources::chip8::*;
use crate::systems::*;

fn main() -> std::io::Result<()> {
    let args : Vec<String> = std::env::args().collect();
    let filepath = &args[1];

    let f = std::fs::File::open(filepath)?;
    let len = f.metadata().unwrap().len();
    
    let mut data : Vec<u8> = Vec::new();
    data.resize(len as usize, 0);

    let mut file = std::io::BufReader::new(f);
    file.read(data.as_mut_slice()).expect("File couldn't be read successfully!");

    let pixel_buffer_size = PixelBufferSize {
        size: UVec2::new(WIDTH, HEIGHT),
        pixel_size: UVec2::new(1, 1),
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Hello Bevy Pixels".to_string(),
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
        .add_plugin(PixelBufferPlugin)
        .add_plugin(EguiPlugin)
        .insert_resource(Chip8::new(&data, 600))
        .add_startup_system(pixel_buffer_setup(pixel_buffer_size))
        .add_system(ui::ui_system)
        .add_system(emulator::emulator_system)
        .run();

    Ok(())
}