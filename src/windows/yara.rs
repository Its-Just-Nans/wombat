//! Yara

use std::fmt::Debug;

use bladvak::eframe::egui;
use bladvak::egui_extras::{Column, TableBuilder};
use bladvak::errors::ErrorManager;
use yara_x::{Compiler, Rule};

/// type alias for rule
pub(crate) type RuleResult = (String, bool);

/// type alias for match
pub(crate) type Match = Vec<RuleResult>;

/// yara decoded result
#[derive(Debug)]
pub(crate) struct YaraResult {
    /// yara matching rules
    matching_rules: Match,
    /// yara non matching rules
    non_matching_rules: Match,
}

/// Yara data
#[derive(Debug)]
pub(crate) struct YaraData {
    /// rule string
    rule: String,
    /// results - matching and non-matching rules
    results: Option<YaraResult>,
}

/// Yara data
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct Yara {
    /// is open
    pub(crate) is_open: bool,

    #[serde(skip)]
    /// optional data
    data: Option<YaraData>,

    /// Error
    #[serde(skip)]
    error: Option<String>,
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
        "Yara"
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.data = None;
        self.error = None;
    }

    /// Extract a rule result
    pub(crate) fn extract_result(r: &Rule<'_, '_>) -> RuleResult {
        (r.identifier().to_string(), r.is_private())
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
                        ui.text_edit_multiline(&mut data.rule);
                        if ui.button("Run rules").clicked() {
                            self.error = None;
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
                                            self.error = Some(e.to_string());
                                        }
                                    }
                                }
                                Err(e) => {
                                    data.results = None;
                                    self.error = Some(e.to_string());
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
                        } else if self.error.is_none() {
                            ui.label("No result - click to Run rules");
                        }

                        if let Some(e) = &self.error {
                            ui.colored_label(egui::Color32::RED, "Error: ");
                            ui.colored_label(egui::Color32::RED, e);
                        }
                    } else {
                        let yara_data = YaraData {
                            rule: r#"rule lorem_ipsum {
    strings:
        $ = "Lorem ipsum"
    condition:
        all of them
}"#
                            .to_string(),
                            results: None,
                        };
                        self.data = Some(yara_data);
                    }
                });
            self.is_open = is_open;
        }
    }
}
