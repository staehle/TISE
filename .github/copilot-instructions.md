# Copilot instructions (terra_invicta_save_editor)

## No “magic strings” in Rust UI or save logic

This repo intentionally avoids sprinkling important string literals throughout the Rust code, because typos like `"publicOpinon"` won’t fail at compile time.

### Use `tise2/src/statics.rs` for constants

- **UI strings** go in `tise2/src/statics.rs` with an `EN_` prefix.
  - Example: `EN_BTN_OPEN`, `EN_WINDOW_ABOUT`, `EN_PUBLIC_OPINION_HELPER`.
  - In code, use `statics::EN_*` rather than inline `"Open…"`, `"About"`, etc.

- **Terra Invicta save-structure keys** go in `tise2/src/statics.rs` with a `TI_` prefix.
  - Example: `TI_PROP_PUBLIC_OPINION`, `TI_PUBLIC_OPINION_UNDECIDED`, `TI_GAMESTATES`, `TI_REF_FIELD_VALUE`, `TI_REF_FIELD_TYPE`.
  - In code, use `statics::TI_*` rather than inline `"publicOpinion"`, `"Undecided"`, `"gamestates"`, `"value"`, `"$type"`, etc.

### What to do when adding/editing features

- If you need a new UI label/heading/button text: **add an `EN_*` const** in `tise2/src/statics.rs`, then reference it from the UI.
- If you need a new key/property name for parsing/formatting TI saves: **add a `TI_*` const** in `tise2/src/statics.rs`, then reference it everywhere.
- Prefer compile-time string reuse over retyping literals in `gui.rs`, `save.rs`, `value.rs`, tests, etc.

### Validation expectations

- Keep changes minimal and consistent with existing style.
- Before considering work “done”, run:
  - `cargo fmt --all`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
