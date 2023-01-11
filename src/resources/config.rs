use bevy::prelude::Resource;

#[derive(Resource)]
pub struct ConfigResource {
    pub debug_ui: bool,
}