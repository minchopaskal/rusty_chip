use bevy::{prelude::Resource, time::Timer};

/// A simple timer used for drawing at 60FPS.
#[derive(Resource)]
pub struct DrawTimer {
    pub timer: Timer,
}
