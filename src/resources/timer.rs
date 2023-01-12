use bevy::{prelude::Resource, time::Timer};

#[derive(Resource)]
pub struct DrawTimer {
    pub timer: Timer,
}