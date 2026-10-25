//! Core library for the Terra Invicta Save Editor (TISE).
//! Provides JSON5 parsing/serialization tailored for Terra Invicta save files, including
//! round-trip guarantees and efficient indexing.

pub mod diff;
mod gamedata;
mod gui;
mod save;
pub mod statics;
mod update;
mod value;

pub use diff::{
    DiffKind, FieldChange, IgnoreRules, ObjectDiff, SaveDiff, diff_against_path, diff_paths,
    diff_saves,
};
pub use gui::run_gui;
pub use save::{LoadedSave, SaveFormat};
pub use value::TiValue;
