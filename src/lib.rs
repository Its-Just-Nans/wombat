//! Wombat is a hex viewer made with [bladvak](https://github.com/Its-Just-Nans/bladvak) (egui)
//!
//! ```sh
//! cargo install wombat --locked
//!
//! wombat path/to/file.bin
//! ```

#![warn(clippy::all, rust_2018_idioms)]
#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::pedantic,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf
)]
#![allow(clippy::multiple_crate_versions)]

mod app;
mod central_panel;
mod display_settings;
mod document;
mod offset;
mod panels;
mod selection;
mod ui_table;
mod windows;

pub use app::WombatApp;
