use std::io::Read;

use bevy::{prelude::*, window::WindowResizeConstraints};
use bevy_pixels::{PixelsPlugin, PixelsStage};

mod config;
mod resources;
mod systems;

use crate::config::*;
use crate::resources::chip8::*;
use crate::systems::emulator::*;

fn main() -> std::io::Result<()> {
    let args : Vec<String> = std::env::args().collect();
    let filepath = &args[1];

    let f = std::fs::File::open(filepath)?;
    let len = f.metadata().unwrap().len();
    
    let mut data : Vec<u8> = Vec::new();
    data.resize(len as usize, 0);

    let mut file = std::io::BufReader::new(f);
    file.read(data.as_mut_slice()).expect("File couldn't be read successfully!");

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
        .add_plugin(PixelsPlugin {
            width: WIDTH,
            height: HEIGHT,
            ..default()
        })
        .add_system_to_stage(PixelsStage::Draw, emulator_system)
        .insert_resource(Chip8::new(&data, 600))
        .run();

    Ok(())
}