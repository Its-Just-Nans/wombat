//! Central panel

use bladvak::eframe::egui::{
    self, Color32, FontFamily, FontId, Painter, ScrollArea, Stroke, TextStyle, Theme, Vec2,
};
use bladvak::errors::ErrorManager;

use crate::WombatApp;
use crate::app::Accent;

impl WombatApp {
    /// Show the central panel
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn app_central_panel(
        &mut self,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) {
        let text_style = TextStyle::Monospace;
        let row_height = ui.text_style_height(&text_style).max(14.0) + 1.0; // fallback
        ScrollArea::vertical().show_viewport(ui, |ui: &mut egui::Ui, viewport: egui::Rect| {
            if self.offset.need_change {
                // convert line to px
                let offset_needed = (self.offset.line_to_go as f32) * row_height;
                let offset_needed = if self.offset.current > offset_needed {
                    (self.offset.current - offset_needed) * Vec2::DOWN
                } else {
                    (offset_needed - self.offset.current) * Vec2::UP
                };
                ui.scroll_with_delta(offset_needed);
                self.offset.need_change = false;
            }
            let margin = ui.visuals().clip_rect_margin;

            self.offset.current = ui.clip_rect().top() - ui.min_rect().top() + margin;
            // 1) compute text metrics: row height using monospace TextStyle if available
            // Choose a monospace font id. Use the style's size for monospace if available:
            let font_size = ui
                .style()
                .text_styles
                .get(&text_style)
                .map_or(14.0, |s| s.size);
            // total lines we'll render
            let lines_total = self
                .binary_file
                .len()
                .div_ceil(self.display_settings.bytes_per_line);

            // total content height in points
            let total_height = (lines_total as f32) * row_height;

            // Reserve the space for the whole content (so scrollbar knows the full size)
            // We don't actually draw all rows, only the visible ones.
            let _rect = ui.allocate_space(egui::vec2(viewport.width(), total_height));
            // 2) find visible line range from viewport
            // viewport.rect.top() is the y of the top of the visible area in "world coordinates".
            // Convert to a line index
            let top_y = viewport.top(); // visible area's top in world coords
            let bottom_y = viewport.bottom(); // visible area's bottom

            // Ensure we clamp negatives
            let first_line = (top_y / row_height).floor().max(0.0) as usize;
            let last_line = (bottom_y / row_height).ceil().max(0.0) as usize;

            // clamp to valid range
            let first_line = first_line.min(lines_total);
            let last_line = last_line.min(lines_total);
            // padding from left inside the viewport
            let left = viewport.left() + 4.0;
            self.show_lines(ui, left, font_size, row_height, (first_line, last_line));
        });
    }

    /// Show file lines
    /// # Errors
    /// Fails if a something happens during render
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::too_many_lines)]
    fn show_lines(
        &mut self,
        ui: &mut egui::Ui,
        left: f32,
        font_size: f32,
        row_height: f32,
        (first_line, last_line): (usize, usize),
    ) {
        let mut mark_selection_stale = false;
        let bytes_per_line = self.display_settings.bytes_per_line;
        let font_id = FontId::new(font_size, FontFamily::Monospace);

        // 3) painter + font
        let painter = ui.painter();

        let mut y = first_line as f32 * row_height;

        // we'll draw 3 columns: offset, hex bytes, ascii
        // Choose x positions relative to `left`
        let offset_col_nb: usize = 8;
        let offset_col_width = 80.0; // "00000000:" width
        let hex_col_x = left + offset_col_width;
        // For hex column width estimate: bytes_per_line * 3 chars ("xx ") maybe plus small gap
        let hex_col_width = (bytes_per_line as f32) * 3.0 * (font_size * 0.6); // rough estimate
        let ascii_col_x = hex_col_x + hex_col_width + 8.0;
        for line in first_line..last_line {
            let offset = line * bytes_per_line;
            let slice_end = (offset + bytes_per_line).min(self.binary_file.len());
            let slice = &self.binary_file[offset..slice_end];

            // formatted offset
            let offset_text = format!("{offset:0offset_col_nb$X}:");

            // hex text: group each byte as two hex digits separated by a space
            let mut hex_buf = Vec::with_capacity(bytes_per_line);
            for b in slice {
                if self.display_settings.display_lsb {
                    hex_buf.push(format!("{:02X} ", b.reverse_bits()));
                } else {
                    hex_buf.push(format!("{b:02X} "));
                }
            }

            // ascii text: printable ascii or '.'
            let mut ascii_buf = Vec::with_capacity(bytes_per_line);
            for b in slice {
                let b = if self.display_settings.display_lsb {
                    b.reverse_bits()
                } else {
                    *b
                };
                let c = match b {
                    x if Self::RANGE_ASCII_PRINTABLE.contains(&x) => x as char,
                    c => {
                        if self.display_settings.limit_to_base_ascii {
                            '.'
                        } else {
                            c as char
                        }
                    }
                };
                ascii_buf.push(c);
            }

            // draw using painter at explicit positions so alignment stays correct
            let origin = ui.min_rect().min;
            painter.text(
                origin + Vec2::new(left, y),
                egui::Align2::LEFT_TOP,
                offset_text,
                font_id.clone(),
                ui.visuals().text_color(),
            );
            for (idx, (hex, ascii)) in std::iter::zip(&hex_buf, &ascii_buf).enumerate() {
                let x_pos = (idx as f32) * 3.0 * (font_size * 0.6);
                let color = if self
                    .selection
                    .range
                    .is_some_and(|s| (s.0..=s.1).contains(&(offset + idx)))
                {
                    if ui.ctx().theme() == Theme::Light {
                        self.selection.color.0
                    } else {
                        self.selection.color.1
                    }
                } else {
                    ui.visuals().text_color()
                };
                painter.text(
                    origin + Vec2::new(hex_col_x + x_pos, y),
                    egui::Align2::LEFT_TOP,
                    hex,
                    font_id.clone(),
                    color,
                );
                let hex_pos = (idx as f32) * (font_size * 0.6);
                painter.text(
                    origin + Vec2::new(ascii_col_x + hex_pos, y),
                    egui::Align2::LEFT_TOP,
                    ascii,
                    font_id.clone(),
                    color,
                );
            }

            let char_width = ui.fonts_mut(|f| f.glyph_width(&font_id, '0'));

            let offset_clicked = Self::interact_offset(
                ui,
                if self.visual_debug {
                    Some(painter)
                } else {
                    None
                },
                origin,
                bytes_per_line,
                char_width,
                offset_col_nb,
                row_height,
                line,
                y,
            );
            if offset_clicked {
                let is_shift = ui.ctx().input(|i| i.modifiers.shift);
                self.selection.range = self.handle_offset_click(offset, is_shift);
                mark_selection_stale = true;
            }
            let hex_group_width = char_width * 2.0; // "FF " is 3 chars

            for (idx, b) in slice.iter().enumerate() {
                let bx = hex_col_x + (idx as f32) * (hex_group_width + char_width);

                let byte_rect = egui::Rect::from_min_size(
                    origin + Vec2::new(bx, y),
                    egui::vec2(hex_group_width, row_height),
                );

                let resp = ui.interact(
                    byte_rect,
                    ui.id().with(("hex", line, idx)),
                    egui::Sense::click(),
                );
                if self.visual_debug {
                    painter.rect(
                        byte_rect,
                        1.0,
                        Color32::TRANSPARENT,
                        Stroke::new(0.5, Color32::BLACK),
                        egui::StrokeKind::Middle,
                    );
                }

                let is_clicked = resp.clicked();
                if resp.hovered() {
                    resp.on_hover_ui(|ui| {
                        let position = line * bytes_per_line + idx;
                        ui.label(format!("Bytes at index 0x{position:X} ({position})"));
                        self.ui_table_u8(ui, *b, &Accent::Hex);
                    });
                }
                if is_clicked {
                    let is_shift = ui.ctx().input(|i| i.modifiers.shift);
                    self.selection.range = self.handle_selection_click(offset, idx, is_shift);
                    mark_selection_stale = true;
                }

                // ASCII hover and click
                let bx = ascii_col_x + (idx as f32) * (font_size * 0.6);
                let width = 1.0 * char_width;

                let byte_rect = egui::Rect::from_min_size(
                    origin + Vec2::new(bx, y),
                    egui::vec2(width, row_height),
                );

                let resp = ui.interact(
                    byte_rect,
                    ui.id().with(("ascii", line, idx)),
                    egui::Sense::click(),
                );
                if self.visual_debug {
                    painter.rect(
                        byte_rect,
                        1.0,
                        Color32::TRANSPARENT,
                        Stroke::new(1.0, Color32::BLACK),
                        egui::StrokeKind::Outside,
                    );
                }

                let is_clicked = resp.clicked();
                if resp.hovered() {
                    resp.on_hover_ui(|ui| {
                        let position = line * bytes_per_line + idx;
                        ui.label(format!("Bytes at index 0x{position:X} ({position})"));
                        self.ui_table_u8(ui, *b, &Accent::Ascii);
                    });
                }
                if is_clicked {
                    let is_shift = ui.ctx().input(|i| i.modifiers.shift);
                    self.selection.range = self.handle_selection_click(offset, idx, is_shift);
                    mark_selection_stale = true;
                }
            }
            y += row_height;
        }
        if mark_selection_stale {
            self.stale_selection();
        }
    }

    /// Handle selection click
    fn handle_selection_click(
        &self,
        offset: usize,
        idx: usize,
        is_shift: bool,
    ) -> Option<(usize, usize)> {
        let current_idx = offset + idx;
        if let Some((select1, select2)) = self.selection.range {
            if is_shift {
                if select1 == current_idx {
                    return Some((current_idx, current_idx));
                } else if current_idx < select1 {
                    return Some((current_idx, select2));
                } else if select1 > current_idx {
                    return Some((current_idx, select1));
                } else if current_idx > select2 || (select1 < current_idx && current_idx < select2)
                {
                    return Some((select1, current_idx));
                }
            } else if select1 == current_idx {
                // unselect
                return None;
            } else {
                // no alt - set a single selection
                return Some((current_idx, current_idx));
            }
        }
        // no previous selection - create new selection
        Some((current_idx, current_idx))
    }

    /// Handle offset click
    fn handle_offset_click(&self, offset: usize, is_shift: bool) -> Option<(usize, usize)> {
        let end_idx = offset + self.display_settings.bytes_per_line - 1;
        let end_idx = if self.binary_file.len() > end_idx {
            end_idx
        } else {
            offset + (self.binary_file.len() - offset - 1)
        };
        if let Some((select1, select2)) = self.selection.range {
            if select1 == offset && select2 == end_idx {
                return None;
            }
            if is_shift {
                if offset > select1 {
                    // offset is after
                    return Some((select1, end_idx));
                }
                return Some((offset, select2));
            }
        }
        Some((offset, end_idx))
    }

    /// Interact the offset
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn interact_offset(
        ui: &egui::Ui,
        painter: Option<&Painter>,
        origin: egui::Pos2,
        bytes_per_line: usize,
        char_width: f32,
        offset_col_nb: usize,
        row_height: f32,
        line: usize,
        y: f32,
    ) -> bool {
        #[allow(clippy::cast_precision_loss)]
        let byte_rect = egui::Rect::from_min_size(
            origin + Vec2::new(0.0, y),
            egui::vec2(char_width * offset_col_nb as f32 + char_width, row_height),
        );

        let resp = ui.interact(
            byte_rect,
            ui.id().with(("offset_hex", line, 0)),
            egui::Sense::click(),
        );
        if let Some(painter) = painter {
            painter.rect(
                byte_rect,
                1.0,
                Color32::TRANSPARENT,
                Stroke::new(0.5, Color32::RED),
                egui::StrokeKind::Middle,
            );
        }
        let is_clicked = resp.clicked();
        if resp.hovered() {
            resp.on_hover_ui(|ui| {
                let p_start = line * bytes_per_line;
                let p_end = line * bytes_per_line + bytes_per_line - 1;
                let label = format!(
                    "Position from 0x{p_start:X} ({p_start}) to 0x{p_end:X} ({p_end})\nLine: {}",
                    y / row_height
                );
                ui.label(label);
            });
        }
        is_clicked
    }
}
