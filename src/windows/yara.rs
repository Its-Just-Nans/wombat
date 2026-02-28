//! Yara

use std::fmt::Debug;

use bladvak::eframe::egui;
use bladvak::errors::ErrorManager;

/// type alias for rule
pub(crate) type RuleResult = (String, bool);

/// type alias for match
pub(crate) type Match = Vec<RuleResult>;

/// yara decoded result
#[derive(Debug)]
#[cfg_attr(target_arch = "wasm32", allow(unused))]
pub(crate) struct YaraResult {
    /// yara matching rules
    matching_rules: Match,
    /// yara non matching rules
    non_matching_rules: Match,
}

/// Yara data
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct YaraData {
    /// rule string
    rule: String,

    /// results - matching and non-matching rules
    #[serde(skip)]
    #[cfg_attr(target_arch = "wasm32", allow(unused))]
    results: Option<YaraResult>,

    /// Error
    #[serde(skip)]
    error: Option<String>,
}

impl Default for YaraData {
    fn default() -> Self {
        Self {
            rule: r#"rule lorem_ipsum {
    strings:
        $ = "Lorem ipsum"
    condition:
        all of them
}"#
            .to_string(),
            results: None,
            error: None,
        }
    }
}

/// Yara data
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct Yara {
    /// is open
    pub(crate) is_open: bool,

    /// optional data
    data: Option<YaraData>,
}

impl Yara {
    /// Create empty
    pub(crate) fn new() -> Self {
        Self {
            is_open: false,
            ..Self::default()
        }
    }

    /// Window title
    pub(crate) fn window_title() -> &'static str {
        "Yara-X"
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.data = None;
    }

    /// Extract a rule result
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn extract_result(r: &yara_x::Rule<'_, '_>) -> RuleResult {
        (r.identifier().to_string(), r.is_private())
    }

    /// Window UI
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn window_ui(_binary_data: &[u8], _data: &mut YaraData, ui: &mut egui::Ui) {
        ui.label("Yara is not enabled in wasm32");
    }

    /// Window UI
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn window_ui(binary_data: &[u8], data: &mut YaraData, ui: &mut egui::Ui) {
        use bladvak::egui_extras::{Column, TableBuilder};
        use yara_x::Compiler;

        if ui.button("Run rules").clicked() {
            let mut compiler = Compiler::new();
            match compiler.add_source(&*data.rule) {
                Ok(_) => {
                    let rules = compiler.build();
                    let mut scanner = yara_x::Scanner::new(&rules);
                    match scanner.scan(binary_data) {
                        Ok(results) => {
                            let matching_rules = results
                                .matching_rules()
                                .map(|r| Self::extract_result(&r))
                                .collect();
                            let non_matching_rules = results
                                .non_matching_rules()
                                .map(|r| Self::extract_result(&r))
                                .collect();
                            data.results = Some(YaraResult {
                                matching_rules,
                                non_matching_rules,
                            });
                        }
                        Err(e) => {
                            data.results = None;
                            data.error = Some(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    data.results = None;
                    data.error = Some(e.to_string());
                }
            }
        }
        if let Some(res) = &data.results {
            TableBuilder::new(ui)
                .striped(true)
                .column(Column::auto())
                .column(Column::auto())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.label("Matching Rules");
                    });
                    header.col(|ui| {
                        ui.label("Non matching rules");
                    });
                })
                .body(|mut body| {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            for one_res in &res.matching_rules {
                                ui.label(&one_res.0);
                            }
                        });
                        row.col(|ui| {
                            for one_res in &res.non_matching_rules {
                                ui.label(&one_res.0);
                            }
                        });
                    });
                });
        } else if data.error.is_none() {
            ui.label("No result - click to Run rules");
        }
    }

    /// Show the histogram ui
    pub(crate) fn ui(
        &mut self,
        binary_data: &[u8],
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) {
        if self.is_open {
            let mut is_open = self.is_open;
            egui::Window::new(Self::window_title())
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    if let Some(data) = &mut self.data {
                        let is_wasm = cfg!(target_arch = "wasm32");
                        ui.add_enabled_ui(!is_wasm, |ui| {
                            ui.text_edit_multiline(&mut data.rule);
                        });
                        Self::window_ui(binary_data, data, ui);
                        if let Some(e) = &data.error {
                            ui.colored_label(egui::Color32::RED, "Error: ");
                            ui.colored_label(egui::Color32::RED, e);
                        }
                    } else {
                        let yara_data = YaraData::default();
                        self.data = Some(yara_data);
                    }
                });
            self.is_open = is_open;
        }
    }
}
