use bevy::prelude::{ScanCode, ResMut, Res, Input};
use crate::resources::chip8::*;
use crate::config::NUM_KEYS;

const KEY_MAP : [u32; NUM_KEYS]= [
    45, // 0 => X
    2,  // 1 => 1
    3,  // 2 => 2
    4,  // 3 => 3
    16, // 4 => Q
    17, // 5 => W
    18, // 6 => E
    30, // 7 => A
    31, // 8 => S
    32, // 9 => D
    44, // A => Z
    46, // B => C
    5,  // C => 4
    19, // D => R
    33, // E => F
    47, // F => V
];

pub fn keyboard_system(mut chip8_res: ResMut<Chip8>, keys: Res<Input<ScanCode>>) {
    for i in 0..NUM_KEYS {
        if keys.just_released(ScanCode(KEY_MAP[i])) {
            chip8_res.input[i] = KeyState::JustReleased;
        }
        if keys.pressed(ScanCode(KEY_MAP[i])) {
            chip8_res.input[i] = KeyState::Pressed;
        }
    }
}