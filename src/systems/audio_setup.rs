use bevy::prelude::*;
use crate::resources::beep::*;

/// Load CHIP-8's beep sound in bevy.
pub fn setup_audio_system(asset_server: Res<AssetServer>, mut beep_source : ResMut<BeepResource>) {
    beep_source.sound = Box::new(asset_server.load("sounds/c_major.wav"));
}