use std::cmp;

use bevy::prelude::{Res, ResMut, Time};
use bevy_egui::{egui, EguiContext};
use crate::{resources::chip8::*, config::RAM_SIZE};

pub fn ui_system(mut egui_ctx: ResMut<EguiContext>, mut chip8_res : ResMut<Chip8>, time: Res<Time>) {
    egui::Window::new("Simulation control").show(egui_ctx.ctx_mut(), 
        |ui| {
            ui.horizontal(|ui| {
                if !chip8_res.paused() && ui.button("Pause").clicked() {
                    chip8_res.pause();
                }
                if chip8_res.paused() && ui.button("Play").clicked() {
                    chip8_res.run();
                }
                if chip8_res.paused() && ui.button("Step").clicked() {
                    chip8_res.step(time.delta());
                }
            });

            ui.separator();

            let mut clock_hz = chip8_res.clock_hz;
            ui.add(egui::Slider::new(&mut clock_hz, 1..=2000).text("Cpu clock in Hz"));
            chip8_res.change_clock(clock_hz);
            
            ui.separator();

            ui.checkbox(&mut chip8_res.super_chip, "SuperChip/Chip-48 behaviour").on_hover_ui(|ui| {
                ui.label("Changes the behaviour of some instructions.");
            });
        }
    );

    egui::Window::new("Debug").show(egui_ctx.ctx_mut(), 
        |ui| {
            ui.checkbox(&mut chip8_res.debug, "Debug");
            if chip8_res.debug {
                let pc = chip8_res.pc() as usize;
                ui.label(format!("ROM size: {}", chip8_res.rom_sz()));
                ui.label(format!("PC: {}", pc));
                ui.label(format!("Stack: {:?}", chip8_res.stack()));
                
                let mut ram_cp : [u8; 25] = [0; 25];
                let rng = cmp::max(pc - 12, 0)..=cmp::min(pc + 12, RAM_SIZE-1);
                let slice_len = rng.end() - rng.start() + 1;
                ram_cp[0..slice_len].copy_from_slice(&chip8_res.ram()[rng]);
                ui.label(format!("RAM: ... {:?} ...", ram_cp));
            }
        }
    );
}