use bevy::prelude::*;

/// Resource storing the beeping sound of Chip8
#[derive(Resource, Default)]
pub struct BeepResource {
    pub sound : Box<Handle<AudioSource>>,
}
