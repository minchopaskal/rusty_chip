use std::time::Duration;

use crate::{
    config::{DELTA_S, NUM_KEYS},
    resources::chip8::{Chip8, KeyState},
};
use bevy::{
    input::keyboard::KeyboardInput,
    prelude::{EventReader, Input, KeyCode, Res, ResMut},
};

use scancode::Scancode;

/// Key mapping from real keyboard to CHIP-8s input.
///
/// Mapping uses scancodes in order to support different
/// keyboard layouts.
///
/// For the QWERTY layout the mapping looks like this:
///
///      (real)               (chip-8)
/// -----------------    -----------------
/// | 1 | 2 | 3 | 4 |    | 1 | 2 | 3 | C |
/// -----------------    -----------------
/// | Q | W | R | T |    | 4 | 5 | 6 | D |
/// ----------------- -> -----------------
/// | A | S | D | F |    | 7 | 8 | 9 | E |
/// -----------------    -----------------
/// | Z | X | C | V |    | A | 0 | B | F |
/// -----------------    -----------------
// const KEY_MAP : [u32; NUM_KEYS]= [
//     45, // 0 => X
//     2,  // 1 => 1
//     3,  // 2 => 2
//     4,  // 3 => 3
//     16, // 4 => Q
//     17, // 5 => W
//     18, // 6 => E
//     30, // 7 => A
//     31, // 8 => S
//     32, // 9 => D
//     44, // A => Z
//     46, // B => C
//     5,  // C => 4
//     19, // D => R
//     33, // E => F
//     47, // F => V
// ];

const KEY_MAP: [Scancode; NUM_KEYS] = [
    Scancode::X,    // 0 => X
    Scancode::Num1, // 1 => 1
    Scancode::Num2, // 2 => 2
    Scancode::Num3, // 3 => 3
    Scancode::Q,    // 4 => Q
    Scancode::W,    // 5 => W
    Scancode::E,    // 6 => E
    Scancode::A,    // 7 => A
    Scancode::S,    // 8 => S
    Scancode::D,    // 9 => D
    Scancode::Z,    // A => Z
    Scancode::C,    // B => C
    Scancode::Num4, // C => 4
    Scancode::R,    // D => R
    Scancode::F,    // E => F
    Scancode::V,    // F => V
];

/// Simple input handling system
pub fn keyboard_system(
    mut chip8_res: ResMut<Chip8>,
    keycodes: Res<Input<KeyCode>>,
    mut key_evr: EventReader<KeyboardInput>,
) {
    use bevy::input::ButtonState;

    for ev in key_evr.read() {
        match ev.state {
            ButtonState::Released => {
                if let Some(sc) = Scancode::new(ev.scan_code as u8) {
                    for (i, key) in KEY_MAP.iter().enumerate() {
                        if sc == *key {
                            chip8_res.input[i] = KeyState::JustReleased;
                        }
                    }
                }
            }
            ButtonState::Pressed => {
                if let Some(sc) = Scancode::new(ev.scan_code as u8) {
                    for (i, key) in KEY_MAP.iter().enumerate() {
                        if sc == *key {
                            chip8_res.input[i] = KeyState::Pressed;
                        }
                    }
                }
            }
        }
    }

    if chip8_res.paused() && keycodes.pressed(KeyCode::Space) {
        chip8_res.step(Duration::from_secs_f64(DELTA_S));
    }
}

