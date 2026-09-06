//! Font preview

use std::collections::BTreeSet;

use bladvak::{
    eframe::egui::{self, Slider},
    egui_extras::{Column, TableBuilder},
};

/// Font preview data
#[derive(Debug, PartialEq)]
pub(crate) struct FontPreview {
    /// font preview data
    pub(crate) font_id: egui::FontId,
    /// font glyphs
    pub(crate) available_glyphs: BTreeSet<char>,
    /// text
    pub(crate) text: String,
}

impl FontPreview {
    /// new
    pub(crate) fn new(family: egui::FontFamily) -> Self {
        Self {
            font_id: egui::FontId { size: 24.0, family },
            available_glyphs: BTreeSet::new(),
            text: "The quick brown fox jumps over the lazy dog".to_string(),
        }
    }
    /// Show Ui
    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) {
        if self.available_glyphs.is_empty() {
            self.available_glyphs = available_characters(ui, &self.font_id.family);
        }
        ui.horizontal(|ui| {
            ui.label("Font size: ");
            ui.add(Slider::new(&mut self.font_id.size, 4.0..=40.0).max_decimals(1));
        });
        ui.collapsing("Test", |ui| {
            ui.text_edit_multiline(&mut self.text);
            ui.label(egui::RichText::new(self.text.as_str()).font(self.font_id.clone()));
        });
        ui.collapsing("Font book", |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::splat(2.0);

                    for chr in &self.available_glyphs {
                        let button = egui::Button::new(
                            egui::RichText::new(chr.to_string()).font(self.font_id.clone()),
                        )
                        .frame(false);

                        let tooltip_ui = |ui: &mut egui::Ui| {
                            let font_id = self.font_id.clone();

                            char_info_ui(ui, *chr, font_id);
                        };

                        if ui.add(button).on_hover_ui(tooltip_ui).clicked() {
                            ui.copy_text(chr.to_string());
                        }
                    }
                });
            });
        });
        ui.collapsing("Table font book", |ui| {
            let col_width = (ui.available_width() / 16.0).min(self.font_id.size * 1.5);
            let max_char = self
                .available_glyphs
                .iter()
                .next_back()
                .copied()
                .unwrap_or('\0') as usize;
            let max_char = max_char / 15 + 1;
            let table = TableBuilder::new(ui).columns(Column::exact(col_width), 16);
            table.body(|body| {
                body.rows(self.font_id.size * 1.5, max_char, |mut row| {
                    let row_index = row.index() * 15;
                    for idx in row_index..=(row_index + 15) {
                        let to_show = if let Ok(u) = u32::try_from(idx)
                            && let Some(c) = char::from_u32(u)
                            && self.available_glyphs.contains(&c)
                        {
                            Some(c)
                        } else {
                            None
                        };
                        row.col(|ui| {
                            if let Some(chr) = to_show {
                                let rect = ui.available_rect_before_wrap();
                                ui.painter().rect_filled(
                                    rect,
                                    0.0,
                                    egui::Color32::from_rgb(220, 235, 255),
                                );
                                let tooltip_ui = |ui: &mut egui::Ui| {
                                    let font_id = self.font_id.clone();

                                    char_info_ui(ui, chr, font_id);
                                };
                                let text =
                                    egui::RichText::new(chr.to_string()).font(self.font_id.clone());
                                if ui
                                    .put(rect, egui::Label::new(text).halign(egui::Align::Center))
                                    .on_hover_ui(tooltip_ui)
                                    .clicked()
                                {
                                    ui.copy_text(chr.to_string());
                                }
                            }
                        });
                    }
                });
            });
        });
    }
}

/// Available characters
pub(crate) fn available_characters(ui: &egui::Ui, family: &egui::FontFamily) -> BTreeSet<char> {
    ui.fonts_mut(|f| f.fonts.font(family).characters().keys().copied().collect())
}

/// Show info on the char
fn char_info_ui(ui: &mut egui::Ui, chr: char, font_id: egui::FontId) {
    let resp = ui.label(egui::RichText::new(chr.to_string()).font(font_id));

    egui::Grid::new("char_info")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Index");
            ui.label(format!("{}", chr as u32));
            ui.end_row();

            ui.label("Code");
            ui.label(format!("U+{:04X}", chr as u32));
            ui.end_row();

            ui.label("Hex");
            ui.label(format!("{:X}", chr as u32));
            ui.end_row();

            ui.label("Width");
            ui.label(format!("{:.1} pts", resp.rect.width()));
            ui.end_row();
        });
}
