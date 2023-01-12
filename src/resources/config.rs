use bevy::prelude::Resource;

#[derive(Resource)]
pub struct ConfigResource {
    pub debug_ui: bool,
    pub show_grid: bool,
    pub trace: bool,
    pub circle_pixels: bool,
    pub reduce_flicker: bool,
}