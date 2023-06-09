use crate::{
    l3x::{L3XCommand, L3X, MaybeL3X},
    registers::Registers,
    traveler::Traveler,
    wasync::AsyncContext,
};
use egui::{CollapsingHeader, CollapsingResponse, Ui, WidgetText};
use macroquad::prelude::*;
use vec_drain_where::VecDrainWhereExt;

use super::{Matrix, MatrixMode,MatrixAction};

trait EguiExt {
    fn collapsing_open<R>(
        &mut self,
        heading: impl Into<WidgetText>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R>;
}

impl EguiExt for Ui {
    fn collapsing_open<R>(
        &mut self,
        heading: impl Into<WidgetText>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R> {
        CollapsingHeader::new(heading)
            .default_open(true)
            .show(self, add_contents)
    }
}

#[derive(Default)]
pub struct UiSingleInput {
    text: String,
    error_text: Option<String>,
    value: Registers,
}

impl UiSingleInput {
    fn ui(&mut self, ui: &mut Ui, simulating: bool) {
        ui.set_enabled(!simulating);
        ui.label(format!("Current value: {}", self.value));
        let text_edit = ui.text_edit_singleline(&mut self.text);
        if let Some(ref err) = self.error_text {
            ui.label(WidgetText::from(err).color(egui::Color32::RED));
        }
        if text_edit.has_focus() && ui.input(|i| !i.keys_down.is_empty()) {
            self.error_text = None;
        }
        if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            match self.text.parse() {
                Ok(registers) => self.value = registers,
                Err(e) => self.error_text = Some(e.to_string()),
            }
        }
    }

    pub fn value(&self) -> &Registers {
        &self.value
    }
}

#[derive(Default)]
pub struct UiStreamInput {
    next_frame_focus: bool,
    input_text: Option<String>,
    value: Vec<Registers>,
}

impl UiStreamInput {
    fn ui(&mut self, ui: &mut Ui, simulating: bool) {
        ui.set_enabled(!simulating);
        self.value
            .e_drain_where(|registers| ui.button(registers.to_string()).clicked())
            .for_each(drop);
        if let Some(ref mut text) = self.input_text {
            let textedit = ui.text_edit_singleline(text);
            if self.next_frame_focus {
                textedit.request_focus();
                self.next_frame_focus = false;
            }
            if textedit.lost_focus() {
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(registers) = text.parse() {
                        self.value.push(registers);
                    }
                    text.clear();
                    self.next_frame_focus = true;
                } else {
                    self.input_text = None;
                }
            }
        } else if ui.button("Add to stream").clicked() {
            self.input_text = Some(String::new());
            self.next_frame_focus = true;
        }
    }

    pub fn value(&self) -> &Vec<Registers> {
        &self.value
    }
}

impl Matrix {
    fn ui_simulation_tools(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.scope(|ui| {
                ui.set_enabled(!self.simulating);
                if ui.button("Start").clicked() {
                    self.init_simulation()
                }
            });
            ui.scope(|ui| {
                ui.set_enabled(self.simulating);
                ui.scope(|ui| {
                    ui.set_enabled(!self.stepping);
                    if ui
                        .button("▶")
                        .on_hover_text("play (step automatically)")
                        .on_disabled_hover_text("play (step automatically")
                        .clicked()
                    {
                        self.stepping = true;
                    };
                });
                ui.scope(|ui| {
                    ui.set_enabled(self.stepping);
                    if ui
                        .button("⏸")
                        .on_hover_text("pause (stop autostepping)")
                        .on_disabled_hover_text("pause (stop autostepping")
                        .clicked()
                    {
                        self.stepping = false;
                    };
                });
                if ui
                    .button("⏭")
                    .on_hover_text("step by one cycle")
                    .on_disabled_hover_text("step by one cycle")
                    .clicked()
                {
                    self.step()
                }
                if ui
                    .button("⏹")
                    .on_hover_text("exit the simulation")
                    .on_disabled_hover_text("exit the simulation")
                    .clicked()
                {
                    self.cleanup_simulation();
                    self.stepping = false;
                }
            });
            ui.separator();
            ui.scope(|ui| {
                ui.set_enabled(!self.simulating);
                ui.horizontal(|ui| {
                    let l3_radio = ui.radio_value(&mut self.mode, MatrixMode::L3, "L3");
                    let l3x_radio = ui.radio_value(&mut self.mode, MatrixMode::L3X, "L3X");
                    if l3_radio.union(l3x_radio).changed() {
                        self.dims = self.dims.max(self.mode.minimum_size());
                        self.force_queue_l3x()
                    }
                });
            });
        });

        ui.horizontal(|ui| {
            ui.label("Simulation rate (in frame time)");
            ui.add(egui::widgets::Slider::new(&mut self.period, 1..=120))
        });
    }

    fn ui_cell_value_view(&mut self, ui: &mut Ui, location: IVec2) {
        ui.set_enabled(!self.simulating);
        ui.label(format!("at location {location}"));
        ui.horizontal(|ui| {
            let textedit = ui.text_edit_singleline(&mut self.selecting_text);
            if self.focus_editing {
                if textedit.has_focus() {
                    self.focus_editing = false;
                } else {
                    textedit.request_focus()
                }
            }
            if textedit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(serialize_success) = L3X::try_from(self.selecting_text.as_str()) {
                    if self.is_editing_input_stream()
                        && serialize_success.command != L3XCommand::Queue
                    {
                        log::warn!("In L3X mode, edited square *must* be a queue!")
                    } else {
                        //self.instructions.insert(location, serialize_success);
                        self.apply(MatrixAction::Paste(location,super::L3XData { data: vec![vec![crate::l3x::MaybeL3X::Some(serialize_success)]], dims:uvec2(1,1) }));
                    }
                } else {
                    log::warn!("Serialization failure")
                }
            }
            if ui.button("Clear").clicked() {
                self.selecting_text.clear();
                self.instructions.remove(&location);
                self.force_queue_l3x();
            }
        });
    }

    fn ui_cell_traveler_view(&mut self, ui: &mut Ui, location: IVec2) {
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

    fn ui_output_view(&mut self, ui: &mut Ui) {
        ui.label("Output");
        if let Some(ref register) = self.output {
            ui.label(register.to_string());
        }

        if self.mode == MatrixMode::L3X {
            ui.separator();
            ui.label("Output stream");
            for register in &self.output_stream {
                ui.label(register.to_string());
            }
        }
    }

    fn ui_edit_matrix(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.gridlines, "Gridlines");
        if ui.button("transpose").clicked() {
            //self.transpose();
            if let Some(selection)=self.selecting {
                self.apply(MatrixAction::Transpose(selection));
                self.selecting=Some(selection.transpose());
            }
        }
        if ui.button("reflect horizontally").clicked() {
            //self.transpose();
            if let Some(selection)=self.selecting {
                self.apply(MatrixAction::ReflectH(selection));
            }
        }
        if ui.button("reflect vertically").clicked() {
            //self.transpose();
            if let Some(selection)=self.selecting {
                self.apply(MatrixAction::ReflectV(selection));
            }
        }
        if ui.button("clear").clicked() {
            //self.transpose();
            if let Some(selection)=self.selecting {
                self.apply(MatrixAction::Paste(selection.starts,super::L3XData { data: vec![vec![MaybeL3X::None;selection.width() as usize];selection.height() as usize], dims: uvec2(selection.width() as u32,selection.height() as u32) }));
            }
        }
        if ui.button("copy").clicked() {
            if let Some(selection)=self.selecting {
                let data=self.snip(selection);
                self.copy_data=Some(data.clone());
                self.apply_raw(MatrixAction::Paste(selection.starts,data));
            }
        }
        if ui.button("paste").clicked() {
            if let Some(selection)=self.selecting {
                if let Some(data)=&self.copy_data {
                    self.selecting=Some(super::Selection { starts: selection.starts, ends: selection.starts+ivec2(data.dims.x as i32-1,data.dims.y as i32-1) });
                    self.apply(MatrixAction::Paste(selection.starts,data.clone()));
                    
                }
            }
        }
        if ui.button("undo").clicked() {
            self.undo();
        }
    }

    fn ui_import(&mut self, ui: &mut Ui, ctx: &mut AsyncContext) {
        ui.horizontal(|ui| {
            ui.scope(|ui| {
                ui.set_enabled(!self.simulating);
                if ui.button("Import").clicked() {
                    ctx.start_file_import();
                }
            });
            if ui.button("Export").clicked() {
                if let Ok(data) = self.export_data() {
                    ctx.start_file_export(data, self.mode)
                }
            };
        });
    }

    fn ui_sound(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Volume");
            ui.add(egui::widgets::Slider::new(&mut self.global_volume, 0..=100));
        });
        ui.checkbox(
            &mut self.sound_follows_cursor,
            "Play sound for traveler closest to cursor",
        );
    }

    pub fn config_ui(
        &mut self,
        ui: &mut Ui,
        // #[cfg(not(target_arch = "wasm32"))] executor: &mut async_executor::LocalExecutor,
        executor: &mut AsyncContext,
    ) {
        ui.heading("Simulation");
        self.ui_simulation_tools(ui);

        ui.separator();
        ui.collapsing_open("Import tools", |ui| {
            // #[cfg(not(target_arch = "wasm32"))]
            self.ui_import(ui, executor);
        });

        ui.separator();
        ui.collapsing_open("Single input", |ui| {
            self.single_input.ui(ui, self.simulating)
        });

        if self.mode == MatrixMode::L3X {
            ui.separator();
            ui.collapsing_open("Multi input (L3X)", |ui| {
                self.stream_input.ui(ui, self.simulating)
            });
        }

        ui.separator();
        ui.collapsing_open("Matrix editing", |ui| {
            self.ui_edit_matrix(ui);
            if let Some(location) = self.selecting {
                ui.scope(|ui| self.ui_cell_value_view(ui, location.starts));
            }
        });

        if let Some(location) = self.selecting {
            if self.simulating {
                ui.separator();
                ui.collapsing_open("Travelers", |ui| {
                    self.ui_cell_traveler_view(ui, location.starts)
                });
            }
        }

        ui.separator();
        ui.collapsing_open("Sound", |ui| self.ui_sound(ui));

        ui.separator();
        ui.collapsing_open("Output", |ui| self.ui_output_view(ui));
        ui.separator();
        ui.collapsing("About", |ui| {
            ui.label("Authors: Eric Yu (zbrachinara), Caleb Dastrup (CJD13)");
            ui.hyperlink_to("Github link", "https://github.com/zbrachinara/l3x-ide");
        });
    }
}
