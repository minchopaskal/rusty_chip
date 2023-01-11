use std::cmp;
use std::io::Read;

use bevy::{prelude::{Res, ResMut, Time}};
use bevy_egui::{egui, EguiContext};
use bevy_pixel_buffer::query::QueryPixelBuffer;
use rfd::FileDialog;

use crate::{resources::chip8::*, config::{RAM_SIZE, REGISTER_COUNT}};

pub fn ui_system(mut egui_ctx: ResMut<EguiContext>, mut chip8_res : ResMut<Chip8>, pb: QueryPixelBuffer, time: Res<Time>) {
    let ctx = egui_ctx.ctx_mut();

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, "File", |ui| {
                if ui.button("Open").clicked() {
                    let file = FileDialog::new()
                        .add_filter("", &["ch8"])
                        .set_directory("test/")
                        .pick_file();

                    if file == None {
                        ui.close_menu();
                        return;
                    }

                    let f = std::fs::File::open(file.unwrap().as_path()).expect("Invalid filename!");
                    let len = f.metadata().unwrap().len();
                    
                    let mut data : Vec<u8> = Vec::new();
                    data.resize(len as usize, 0);

                    let mut file = std::io::BufReader::new(f);
                    file.read(data.as_mut_slice()).expect(&format!("Couldn't read file!"));

                    chip8_res.insert_cartridge(&data);

                    ui.close_menu();
                }
            });
        });
    });

    egui::SidePanel::left("left_panel").show(ctx, |ui| {
        ui.heading("Simulation Control");
        ui.separator();

        ui.horizontal(|ui| {
            if chip8_res.rom_sz() <= 0 {
                ui.label("Please open a ROM (.ch8) file!");
                return;
            }

            if !chip8_res.paused() && ui.button("Pause").clicked() {
                chip8_res.pause();
            }
            if chip8_res.paused() && ui.button("Play").clicked() {
                chip8_res.run();
            }
            if chip8_res.paused() && ui.button("Step").clicked() {
                chip8_res.step(time.delta());
            }
            if ui.button("Reset").clicked() {
                chip8_res.reset();
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

        ui.heading("CHIP8 Inspector");
        ui.separator();

        ui.checkbox(&mut chip8_res.debug, "Debug");
        if chip8_res.debug {
            let pc = chip8_res.pc() as usize;
            ui.label(format!("ROM size: {}", chip8_res.rom_sz()));
            ui.label(format!("PC: {}", pc));
            ui.label(format!("SP: {}", chip8_res.sp()));
        }
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        // get the egui texture
        let texture = pb.egui_texture();

        // show the texture as an image
        ui.image(texture.id, texture.size);

        // TODO: Panel should be only as big as the framebuffer
    });

    egui::SidePanel::right("right_panel").show(ctx, |ui| {
        ui.heading("Register Inspector");

        for i in 0..REGISTER_COUNT {

        }

        ui.separator();
        
        ui.heading("Stack Inspector");
        
        // TODO
    });

    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.heading("RAM Inspector");
        // TODO
    });
}