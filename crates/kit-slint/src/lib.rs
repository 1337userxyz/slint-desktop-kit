//! `kit-slint`: Produktivitäts-Helfer für Rust + Slint Desktop-Anwendungen.
//!
//! Beinhaltet:
//! - **Model-Konverter**: `ToSlintModel`, `CollectSlintModel`, `slint_model!`, `slint_strings!`
//! - **Async-Tasks**: `spawn_ui_task`, `spawn_ui_task_result`, `invoke_ui`
//! - **Timer & Toasts**: `timer::single_shot`

pub use slint;
pub use slint::{Color, ComponentHandle, Model, ModelRc, SharedString, VecModel};

pub mod color;
pub mod model;
pub mod nixos;
pub mod tasks;
pub mod timer;

pub use color::{parse_hex, parse_hex_with_alpha, to_hex, to_hex_argb, transparent, with_alpha, ColorCache};
pub use model::{CollectSlintModel, CollectSlintStrings, ToSlintModel};
pub use nixos::ensure_gui_ready;
pub use tasks::{invoke_ui, spawn_ui_task, spawn_ui_task_result};
