use bevy::prelude::*;

/// Resource storing the beeping sound of Chip8
#[derive(Component)]
pub struct Beep;

/// Load CHIP-8's beep sound in bevy.
pub fn setup_audio_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("sounds/c_major.wav"),
            ..default()
        },
        Beep,
    ));
}
