use bevy::prelude::{Res, ResMut, Time};
use bevy_egui::{egui, EguiContext};
use crate::resources::chip8::*;

pub fn ui_system(mut egui_ctx: ResMut<EguiContext>, mut chip8_res : ResMut<Chip8>, time: Res<Time>) {
    egui::Window::new("Simulation control").show(egui_ctx.ctx_mut(), 
        |ui| {
            if ui.button("Pause").clicked() {
                chip8_res.pause();
            }
            if ui.button("Play").clicked() {
                chip8_res.run();
            }
            if chip8_res.paused() && ui.button("Step").clicked() {
                chip8_res.step(time.delta());
            }
        }
    );
}