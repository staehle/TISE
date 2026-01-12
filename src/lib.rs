//! Core library for the Terra Invicta Save Editor (TISE).
//! Provides JSON5 parsing/serialization tailored for Terra Invicta save files, including
//! round-trip guarantees and efficient indexing.

mod gui;
mod save;
pub mod statics;
mod value;

pub use gui::run_gui;
pub use save::{LoadedSave, SaveFormat};
pub use value::TiValue;
