//! `kit-slint`: Produktivitäts-Helfer für Rust + Slint Desktop-Anwendungen.
//!
//! Beinhaltet:
//! - **Model-Konverter**: `ToSlintModel`, `CollectSlintModel`, `slint_model!`, `slint_strings!`
//! - **Async-Tasks**: `spawn_ui_task`, `spawn_ui_task_result`, `invoke_ui`
//! - **Timer & Toasts**: `timer::single_shot`

pub use slint;
pub use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

pub mod model;
pub mod tasks;
pub mod timer;

pub use model::{CollectSlintModel, CollectSlintStrings, ToSlintModel};
pub use tasks::{invoke_ui, spawn_ui_task, spawn_ui_task_result};
