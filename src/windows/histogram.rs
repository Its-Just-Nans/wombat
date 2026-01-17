//! Histogram

use bladvak::eframe::egui;
use bladvak::eframe::egui::ahash::{HashMap, HashMapExt};
use bladvak::errors::ErrorManager;
use egui_plot::{Bar, BarChart, Legend, Plot};

/// Histogram data
#[derive(Debug)]
pub(crate) struct Histogram {
    /// is open
    pub(crate) is_open: bool,
    /// histogram values
    data: Option<HashMap<u8, usize>>,
    /// vertical or horizontal
    vertical: bool,
    /// bar width
    bar_width: f64,
}

impl Histogram {
    /// Create empty histogram
    pub(crate) fn new() -> Self {
        Self {
            is_open: false,
            data: None,
            vertical: false,
            bar_width: 1.0,
        }
    }

    /// Calculate the histogram
    fn calculate_histogram(binary_data: &[u8]) -> HashMap<u8, usize> {
        let mut map = binary_data.iter().fold(HashMap::new(), |mut map, &byte| {
            *map.entry(byte).or_insert(0) += 1;
            map
        });
        for idx in 0u8..=255 {
            map.entry(idx).or_insert(0);
        }
        map
    }

    /// Show the histogram ui
    pub(crate)  fn ui(&mut self, binary_data: &[u8], ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        if self.is_open {
            if self.data.is_some() {
                let mut is_open = self.is_open;
                egui::Window::new("Histogram")
                    .open(&mut is_open)
                    .vscroll(true)
                    .show(ui.ctx(), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Orientation:");
                            ui.selectable_value(&mut self.vertical, true, "Vertical");
                            ui.selectable_value(&mut self.vertical, false, "Horizontal");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Bar width");
                            ui.add(egui::Slider::new(&mut self.bar_width, 0.001..=2.0));
                        });
                        self.show_plot(ui);
                    });
                self.is_open = is_open;
            } else {
                self.data = Some(Self::calculate_histogram(binary_data));
            }
        }
    }

    /// Show the plot
    pub fn show_plot(&self, ui: &mut egui::Ui) {
        let Some(data) = &self.data else {
            return;
        };
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut chart = BarChart::new(
            "Histogram",
            data.iter()
                .map(|(byte, value)| {
                    #[allow(clippy::cast_precision_loss)]
                    let value = *value as f64;
                    min = value.min(min);
                    max = value.max(max);
                    Bar::new(f64::from(*byte), value)
                })
                .collect(),
        )
        .width(self.bar_width)
        .color(egui::Color32::LIGHT_BLUE);

        if !self.vertical {
            chart = chart.horizontal();
        }

        let x_label = "bytes (0 to 255)";
        let y_label = format!("count (min:{min} max:{max})");
        Plot::new("Distribution")
            .legend(Legend::default())
            .x_axis_label(if self.vertical { x_label } else { &y_label })
            .y_axis_label(if self.vertical { &y_label } else { x_label })
            .clamp_grid(true)
            .allow_zoom(egui::Vec2b::new(true, true))
            // .allow_drag(egui::Vec2b::new(true, true))
            // .allow_scroll(egui::Vec2b::new(true, true))
            .show(ui, |plot_ui| plot_ui.bar_chart(chart));
    }
}