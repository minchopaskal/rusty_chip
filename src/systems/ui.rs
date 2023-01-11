use std::io::Read;

use bevy::{prelude::{Res, ResMut, Time}};
use bevy_egui::{egui::{self, RichText, TextStyle, Color32}, EguiContext};
use bevy_pixel_buffer::query::QueryPixelBuffer;
use rfd::FileDialog;

use crate::{resources::{chip8::*, config::*}, config::*};

fn show_central_panel(egui_ctx : &egui::Context, pb: QueryPixelBuffer,) {
    egui::CentralPanel::default().show(egui_ctx, |ui| {
        ui.centered_and_justified(|ui| {
            // get the egui texture
            let texture = pb.egui_texture();

            // show the texture as an image
            ui.image(texture.id, texture.size);
        });
    });
}

pub fn ui_system(mut egui_ctx: ResMut<EguiContext>, mut chip8_res : ResMut<Chip8>, pb: QueryPixelBuffer, time: Res<Time>, cfg: Res<ConfigResource>) {
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
            if cfg.debug_ui && chip8_res.paused() && ui.button("Step").clicked() {
                chip8_res.step(time.delta());
            }
            if ui.button("Reset").clicked() {
                chip8_res.reset();
            }
        });

        ui.separator();

        ui.checkbox(&mut chip8_res.super_chip, "SuperChip/Chip-48 behaviour").on_hover_ui(|ui| {
            ui.label("Changes the behaviour of some instructions.");
        });

        let mut clock_hz = chip8_res.clock_hz;
        ui.add(egui::Slider::new(&mut clock_hz, 1..=2000).text("Cpu clock in Hz"));
        chip8_res.change_clock(clock_hz);

        if !cfg.debug_ui {
            return;
        }

        ui.separator();

        ui.heading("CHIP8 Inspector");
        ui.separator();
        
        let pc = chip8_res.pc() as usize;
        ui.label(RichText::new(format!("ROM size: {}", chip8_res.rom_sz())).text_style(TextStyle::Monospace));
        ui.label(RichText::new(format!("PC: {} (0x{:03x})", pc, pc)).text_style(TextStyle::Monospace));
        ui.label(RichText::new(format!("SP: {}", chip8_res.sp())).text_style(TextStyle::Monospace));

        let instr_bytes : &[u8] = &chip8_res.ram()[pc..=pc+1];
        let instr : u16 = ((instr_bytes[0] as u16) << 8) | instr_bytes[1] as u16;
        ui.label(RichText::new(format!("Next instruction: 0x{:04x}", instr)).text_style(TextStyle::Monospace));
    });

    if !cfg.debug_ui {
        show_central_panel(ctx, pb);
        return;
    }

    egui::SidePanel::right("right_panel").show(ctx, |ui| {
        ui.heading("Register Inspector");

        ui.separator();
        
        for i in 0..REGISTER_COUNT {
            ui.label(RichText::new(format!("V{:01X}: 0x{:02x}", i, chip8_res.registers()[i])).text_style(TextStyle::Monospace));
        }

        ui.separator();
        
        ui.heading("Stack Inspector");
        
        ui.separator();
        
        ui.label(RichText::new(format!("SP: {}", chip8_res.sp())).text_style(TextStyle::Monospace));

        for i in 0..REGISTER_COUNT {
            ui.label(RichText::new(format!("Stack[{:02X}]: 0x{:02x}", i, chip8_res.stack()[i])).text_style(TextStyle::Monospace));
        }
    });

    egui::TopBottomPanel::bottom("bottom_panel")
    .resizable(true)
    .show(ctx, |ui| {
        ui.heading("RAM Inspector");
        
        ui.separator();

        let scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false, true]);

        scroll_area.show(ui, |ui| {
            const ROW_LEN : usize = 16;
            ui.horizontal(|ui| {
                ui.label(RichText::new("        ").text_style(TextStyle::Monospace));
                for i in 0..ROW_LEN {
                    ui.label(RichText::new(format!("{:02x}{}", i, if i == ROW_LEN - 1 { "" } else { " " }))
                        .text_style(TextStyle::Monospace));
                }
            });

            let ram = &chip8_res.ram();
            let mut i = 0;
            while i < ram.len() {
                ui.horizontal(|ui| {
                    const FONT_ADDR : usize = FONT_RANGE.start;
                    let (color, tail_label) = match i {
                        START_PC => { (Color32::LIGHT_RED, " (ROM)") },
                        FONT_ADDR => { (Color32::LIGHT_BLUE, " (FONT)") },
                        _ => { (Color32::WHITE, "") }
                    };
                    ui.label(RichText::new(format!("0x{:05x} ", i))
                        .text_style(TextStyle::Monospace)
                        .color(color)
                    );
                    
                    for j in 0..ROW_LEN {
                        ui.label(RichText
                            ::new(format!("{:02x}{}", ram[i + j], if j == ROW_LEN - 1 { tail_label } else { " " }))
                            .text_style(TextStyle::Monospace));
                    }
                });
                i += 16;            
            }
        });
    });

    show_central_panel(ctx, pb);
}