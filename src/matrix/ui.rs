use egui::Ui;
use vec_drain_where::VecDrainWhereExt;
use crate::{l3x::{L3X, L3XCommand}, traveler::Traveler};

use super::{Matrix, MatrixMode};

impl Matrix {
    pub fn config_ui(&mut self, ui: &mut Ui) {
        ui.label("Simulation");
        ui.horizontal(|ui| {
            ui.scope(|ui| {
                ui.set_enabled(!self.simulating);
                if ui.button("Start").clicked() {
                    self.init_simulation()
                }
            });
            ui.scope(|ui| {
                ui.set_enabled(self.simulating);
                ui.button("▶").on_hover_text("play (step automatically)");
                ui.button("⏸").on_hover_text("pause (stop autostepping)");
                if ui.button("⏭").on_hover_text("step by one cycle").clicked() {
                    self.step()
                }
                if ui
                    .button("⏹")
                    .on_hover_text("exit the simulation")
                    .clicked()
                {
                    self.cleanup_simulation();
                }
            });
        });

        ui.scope(|ui| {
            ui.set_enabled(!self.simulating);
            ui.separator();
            ui.label("L3 Mode");
            ui.horizontal(|ui| {
                let l3_radio = ui.radio_value(&mut self.mode, MatrixMode::L3, "L3");
                let l3x_radio = ui.radio_value(&mut self.mode, MatrixMode::L3X, "L3X");
                if l3_radio.union(l3x_radio).changed() {
                    self.dims = self.dims.max(self.mode.minimum_size());
                    self.force_queue_l3x()
                }
            });

            ui.separator();
            ui.label("Single input (L3)");
            if let Some(ref value) = self.single_input {
                ui.label(format!("Current value: {}", value));
            }
            if ui
                .text_edit_singleline(&mut self.single_input_text)
                .lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
            {
                if let Ok(registers) = self.single_input_text.parse() {
                    self.single_input = Some(registers);
                }
            }

            if self.mode == MatrixMode::L3X {
                ui.separator();
                ui.label("Multi input (L3X)");

                self.stream_input
                    .e_drain_where(|registers| ui.button(registers.to_string()).clicked())
                    .for_each(drop);
                if let Some(ref mut text) = self.stream_input_text {
                    let textedit = ui.text_edit_singleline(text);
                    if self.single_input_next_frame_focus {
                        textedit.request_focus();
                        self.single_input_next_frame_focus = false;
                    }
                    if textedit.lost_focus() {
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if let Ok(registers) = text.parse() {
                                self.stream_input.push(registers);
                            }
                            text.clear();
                            self.single_input_next_frame_focus = true;
                        } else {
                            self.stream_input_text = None;
                        }
                    }
                } else if ui.button("Add to stream").clicked() {
                    self.stream_input_text = Some(String::new());
                    self.single_input_next_frame_focus = true;
                }
            }
        });

        if let Some(location) = self.selecting {
            ui.scope(|ui| {
                ui.set_enabled(!self.simulating);
                ui.separator();
                ui.label(format!("Cell value @ {location}"));
                ui.horizontal(|ui| {
                    let textedit = ui.text_edit_singleline(&mut self.selecting_text);
                    match self.focus_editing {
                        x if x > 1 => self.focus_editing -= 1,
                        1 => {
                            textedit.request_focus();
                            self.focus_editing -= 1;
                        }
                        _ => (),
                    }
                    if textedit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(serialize_success) = L3X::try_from(self.selecting_text.as_str()) {
                            if self.is_editing_input_stream()
                                && serialize_success.command != L3XCommand::Queue
                            {
                                log::warn!("In L3X mode, edited square *must* be a queue!")
                            } else {
                                self.storage.insert(location, serialize_success);
                            }
                        } else {
                            log::warn!("Serialization failure")
                        }
                    }
                    if ui.button("Clear").clicked() {
                        self.selecting_text.clear();
                        self.storage.remove(&location);
                        self.force_queue_l3x();
                    }
                });
            });

            if self.simulating {
                ui.separator();
                ui.label("Travelers on this cell");
                self.travelers
                    .iter()
                    .filter(|&&Traveler { location: loc, .. }| loc == location)
                    .for_each(|traveler| {
                        ui.label(traveler.to_string());
                    });

                if let Some(queue) = self.queues.get(&location) {
                    ui.separator();
                    ui.label("Queue on this cell");
                    for register in queue {
                        ui.label(register.to_string());
                    }
                }
            }

            ui.separator();
            ui.label("Output");
            if let Some(ref register) = self.output {
                ui.label(register.to_string());
            }

            ui.separator();
            ui.label("Output stream");
            for register in &self.output_stream {
                ui.label(register.to_string());
            }
        }
    }
}
