use crate::diff::{self, IgnoreRules, SaveDiff};
use crate::statics;
use crate::{GroupCategory, LoadedSave, TiValue};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::OnceLock,
};

#[derive(Clone, Debug)]
enum PublicOpinionDrag {
    Divider {
        divider_index: usize,
    },
    SliceRadial {
        slice_index: usize,
        start_dist: f32,
        start_value: f64,
        start_remainder: f64,
    },
}

pub fn run_gui() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 900.0]),
        ..Default::default()
    };
    let title = format!("{} {}", statics::EN_APP_TITLE, env!("CARGO_PKG_VERSION"));
    eframe::run_native(
        &title,
        options,
        Box::new(|_cc| {
            let mut app = TiseApp {
                theme_dark: true,
                update_check: Some(crate::update::UpdateCheck::spawn()),
                ..Default::default()
            };
            app.init_game_data(crate::gamedata::Config::load());
            Ok(Box::new(app))
        }),
    )
}

/// The main application state and GUI logic.
/// Stores the LoadedSave (owned), UI state (selection, scroll), and editor buffers.
#[derive(Default)]
struct TiseApp {
    save: Option<LoadedSave>,
    dialog_dir: Option<PathBuf>,
    selected_group: Option<String>,
    selected_object_id: Option<i64>,
    selected_property: Option<String>,
    edit_buffer: String,
    raw_edit_mode: bool,
    scroll_groups_to_selected: bool,
    scroll_objects_to_selected: bool,
    scroll_properties_to_selected: bool,
    scroll_align_center: bool,
    status: String,
    last_error: Option<String>,

    // Buffers for nested editors inside structured values.
    nested_edit_buffers: std::collections::HashMap<String, String>,

    // Feature parity: navigation history + sorting + go-to-id.
    history_back: Vec<i64>,
    history_forward: Vec<i64>,
    sort_objects_by_id: bool,
    go_to_id_open: bool,
    go_to_id_input: String,
    go_to_id_request_focus: bool,

    // Undo/Redo + change descriptions.
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    changes_open: bool,

    // Feature parity: About dialog.
    about_open: bool,

    // Feature: Search & Reference Browser.
    search_ref_browser_open: bool,
    search_ref_browser_query: String,
    search_ref_browser_request_focus: bool,
    search_ref_cache: Option<Vec<i64>>,
    search_ref_cache_query: String,

    // Feature: Search Items (scan all keys/values).
    search_items_open: bool,
    search_items_query: String,
    search_items_request_focus: bool,
    search_items_sort_key: ItemSortKey,
    search_items_sort_asc: bool,
    search_items_cache: Option<Vec<ItemSearchHit>>,
    search_items_cache_query: String,

    // Feature parity: special editor for TINationState.publicOpinion.
    public_opinion_inputs: Vec<(String, String)>,
    public_opinion_remainder: Option<f64>,
    public_opinion_drag: Option<PublicOpinionDrag>,

    // Editor: change property type popup.
    change_type_open: bool,
    change_type_preview: Option<TiValue>,

    // Theme.
    theme_dark: bool,

    // Feature: filter TIHabState objects by controlling faction.
    hab_faction_filter: Option<i64>,

    // Feature: filter TIHabModuleState objects by controlling faction.
    hab_module_faction_filter: Option<i64>,

    // Feature: filter TIOrgState objects by controlling faction.
    org_faction_filter: OrgFactionFilter,

    // Feature: filter TIOrgState objects by tier.
    org_tier_filter: Option<i64>,

    // Feature: filter TIHabModuleState objects by construction completion state.
    hab_module_construction_filter: HabModuleConstructionFilter,

    // Feature: filter TICouncilorState objects by faction.
    councilor_faction_filter: Option<i64>,

    // Feature: bulk-add checked TIOrgState objects to a chosen faction.
    org_bulk_selected_ids: BTreeSet<i64>,
    org_add_faction_target: Option<i64>,

    // Feature: Compare Saves (structural diff against another save file).
    diff_open: bool,
    /// Full, unfiltered diff cached from the last Compare. The window re-derives
    /// the visible view via `SaveDiff::filtered`, so toggling ignore checkboxes
    /// never re-reads the files.
    diff_full: Option<SaveDiff>,
    /// Path of the save we compared against (for the window header).
    diff_other_path: Option<PathBuf>,
    /// "Hide per-turn noise" preset toggle (on by default).
    diff_hide_noise: bool,
    /// Positive group filter: when true, show every changed group. When false,
    /// show only the groups checked in `diff_visible_groups`.
    diff_show_all_groups: bool,
    /// Groups the user has explicitly checked to keep visible when
    /// `diff_show_all_groups` is false.
    diff_visible_groups: BTreeSet<String>,
    /// Case-insensitive search/filter text for object metadata and fields.
    diff_search: String,

    /// Startup GitHub update check. Polled each frame; renders a dismissible
    /// banner when a newer release exists. `None` until `run_gui` seeds it.
    update_check: Option<crate::update::UpdateCheck>,
    /// Set once the user dismisses the update banner, so it stays hidden for
    /// the rest of the session.
    update_dismissed: bool,

    /// Persisted settings (remembered game folder).
    app_config: crate::gamedata::Config,
    /// Background/finished load of base-game template names.
    game_data: Option<crate::gamedata::GameDataLoad>,
    game_dir_dialog_open: bool,
    game_dir_input: String,
    game_dir_error: Option<String>,
    /// Search text for the councillor trait picker.
    trait_add_filter: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum HabModuleConstructionFilter {
    #[default]
    All,
    Completed,
    InProgress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum OrgFactionFilter {
    #[default]
    All,
    NoFaction,
    Faction(i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum ItemSortKey {
    Group,
    #[default]
    Id,
    Property,
    Value,
}

#[derive(Clone, Debug)]
struct ItemSearchHit {
    group: String,
    group_display: String,
    object_id: i64,
    prop: String,
    value_preview: String,
}

#[derive(Clone, Debug)]
struct EditAction {
    group: String,
    object_id: i64,
    prop: String,
    before: Option<TiValue>,
    after: Option<TiValue>,
    description: String,
}

impl TiseApp {
    fn format_public_opinion_value(v: f64) -> String {
        // Public opinion values are usually small fractions; keep the UI readable.
        // We still preserve enough precision for typical edits.
        if !v.is_finite() {
            return v.to_string();
        }

        let mut s = format!("{v:.9}");
        if s.contains('.') {
            while s.ends_with('0') {
                s.pop();
            }
            if s.ends_with('.') {
                s.pop();
            }
        }
        if s.is_empty() { "0".to_string() } else { s }
    }

    fn public_opinion_color_for(key: &str) -> egui::Color32 {
        match key {
            statics::TI_PUBLIC_OPINION_SUBMIT => egui::Color32::from_rgb(128, 0, 128),
            statics::TI_PUBLIC_OPINION_COOPERATE => egui::Color32::from_rgb(0, 160, 0),
            statics::TI_PUBLIC_OPINION_EXPLOIT => egui::Color32::from_rgb(255, 165, 0),
            statics::TI_PUBLIC_OPINION_ESCAPE => egui::Color32::from_rgb(255, 215, 0),
            statics::TI_PUBLIC_OPINION_RESIST => egui::Color32::from_rgb(0, 120, 255),
            statics::TI_PUBLIC_OPINION_DESTROY => egui::Color32::from_rgb(220, 0, 0),
            _ => {
                // Deterministic fallback color (for any other keys mods might add).
                let mut h = 2166136261u32;
                for b in key.as_bytes() {
                    h ^= u32::from(*b);
                    h = h.wrapping_mul(16777619);
                }
                let r = 64 + (h & 0x7f) as u8;
                let g = 64 + ((h >> 8) & 0x7f) as u8;
                let b = 64 + ((h >> 16) & 0x7f) as u8;
                egui::Color32::from_rgb(r, g, b)
            }
        }
    }

    fn render_public_opinion_pie(
        &mut self,
        ui: &mut egui::Ui,
        keys: &[String],
        values: &mut [f64],
        remainder: &mut f64,
        enabled: bool,
    ) -> bool {
        let side = ui.available_width().clamp(160.0, 260.0);
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(side, side),
            if enabled {
                egui::Sense::click_and_drag()
            } else {
                egui::Sense::hover()
            },
        );
        let response = response.on_hover_text(statics::EN_PUBLIC_OPINION_CHART_HINT);

        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.5 - 6.0;
        let inner_radius = 0.0;

        // Build slice list: keys + undecided.
        let mut slice_keys: Vec<&str> = keys.iter().map(String::as_str).collect();
        slice_keys.push(statics::TI_PUBLIC_OPINION_UNDECIDED);

        let mut slice_values: Vec<f64> = values.to_vec();
        slice_values.push(*remainder);

        let total: f64 = slice_values.iter().copied().sum();
        let valid_total = (total - 1.0).abs() < 1e-6;

        // Draw slices even if disabled, but only enable interaction when valid.
        let can_interact =
            enabled && valid_total && *remainder >= 0.0 && slice_values.iter().all(|v| *v >= 0.0);

        let start_angle = -std::f32::consts::FRAC_PI_2;
        let painter = ui.painter_at(rect);

        // Compute cumulative fractions for drawing + handle locations.
        let mut cumulative = 0.0f64;
        let mut slice_bounds: Vec<(f64, f64)> = Vec::with_capacity(slice_values.len());
        for v in &slice_values {
            let s = cumulative;
            cumulative += *v;
            let e = cumulative;
            slice_bounds.push((s, e));
        }

        for (i, (k, (s, e))) in slice_keys.iter().zip(slice_bounds.iter()).enumerate() {
            let frac_s = *s;
            let frac_e = *e;
            if frac_e <= frac_s {
                continue;
            }

            let a0 = start_angle + (frac_s as f32) * std::f32::consts::TAU;
            let a1 = start_angle + (frac_e as f32) * std::f32::consts::TAU;

            let color = if *k == statics::TI_PUBLIC_OPINION_UNDECIDED {
                egui::Color32::from_gray(90)
            } else {
                Self::public_opinion_color_for(k)
            };

            let arc = (a1 - a0).abs().max(0.0001);
            let steps = ((arc / std::f32::consts::TAU) * 80.0).ceil() as usize;
            let steps = steps.clamp(6, 120);
            let mut points = Vec::with_capacity(steps + 3);
            points.push(center);
            for j in 0..=steps {
                let t = j as f32 / steps as f32;
                let a = a0 + (a1 - a0) * t;
                let dir = egui::vec2(a.cos(), a.sin());
                points.push(center + dir * radius);
            }
            painter.add(egui::Shape::convex_polygon(
                points,
                color,
                egui::Stroke::new(1.0_f32, ui.visuals().window_stroke().color),
            ));

            // Optional label for larger slices.
            let frac = (frac_e - frac_s).max(0.0);
            if frac >= 0.08 {
                let mid = (frac_s + frac_e) * 0.5;
                let am = start_angle + (mid as f32) * std::f32::consts::TAU;
                let label_pos = center + egui::vec2(am.cos(), am.sin()) * (radius * 0.55);
                let pct = (frac * 100.0).round() as i64;
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    format!("{pct}%"),
                    egui::TextStyle::Small.resolve(ui.style()),
                    ui.visuals().text_color(),
                );
            }

            // Draw divider handles between editable slices and next slice (including Undecided).
            if can_interact && i < slice_values.len() - 1 {
                let boundary = frac_e;
                let ab = start_angle + (boundary as f32) * std::f32::consts::TAU;
                let handle_pos = center + egui::vec2(ab.cos(), ab.sin()) * (radius * 0.92);
                painter.add(egui::Shape::circle_filled(
                    handle_pos,
                    4.0,
                    ui.visuals().widgets.active.bg_fill,
                ));
            }
        }

        // Interaction: divider drag (re-balance two adjacent slices) and slice radial drag (trade with Undecided).
        let mut changed = false;
        if can_interact {
            let pointer_pos = ui
                .ctx()
                .input(|i| i.pointer.interact_pos())
                .filter(|p| rect.contains(*p));

            if response.drag_started()
                && let Some(p) = pointer_pos
            {
                let v = p - center;
                let dist = v.length();
                if dist >= inner_radius && dist <= radius + 10.0 {
                    // Check divider handles first.
                    let mut selected_divider: Option<usize> = None;
                    for (idx, (_s, e)) in
                        slice_bounds.iter().enumerate().take(slice_values.len() - 1)
                    {
                        let ab = start_angle + (*e as f32) * std::f32::consts::TAU;
                        let handle_pos = center + egui::vec2(ab.cos(), ab.sin()) * (radius * 0.92);
                        if handle_pos.distance(p) <= 10.0 {
                            selected_divider = Some(idx);
                            break;
                        }
                    }

                    if let Some(divider_index) = selected_divider {
                        self.public_opinion_drag =
                            Some(PublicOpinionDrag::Divider { divider_index });
                    } else if dist <= radius {
                        // Pick slice by angle.
                        let mut angle = v.y.atan2(v.x);
                        // Convert to fraction from start_angle in [0, 1).
                        angle -= start_angle;
                        while angle < 0.0 {
                            angle += std::f32::consts::TAU;
                        }
                        while angle >= std::f32::consts::TAU {
                            angle -= std::f32::consts::TAU;
                        }
                        let frac = (angle / std::f32::consts::TAU) as f64;

                        let mut slice_index: Option<usize> = None;
                        for (idx, (s, e)) in slice_bounds.iter().enumerate() {
                            if frac >= *s && frac < *e {
                                slice_index = Some(idx);
                                break;
                            }
                        }

                        // Radial drag only for real slices (not Undecided).
                        if let Some(slice_index) = slice_index
                            && slice_index < values.len()
                        {
                            self.public_opinion_drag = Some(PublicOpinionDrag::SliceRadial {
                                slice_index,
                                start_dist: dist,
                                start_value: values[slice_index],
                                start_remainder: *remainder,
                            });
                        }
                    }
                }
            }

            if response.drag_stopped() {
                self.public_opinion_drag = None;
            }

            if response.dragged()
                && let (Some(p), Some(drag)) = (pointer_pos, self.public_opinion_drag.clone())
            {
                let v = p - center;
                let dist = v.length();
                match drag {
                    PublicOpinionDrag::Divider { divider_index } => {
                        // Convert pointer angle to fraction in [0,1).
                        let mut angle = v.y.atan2(v.x);
                        angle -= start_angle;
                        while angle < 0.0 {
                            angle += std::f32::consts::TAU;
                        }
                        while angle >= std::f32::consts::TAU {
                            angle -= std::f32::consts::TAU;
                        }
                        let frac = (angle / std::f32::consts::TAU) as f64;

                        let a_idx = divider_index;
                        let b_idx = divider_index + 1;

                        let prev_cum = slice_bounds[a_idx].0;
                        let next_cum = slice_bounds[b_idx].1;
                        let pair_sum = next_cum - prev_cum;
                        if pair_sum > 0.0 {
                            let min = 0.0001;
                            let new_cum = frac.clamp(prev_cum + min, next_cum - min);
                            let new_a = new_cum - prev_cum;
                            let new_b = pair_sum - new_a;

                            if a_idx < values.len() {
                                values[a_idx] = new_a;
                            } else {
                                *remainder = new_a;
                            }
                            if b_idx < values.len() {
                                values[b_idx] = new_b;
                            } else {
                                *remainder = new_b;
                            }
                            changed = true;
                        }
                    }
                    PublicOpinionDrag::SliceRadial {
                        slice_index,
                        start_dist,
                        start_value,
                        start_remainder,
                    } => {
                        let delta = ((dist - start_dist) / radius) as f64;
                        let mut new_value = start_value + delta;
                        new_value = new_value.clamp(0.0, start_value + start_remainder);
                        let delta_value = new_value - start_value;
                        values[slice_index] = new_value;
                        *remainder = (start_remainder - delta_value).clamp(0.0, 1.0);
                        changed = true;
                    }
                }

                if changed {
                    // Ensure invariants.
                    for v in values.iter_mut() {
                        if !v.is_finite() {
                            *v = 0.0;
                        }
                        *v = (*v).clamp(0.0, 1.0);
                    }
                    if !remainder.is_finite() {
                        *remainder = 0.0;
                    }
                    *remainder = (*remainder).clamp(0.0, 1.0);
                    let sum_vals: f64 = values.iter().copied().sum();
                    let r = (1.0 - sum_vals).clamp(0.0, 1.0);
                    *remainder = r;
                }
            }
        }

        let _ = response;
        changed
    }
    fn sort_item_search_hits(hits: &mut [ItemSearchHit], key: ItemSortKey, asc: bool) {
        hits.sort_by(|a, b| {
            use std::cmp::Ordering;

            let ord = match key {
                ItemSortKey::Group => a
                    .group_display
                    .to_lowercase()
                    .cmp(&b.group_display.to_lowercase())
                    .then_with(|| a.object_id.cmp(&b.object_id))
                    .then_with(|| a.prop.to_lowercase().cmp(&b.prop.to_lowercase()))
                    .then_with(|| {
                        a.value_preview
                            .to_lowercase()
                            .cmp(&b.value_preview.to_lowercase())
                    }),
                ItemSortKey::Id => a
                    .object_id
                    .cmp(&b.object_id)
                    .then_with(|| {
                        a.group_display
                            .to_lowercase()
                            .cmp(&b.group_display.to_lowercase())
                    })
                    .then_with(|| a.prop.to_lowercase().cmp(&b.prop.to_lowercase()))
                    .then_with(|| {
                        a.value_preview
                            .to_lowercase()
                            .cmp(&b.value_preview.to_lowercase())
                    }),
                ItemSortKey::Property => a
                    .prop
                    .to_lowercase()
                    .cmp(&b.prop.to_lowercase())
                    .then_with(|| {
                        a.group_display
                            .to_lowercase()
                            .cmp(&b.group_display.to_lowercase())
                    })
                    .then_with(|| a.object_id.cmp(&b.object_id))
                    .then_with(|| {
                        a.value_preview
                            .to_lowercase()
                            .cmp(&b.value_preview.to_lowercase())
                    }),
                ItemSortKey::Value => a
                    .value_preview
                    .to_lowercase()
                    .cmp(&b.value_preview.to_lowercase())
                    .then_with(|| {
                        a.group_display
                            .to_lowercase()
                            .cmp(&b.group_display.to_lowercase())
                    })
                    .then_with(|| a.object_id.cmp(&b.object_id))
                    .then_with(|| a.prop.to_lowercase().cmp(&b.prop.to_lowercase())),
            };

            if asc {
                ord
            } else {
                match ord {
                    Ordering::Less => Ordering::Greater,
                    Ordering::Equal => Ordering::Equal,
                    Ordering::Greater => Ordering::Less,
                }
            }
        });
    }

    fn item_value_contains_query(val: &TiValue, query_lower: &str) -> bool {
        match val {
            TiValue::Null => statics::EN_LITERAL_NULL.contains(query_lower),
            TiValue::Bool(b) => b.to_string().to_lowercase().contains(query_lower),
            TiValue::Number(n) => {
                let s = TiValue::Number(n.clone()).to_json5_compact();
                s.to_lowercase().contains(query_lower)
            }
            TiValue::String(s) => s.to_lowercase().contains(query_lower),
            TiValue::Array(values) => values
                .iter()
                .any(|v| Self::item_value_contains_query(v, query_lower)),
            TiValue::Object(map) => map.iter().any(|(k, v)| {
                k.to_lowercase().contains(query_lower)
                    || Self::item_value_contains_query(v, query_lower)
            }),
        }
    }

    fn compute_item_search_hits(
        save: &LoadedSave,
        query: &str,
        max_results: usize,
    ) -> Vec<ItemSearchHit> {
        let query = query.trim();
        if query.is_empty() {
            return Vec::new();
        }
        let query_lower = query.to_lowercase();

        let mut hits = Vec::new();
        for group in &save.index.groups {
            let group_display = LoadedSave::group_display_name(group).to_string();
            let Some(objs) = save.index.objects_by_group.get(group) else {
                continue;
            };
            for obj in objs {
                let Some(value_obj) = save.get_object_value(group, obj.id) else {
                    continue;
                };
                for (k, v) in value_obj.iter() {
                    let key_match = k.to_lowercase().contains(&query_lower);
                    let value_match = Self::item_value_contains_query(v, &query_lower);
                    if !key_match && !value_match {
                        continue;
                    }

                    hits.push(ItemSearchHit {
                        group: group.clone(),
                        group_display: group_display.clone(),
                        object_id: obj.id,
                        prop: k.clone(),
                        value_preview: value_preview(v),
                    });

                    if hits.len() >= max_results {
                        return hits;
                    }
                }
            }
        }

        hits
    }

    fn selectable_row_left(
        ui: &mut egui::Ui,
        selected: bool,
        text: &str,
        row_h: f32,
    ) -> egui::Response {
        let w = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(egui::vec2(w, row_h), egui::Sense::click());
        let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

        let visuals = ui.style().interact_selectable(&response, selected);
        if ui.is_rect_visible(rect) {
            ui.painter()
                .rect_filled(rect, visuals.corner_radius, visuals.bg_fill);
            ui.painter().rect_stroke(
                rect,
                visuals.corner_radius,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );

            let font_id = egui::TextStyle::Button.resolve(ui.style());
            let text_pos = rect.left_center() + egui::vec2(6.0, 0.0);
            ui.painter().text(
                text_pos,
                egui::Align2::LEFT_CENTER,
                text,
                font_id,
                visuals.text_color(),
            );
        }

        response
    }

    fn refresh_selected_property_from_save(&mut self, save: &LoadedSave) {
        let (Some(group), Some(object_id), Some(prop)) = (
            self.selected_group.clone(),
            self.selected_object_id,
            self.selected_property.clone(),
        ) else {
            return;
        };

        let (obj_clone, val_clone) = {
            let Some(obj) = save.get_object_value(&group, object_id) else {
                return;
            };
            (obj.clone(), obj.get(&prop).cloned())
        };

        let Some(val) = val_clone.as_ref() else {
            // Property no longer exists.
            self.selected_property = None;
            self.edit_buffer.clear();
            self.raw_edit_mode = false;
            self.public_opinion_inputs.clear();
            self.public_opinion_remainder = None;
            return;
        };

        self.raw_edit_mode = matches!(val, TiValue::Array(_) | TiValue::Object(_))
            && val.is_relational_ref().is_none();

        self.edit_buffer = if val.is_relational_ref().is_some() {
            val.to_json5_compact()
        } else if self.raw_edit_mode {
            val.to_ti_save_pretty()
        } else {
            val.to_json5_compact()
        };

        // Structured editor nested buffers are derived from the current value.
        let prefix = format!("{prop}::");
        self.nested_edit_buffers
            .retain(|k, _| !k.as_str().starts_with(prefix.as_str()));

        // The change-type preview (if open) is tied to the current value.
        self.change_type_preview = None;

        self.refresh_public_opinion_editor(&obj_clone, &prop);
    }

    fn navigate_to_action_target(&mut self, save: &LoadedSave, action: &EditAction) {
        self.select_object_programmatic(&action.group, action.object_id, true, true);
        self.selected_property = Some(action.prop.clone());
        self.scroll_properties_to_selected = true;
        self.scroll_align_center = true;
        self.refresh_selected_property_from_save(save);
    }

    fn describe_change(prop: &str, before: Option<&TiValue>, after: Option<&TiValue>) -> String {
        let b = before
            .map(|v| v.type_name())
            .unwrap_or(statics::EN_LITERAL_MISSING);
        let a = after
            .map(|v| v.type_name())
            .unwrap_or(statics::EN_LITERAL_MISSING);
        if let Some(TiValue::Null) = after {
            format!("Set '{prop}' to null")
        } else if b != a {
            format!("Changed '{prop}' {b} -> {a}")
        } else {
            format!("Updated '{prop}'")
        }
    }

    fn apply_action_to_save(save: &mut LoadedSave, action: &EditAction, use_after: bool) -> bool {
        let target = if use_after {
            action.after.clone()
        } else {
            action.before.clone()
        };

        let Some(obj) = save.get_object_value_mut(&action.group, action.object_id) else {
            return false;
        };

        match target {
            Some(v) => {
                obj.insert(action.prop.clone(), v);
            }
            None => {
                obj.shift_remove(&action.prop);
            }
        }

        save.rebuild_index();
        save.refresh_dirty();
        true
    }

    fn record_action(&mut self, action: EditAction) {
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        let Some(action) = self.undo_stack.pop() else {
            return;
        };

        if self.save.is_none() {
            self.undo_stack.push(action);
            return;
        };

        let applied = {
            let save = self.save.as_mut().expect("checked above");
            Self::apply_action_to_save(save, &action, false)
        };

        if applied {
            self.status = format!("{} {}", statics::EN_PREFIX_UNDO, action.description);
            self.last_error = None;
            let save = self.save.take().unwrap();
            self.navigate_to_action_target(&save, &action);
            self.save = Some(save);
            self.redo_stack.push(action);
        } else {
            self.last_error = Some(statics::EN_ERR_LOCATE_SELECTED_OBJECT.to_string());
            // put it back so we don't lose history on failure
            self.undo_stack.push(action);
        }
    }

    fn redo(&mut self) {
        let Some(action) = self.redo_stack.pop() else {
            return;
        };

        if self.save.is_none() {
            self.redo_stack.push(action);
            return;
        };

        let applied = {
            let save = self.save.as_mut().expect("checked above");
            Self::apply_action_to_save(save, &action, true)
        };

        if applied {
            self.status = format!("{} {}", statics::EN_PREFIX_REDO, action.description);
            self.last_error = None;
            let save = self.save.take().unwrap();
            self.navigate_to_action_target(&save, &action);
            self.save = Some(save);
            self.undo_stack.push(action);
        } else {
            self.last_error = Some(statics::EN_ERR_LOCATE_SELECTED_OBJECT.to_string());
            self.redo_stack.push(action);
        }
    }
    fn value_for_editing(val: &TiValue) -> String {
        if matches!(val, TiValue::Array(_) | TiValue::Object(_))
            && val.is_relational_ref().is_none()
        {
            val.to_ti_save_pretty()
        } else {
            val.to_json5_compact()
        }
    }

    fn as_f64_lossy(n: &crate::value::TiNumber) -> f64 {
        match n {
            crate::value::TiNumber::I64(v) => *v as f64,
            crate::value::TiNumber::U64(v) => *v as f64,
            crate::value::TiNumber::F64(v) => *v,
        }
    }

    fn parse_number_like(text: &str) -> Option<crate::value::TiNumber> {
        match TiValue::parse_json5(text.trim()).ok()? {
            TiValue::Number(n) => Some(n),
            _ => None,
        }
    }

    fn coerce_to_bool(src: &TiValue) -> bool {
        match src {
            TiValue::Bool(b) => *b,
            TiValue::Number(n) => Self::as_f64_lossy(n) != 0.0,
            TiValue::String(s) => {
                let t = s.trim().to_ascii_lowercase();
                matches!(t.as_str(), "true" | "1" | "yes" | "y")
            }
            _ => false,
        }
    }

    fn coerce_to_i64(src: &TiValue) -> i64 {
        match src {
            TiValue::Number(crate::value::TiNumber::I64(v)) => *v,
            TiValue::Number(crate::value::TiNumber::U64(v)) => i64::try_from(*v).unwrap_or(0),
            TiValue::Number(crate::value::TiNumber::F64(v)) => {
                if v.is_finite() {
                    *v as i64
                } else {
                    0
                }
            }
            TiValue::Bool(b) => i64::from(*b),
            TiValue::String(s) => s.trim().parse::<i64>().unwrap_or(0),
            _ => 0,
        }
    }

    fn coerce_to_u64(src: &TiValue) -> u64 {
        match src {
            TiValue::Number(crate::value::TiNumber::U64(v)) => *v,
            TiValue::Number(crate::value::TiNumber::I64(v)) => u64::try_from(*v).unwrap_or(0),
            TiValue::Number(crate::value::TiNumber::F64(v)) => {
                if v.is_finite() && *v >= 0.0 {
                    *v as u64
                } else {
                    0
                }
            }
            TiValue::Bool(b) => u64::from(*b),
            TiValue::String(s) => s.trim().parse::<u64>().unwrap_or(0),
            _ => 0,
        }
    }

    fn coerce_to_f64(src: &TiValue) -> f64 {
        match src {
            TiValue::Number(n) => Self::as_f64_lossy(n),
            TiValue::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            TiValue::String(s) => {
                if let Some(n) = Self::parse_number_like(s) {
                    Self::as_f64_lossy(&n)
                } else {
                    s.trim().parse::<f64>().unwrap_or(0.0)
                }
            }
            _ => 0.0,
        }
    }

    fn coerce_to_string(src: &TiValue) -> String {
        match src {
            TiValue::String(s) => s.clone(),
            TiValue::Null => String::new(),
            TiValue::Bool(b) => b.to_string(),
            TiValue::Number(n) => TiValue::Number(n.clone()).to_json5_compact(),
            _ => String::new(),
        }
    }

    fn empty_object() -> TiValue {
        TiValue::Object(indexmap::IndexMap::new())
    }

    fn empty_array() -> TiValue {
        TiValue::Array(Vec::new())
    }

    fn coerce_to_reference(src: &TiValue) -> TiValue {
        // Our ref detector treats { value: <int> } as a ref; $type is optional.
        let target_id = match src {
            TiValue::Number(n) => n.as_i64().unwrap_or(0),
            TiValue::Bool(b) => i64::from(*b),
            TiValue::String(s) => s.trim().parse::<i64>().unwrap_or(0),
            TiValue::Object(map) => map
                .get(statics::TI_REF_FIELD_VALUE)
                .and_then(|v| match v {
                    TiValue::Number(n) => n.as_i64(),
                    _ => None,
                })
                .unwrap_or(0),
            _ => 0,
        };

        let mut map = indexmap::IndexMap::new();
        map.insert(
            statics::TI_REF_FIELD_VALUE.to_string(),
            TiValue::Number(crate::value::TiNumber::I64(target_id)),
        );
        TiValue::Object(map)
    }

    fn coerce_value_to_type(label: &str, src: &TiValue) -> TiValue {
        match label {
            // Using statics labels as the stable selector.
            l if l == statics::EN_TYPE_NULL => TiValue::Null,
            l if l == statics::EN_TYPE_BOOL => TiValue::Bool(Self::coerce_to_bool(src)),
            l if l == statics::EN_TYPE_I64 => {
                TiValue::Number(crate::value::TiNumber::I64(Self::coerce_to_i64(src)))
            }
            l if l == statics::EN_TYPE_U64 => {
                TiValue::Number(crate::value::TiNumber::U64(Self::coerce_to_u64(src)))
            }
            l if l == statics::EN_TYPE_F64 => {
                TiValue::Number(crate::value::TiNumber::F64(Self::coerce_to_f64(src)))
            }
            l if l == statics::EN_TYPE_STRING => TiValue::String(Self::coerce_to_string(src)),
            l if l == statics::EN_TYPE_ARRAY => match src {
                TiValue::Array(v) => TiValue::Array(v.clone()),
                TiValue::Null => Self::empty_array(),
                TiValue::Bool(_) | TiValue::Number(_) | TiValue::String(_) => {
                    TiValue::Array(vec![src.clone()])
                }
                _ => Self::empty_array(),
            },
            l if l == statics::EN_TYPE_OBJECT => match src {
                TiValue::Object(map) => TiValue::Object(map.clone()),
                _ => Self::empty_object(),
            },
            l if l == statics::EN_TYPE_REFERENCE => Self::coerce_to_reference(src),
            _ => src.clone(),
        }
    }
    fn is_simple_object(map: &indexmap::IndexMap<String, TiValue>) -> bool {
        if map.is_empty() {
            return false;
        }
        map.values().all(|v| {
            matches!(
                v,
                TiValue::Null | TiValue::Bool(_) | TiValue::Number(_) | TiValue::String(_)
            )
        })
    }

    fn render_simple_object_editor(
        ui: &mut egui::Ui,
        map: &mut indexmap::IndexMap<String, TiValue>,
    ) -> bool {
        let mut changed_any = false;
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 6.0;

        ui.push_id("simple_object_table", |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(220.0).resizable(true))
                .column(Column::remainder().resizable(true))
                .header(row_h, |mut header| {
                    header.col(|ui| {
                        ui.strong(statics::EN_COL_KEY);
                    });
                    header.col(|ui| {
                        ui.strong(statics::EN_COL_VALUE);
                    });
                })
                .body(|mut body| {
                    for (k, v) in map.iter_mut() {
                        body.row(row_h, |mut row| {
                            row.col(|ui| {
                                ui.monospace(k);
                            });
                            row.col(|ui| {
                                let changed = match v {
                                    TiValue::Null => {
                                        ui.add_enabled(
                                            false,
                                            egui::Label::new(statics::EN_LITERAL_NULL),
                                        );
                                        false
                                    }
                                    TiValue::Bool(b) => ui.checkbox(b, "").changed(),
                                    TiValue::String(s) => ui
                                        .add(
                                            egui::TextEdit::singleline(s)
                                                .desired_width(ui.available_width()),
                                        )
                                        .changed(),
                                    TiValue::Number(n) => match n {
                                        crate::value::TiNumber::I64(x) => {
                                            let mut tmp = *x;
                                            let resp =
                                                ui.add(egui::DragValue::new(&mut tmp).speed(1));
                                            if resp.changed() {
                                                *x = tmp;
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                        crate::value::TiNumber::U64(x) => {
                                            let mut tmp = *x;
                                            let resp =
                                                ui.add(egui::DragValue::new(&mut tmp).speed(1));
                                            if resp.changed() {
                                                *x = tmp;
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                        crate::value::TiNumber::F64(x) => {
                                            let mut tmp = *x;
                                            let resp = ui.add(
                                                egui::DragValue::new(&mut tmp)
                                                    .speed(0.1)
                                                    .range(f64::NEG_INFINITY..=f64::INFINITY),
                                            );
                                            if resp.changed() {
                                                *x = tmp;
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                    },
                                    // Non-primitive values should not reach this editor.
                                    _ => false,
                                };
                                if changed {
                                    changed_any = true;
                                }
                            });
                        });
                    }
                });
        });

        changed_any
    }

    fn is_simple_list(arr: &[TiValue]) -> bool {
        arr.iter().all(|v| {
            matches!(
                v,
                TiValue::Null | TiValue::Bool(_) | TiValue::Number(_) | TiValue::String(_)
            )
        })
    }

    /// The primitive type labels selectable per-row in the simple list editor.
    /// Kept to primitives so converting a row never knocks the list out of the
    /// "simple list" shape that this editor handles.
    const SIMPLE_LIST_TYPE_LABELS: [&'static str; 6] = [
        statics::EN_TYPE_NULL,
        statics::EN_TYPE_BOOL,
        statics::EN_TYPE_I64,
        statics::EN_TYPE_U64,
        statics::EN_TYPE_F64,
        statics::EN_TYPE_STRING,
    ];

    /// Map a primitive value to the stable type label used by the type pickers.
    /// Non-primitives never appear in a simple list, but we fall back to `null`.
    fn primitive_type_label(v: &TiValue) -> &'static str {
        match v {
            TiValue::Null => statics::EN_TYPE_NULL,
            TiValue::Bool(_) => statics::EN_TYPE_BOOL,
            TiValue::Number(crate::value::TiNumber::I64(_)) => statics::EN_TYPE_I64,
            TiValue::Number(crate::value::TiNumber::U64(_)) => statics::EN_TYPE_U64,
            TiValue::Number(crate::value::TiNumber::F64(_)) => statics::EN_TYPE_F64,
            TiValue::String(_) => statics::EN_TYPE_STRING,
            _ => statics::EN_TYPE_NULL,
        }
    }

    /// Determine the most common primitive type label in a list.
    /// Ties are broken in favor of the type that appears earliest in the list.
    /// Returns `None` for an empty list.
    fn majority_primitive_label(arr: &[TiValue]) -> Option<&'static str> {
        // (label, count, first-seen index). Small N, so a linear scan beats a map.
        let mut counts: Vec<(&'static str, usize, usize)> = Vec::new();
        for (i, v) in arr.iter().enumerate() {
            let label = Self::primitive_type_label(v);
            if let Some(entry) = counts.iter_mut().find(|(l, _, _)| *l == label) {
                entry.1 += 1;
            } else {
                counts.push((label, 1, i));
            }
        }
        // Highest count wins; on a tie the smaller first-seen index wins.
        counts
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1).then(b.2.cmp(&a.2)))
            .map(|(label, _, _)| label)
    }

    /// Build the default value for a newly added/inserted list item, defaulting
    /// to a zero value of the list's majority type (empty list -> null).
    fn default_new_list_item(arr: &[TiValue]) -> TiValue {
        match Self::majority_primitive_label(arr) {
            Some(label) => Self::coerce_value_to_type(label, &TiValue::Null),
            None => TiValue::Null,
        }
    }

    fn render_simple_list_editor(ui: &mut egui::Ui, arr: &mut Vec<TiValue>) -> bool {
        let mut changed_any = false;
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 6.0;

        enum ListOp {
            Delete(usize),
            Insert(usize),
            MoveUp(usize),
            MoveDown(usize),
        }

        let mut op: Option<ListOp> = None;

        ui.push_id("simple_list_table", |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(60.0).resizable(false))
                .column(Column::remainder().resizable(true))
                .column(Column::initial(80.0).resizable(false))
                .column(Column::initial(140.0).resizable(false))
                .header(row_h, |mut header| {
                    header.col(|ui| {
                        ui.strong(statics::EN_COL_INDEX);
                    });
                    header.col(|ui| {
                        ui.strong(statics::EN_COL_VALUE);
                    });
                    header.col(|ui| {
                        ui.strong(statics::EN_COL_TYPE);
                    });
                    header.col(|ui| {
                        ui.strong("");
                    });
                })
                .body(|mut body| {
                    for (idx, v) in arr.iter_mut().enumerate() {
                        body.row(row_h, |mut row| {
                            row.col(|ui| {
                                ui.monospace(idx.to_string());
                            });
                            row.col(|ui| {
                                let changed = match v {
                                    TiValue::Null => {
                                        ui.add_enabled(
                                            false,
                                            egui::Label::new(statics::EN_LITERAL_NULL),
                                        );
                                        false
                                    }
                                    TiValue::Bool(b) => ui.checkbox(b, "").changed(),
                                    TiValue::String(s) => ui
                                        .add(
                                            egui::TextEdit::singleline(s)
                                                .desired_width(ui.available_width()),
                                        )
                                        .changed(),
                                    TiValue::Number(n) => match n {
                                        crate::value::TiNumber::I64(x) => {
                                            let mut tmp = *x;
                                            let resp =
                                                ui.add(egui::DragValue::new(&mut tmp).speed(1));
                                            if resp.changed() {
                                                *x = tmp;
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                        crate::value::TiNumber::U64(x) => {
                                            let mut tmp = *x;
                                            let resp =
                                                ui.add(egui::DragValue::new(&mut tmp).speed(1));
                                            if resp.changed() {
                                                *x = tmp;
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                        crate::value::TiNumber::F64(x) => {
                                            let mut tmp = *x;
                                            let resp = ui.add(
                                                egui::DragValue::new(&mut tmp)
                                                    .speed(0.1)
                                                    .range(f64::NEG_INFINITY..=f64::INFINITY),
                                            );
                                            if resp.changed() {
                                                *x = tmp;
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                    },
                                    // Non-primitive values should not reach this editor.
                                    _ => false,
                                };
                                if changed {
                                    changed_any = true;
                                }
                            });
                            row.col(|ui| {
                                let current_label = Self::primitive_type_label(v);
                                egui::ComboBox::from_id_salt(("simple_list_type", idx))
                                    .selected_text(current_label)
                                    .show_ui(ui, |ui| {
                                        for label in Self::SIMPLE_LIST_TYPE_LABELS {
                                            if ui
                                                .selectable_label(current_label == label, label)
                                                .clicked()
                                                && current_label != label
                                            {
                                                *v = Self::coerce_value_to_type(label, v);
                                                changed_any = true;
                                            }
                                        }
                                    });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    if ui.small_button(statics::EN_BTN_INSERT).clicked() {
                                        op = Some(ListOp::Insert(idx));
                                    }
                                    if ui.small_button(statics::EN_BTN_UP).clicked() {
                                        op = Some(ListOp::MoveUp(idx));
                                    }
                                    if ui.small_button(statics::EN_BTN_DOWN).clicked() {
                                        op = Some(ListOp::MoveDown(idx));
                                    }
                                    if ui.small_button(statics::EN_BTN_DELETE).clicked() {
                                        op = Some(ListOp::Delete(idx));
                                    }
                                });
                            });
                        });
                    }
                });
        });

        if let Some(op) = op {
            match op {
                ListOp::Delete(idx) => {
                    if idx < arr.len() {
                        arr.remove(idx);
                        changed_any = true;
                    }
                }
                ListOp::Insert(idx) => {
                    if idx <= arr.len() {
                        let item = Self::default_new_list_item(arr);
                        arr.insert(idx, item);
                        changed_any = true;
                    }
                }
                ListOp::MoveUp(idx) => {
                    if idx > 0 && idx < arr.len() {
                        arr.swap(idx, idx - 1);
                        changed_any = true;
                    }
                }
                ListOp::MoveDown(idx) => {
                    if idx + 1 < arr.len() {
                        arr.swap(idx, idx + 1);
                        changed_any = true;
                    }
                }
            }
        }

        ui.horizontal(|ui| {
            if ui.button(statics::EN_BTN_ADD_ITEM).clicked() {
                let item = Self::default_new_list_item(arr);
                arr.push(item);
                changed_any = true;
            }
        });

        changed_any
    }

    fn nested_buffer_key(prop: &str, key: &str) -> String {
        format!("{prop}::{key}")
    }

    fn render_mixed_object_editor(
        &mut self,
        ui: &mut egui::Ui,
        prop: &str,
        map: &mut indexmap::IndexMap<String, TiValue>,
    ) -> bool {
        let mut changed_any = false;
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 6.0;

        // Primitive fields in a compact table.
        ui.push_id(("mixed_object_table", prop), |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(220.0).resizable(true))
                .column(Column::remainder().resizable(true))
                .column(Column::initial(80.0).resizable(false))
                .header(row_h, |mut header| {
                    header.col(|ui| {
                        ui.strong(statics::EN_COL_KEY);
                    });
                    header.col(|ui| {
                        ui.strong(statics::EN_COL_VALUE);
                    });
                    header.col(|ui| {
                        ui.strong(statics::EN_COL_TYPE);
                    });
                })
                .body(|mut body| {
                    for (k, v) in map.iter_mut() {
                        let is_primitive = matches!(
                            v,
                            TiValue::Null
                                | TiValue::Bool(_)
                                | TiValue::Number(_)
                                | TiValue::String(_)
                        );
                        if !is_primitive {
                            continue;
                        }

                        body.row(row_h, |mut row| {
                            row.col(|ui| {
                                ui.monospace(k);
                            });
                            row.col(|ui| {
                                let changed = match v {
                                    TiValue::Null => {
                                        ui.add_enabled(
                                            false,
                                            egui::Label::new(statics::EN_LITERAL_NULL),
                                        );
                                        false
                                    }
                                    TiValue::Bool(b) => ui.checkbox(b, "").changed(),
                                    TiValue::String(s) => ui
                                        .add(
                                            egui::TextEdit::singleline(s)
                                                .desired_width(ui.available_width()),
                                        )
                                        .changed(),
                                    TiValue::Number(n) => match n {
                                        crate::value::TiNumber::I64(x) => {
                                            let mut tmp = *x;
                                            let resp =
                                                ui.add(egui::DragValue::new(&mut tmp).speed(1));
                                            if resp.changed() {
                                                *x = tmp;
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                        crate::value::TiNumber::U64(x) => {
                                            let mut tmp = *x;
                                            let resp =
                                                ui.add(egui::DragValue::new(&mut tmp).speed(1));
                                            if resp.changed() {
                                                *x = tmp;
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                        crate::value::TiNumber::F64(x) => {
                                            let mut tmp = *x;
                                            let resp = ui.add(
                                                egui::DragValue::new(&mut tmp)
                                                    .speed(0.1)
                                                    .range(f64::NEG_INFINITY..=f64::INFINITY),
                                            );
                                            if resp.changed() {
                                                *x = tmp;
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                    },
                                    _ => false,
                                };
                                if changed {
                                    changed_any = true;
                                }
                            });
                            row.col(|ui| {
                                ui.monospace(v.type_name());
                            });
                        });
                    }
                });
        });

        ui.separator();

        // Structured fields with per-key nested JSON5 editor.
        for (k, v) in map.iter_mut() {
            if matches!(v, TiValue::Array(_) | TiValue::Object(_)) {
                let header = format!("{} ({})", k, v.type_name());
                ui.collapsing(header, |ui| {
                    let buf_key = Self::nested_buffer_key(prop, k);
                    let default_text = match v {
                        TiValue::Array(_) | TiValue::Object(_) => v.to_ti_save_pretty(),
                        _ => v.to_json5_compact(),
                    };
                    let buf = self
                        .nested_edit_buffers
                        .entry(buf_key.clone())
                        .or_insert(default_text);

                    ui.label(statics::EN_LABEL_JSON5);
                    let editor_h = (ui.available_height() * 0.6).clamp(120.0, 420.0);
                    ui.add_sized(
                        [ui.available_width(), editor_h],
                        egui::TextEdit::multiline(buf).font(egui::TextStyle::Monospace),
                    );

                    ui.horizontal(|ui| {
                        if ui.button(statics::EN_BTN_APPLY).clicked() {
                            match TiValue::parse_json5(buf.trim()) {
                                Ok(parsed) => {
                                    *v = parsed;
                                    changed_any = true;
                                    self.last_error = None;
                                }
                                Err(e) => {
                                    self.last_error = Some(format!(
                                        "Invalid JSON5 for nested value '{k}': {e:#}"
                                    ));
                                }
                            }
                        }
                        if ui.button(statics::EN_BTN_RESET).clicked() {
                            *buf = match v {
                                TiValue::Array(_) | TiValue::Object(_) => v.to_ti_save_pretty(),
                                _ => v.to_json5_compact(),
                            };
                        }
                    });
                });
            }
        }

        changed_any
    }

    fn default_save_dir() -> Option<PathBuf> {
        let home = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)?;

        Some(
            home.join("Documents")
                .join("My Games")
                .join("TerraInvicta")
                .join("Saves"),
        )
    }

    fn initial_dialog_dir() -> Option<PathBuf> {
        static CACHED: OnceLock<Option<PathBuf>> = OnceLock::new();
        CACHED.get_or_init(Self::default_save_dir).clone()
    }

    fn file_dialog(&self) -> rfd::FileDialog {
        let mut dlg =
            rfd::FileDialog::new().add_filter("Terra Invicta Save", &["json", "json5", "gz"]);

        if let Some(dir) = self.dialog_dir.clone().or_else(Self::initial_dialog_dir) {
            dlg = dlg.set_directory(dir);
        }

        dlg
    }

    fn open_file(&mut self) {
        let Some(path) = self.file_dialog().pick_file() else {
            return;
        };

        match LoadedSave::load_path(&path) {
            Ok(save) => {
                self.dialog_dir = path.parent().map(PathBuf::from);
                self.status = format!("Loaded {}", path.display());
                self.selected_group = save.index.groups.first().cloned();
                self.selected_object_id = None;
                self.selected_property = None;
                self.edit_buffer.clear();
                self.raw_edit_mode = false;
                self.scroll_groups_to_selected = false;
                self.scroll_objects_to_selected = false;
                self.scroll_properties_to_selected = false;
                self.scroll_align_center = false;
                self.save = Some(save);
                self.last_error = None;

                self.history_back.clear();
                self.history_forward.clear();
                self.go_to_id_open = false;
                self.go_to_id_input.clear();

                self.undo_stack.clear();
                self.redo_stack.clear();
                self.changes_open = false;
            }
            Err(e) => {
                self.last_error = Some(format!("Failed to load: {e:#}"));
            }
        }
    }

    fn save_file(&mut self) {
        // UX: don't overwrite the loaded file by default.
        self.save_file_as();
    }

    /// Open a file picker for the "other" save, diff it against the currently
    /// loaded save, and cache the full (unfiltered) result. The second save's
    /// tree is dropped inside `diff_against_path`, so we never hold two full
    /// saves resident; only the small `SaveDiff` survives.
    fn compare_against(&mut self) {
        let Some(before) = self.save.as_ref() else {
            self.last_error = Some(statics::EN_DIFF_ERR_NO_SAVE.to_string());
            return;
        };
        let Some(path) = self.file_dialog().pick_file() else {
            return;
        };
        match diff::diff_against_path(before, &path, &IgnoreRules::new()) {
            Ok(full) => {
                self.dialog_dir = path.parent().map(PathBuf::from);
                self.diff_other_path = Some(path);
                self.diff_full = Some(full);
                self.diff_hide_noise = true;
                self.diff_show_all_groups = true;
                self.diff_visible_groups.clear();
                self.diff_search.clear();
                self.diff_open = true;
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("Failed to compare: {e:#}"));
            }
        }
    }

    /// Build the active `IgnoreRules` for the diff window from the noise toggle.
    /// Group visibility and search are positive view filters layered on top via
    /// `SaveDiff::view_filtered`, not destructive ignore rules.
    fn diff_ignore_rules(&self) -> IgnoreRules {
        if self.diff_hide_noise {
            IgnoreRules::default_noise_preset()
        } else {
            IgnoreRules::new()
        }
    }

    /// Short display name for a gamestate group (strips the long TI namespace).
    fn diff_group_label(group: &str) -> &str {
        LoadedSave::group_display_name(group)
    }

    /// Render the Compare Saves window. The full diff is cached; this re-derives
    /// the visible view each frame from the cheap pure filters, so changing the
    /// search box or group selection never re-reads the files.
    fn show_diff_window(&mut self, ctx: &egui::Context) {
        let base = self
            .diff_full
            .as_ref()
            .map(|full| full.filtered(&self.diff_ignore_rules()));
        let changed_groups = base
            .as_ref()
            .map(SaveDiff::changed_groups)
            .unwrap_or_default();

        let other_name = self
            .diff_other_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned());
        let this_name = self
            .save
            .as_ref()
            .and_then(|s| s.source_path.as_ref())
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned());

        let mut open = self.diff_open;
        let mut hide_noise = self.diff_hide_noise;
        let mut show_all_groups = self.diff_show_all_groups;
        let mut visible_groups = self.diff_visible_groups.clone();
        let mut search = self.diff_search.clone();

        egui::Window::new(statics::EN_WINDOW_DIFF)
            .collapsible(true)
            .resizable(true)
            .open(&mut open)
            .default_width(980.0)
            .default_height(700.0)
            .show(ctx, |ui| {
                let Some(base) = base.as_ref() else {
                    ui.label(statics::EN_DIFF_NO_OTHER);
                    return;
                };

                // Capture the window's granted body height up front. A
                // horizontal layout hugs its content height, so without forcing
                // the row to claim the full height the nested scroll area only
                // ever gets content-height to fill and the window snaps back to
                // content size (defeating vertical resize). set_min_height makes
                // the row, and therefore the results scroll area, fill the body.
                let body_height = ui.available_height();

                ui.horizontal(|ui| {
                    ui.set_min_width(940.0);
                    ui.set_min_height(body_height);

                    // Left rail: filters and group selection.
                    ui.vertical(|ui| {
                        ui.set_width(260.0);
                        ui.heading(statics::EN_DIFF_IGNORE_HEADER);
                        ui.checkbox(&mut hide_noise, statics::EN_DIFF_IGNORE_NOISE_PRESET);
                        ui.separator();

                        ui.label(statics::EN_DIFF_SEARCH_LABEL);
                        ui.add(
                            egui::TextEdit::singleline(&mut search)
                                .hint_text(statics::EN_DIFF_SEARCH_HINT)
                                .desired_width(240.0),
                        );

                        ui.separator();
                        if ui
                            .checkbox(&mut show_all_groups, statics::EN_DIFF_SHOW_ALL_GROUPS)
                            .changed()
                            && show_all_groups
                        {
                            visible_groups.clear();
                        }
                        ui.horizontal(|ui| {
                            if ui.button(statics::EN_DIFF_BTN_IGNORE_ALL_GROUPS).clicked() {
                                show_all_groups = false;
                                visible_groups.clear();
                            }
                            if ui.button(statics::EN_DIFF_BTN_SHOW_ALL_GROUPS).clicked() {
                                show_all_groups = true;
                                visible_groups.clear();
                            }
                        });

                        if !show_all_groups {
                            ui.label(statics::EN_DIFF_VISIBLE_GROUPS_LABEL);
                            egui::ScrollArea::vertical()
                                .max_height(460.0)
                                .id_salt("diff_visible_groups")
                                .show(ui, |ui| {
                                    for g in &changed_groups {
                                        let mut on = visible_groups.contains(g);
                                        if ui.checkbox(&mut on, Self::diff_group_label(g)).changed()
                                        {
                                            if on {
                                                visible_groups.insert(g.clone());
                                            } else {
                                                visible_groups.remove(g);
                                            }
                                        }
                                    }
                                });
                        }
                    });

                    ui.separator();

                    // Right rail: summary and results.
                    ui.vertical(|ui| {
                        let group_filter = if show_all_groups {
                            None
                        } else {
                            Some(&visible_groups)
                        };
                        let view = base.view_filtered(group_filter, &search);

                        if let Some(name) = &this_name {
                            ui.label(format!("{} {}", statics::EN_DIFF_PREFIX_BEFORE, name));
                        }
                        if let Some(name) = &other_name {
                            ui.label(format!("{} {}", statics::EN_DIFF_PREFIX_AFTER, name));
                        }
                        let (added, removed, changed) = view.summary_counts();
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                egui::Color32::LIGHT_GREEN,
                                format!("{} {}", added, statics::EN_DIFF_LABEL_ADDED),
                            );
                            ui.colored_label(
                                egui::Color32::LIGHT_RED,
                                format!("{} {}", removed, statics::EN_DIFF_LABEL_REMOVED),
                            );
                            ui.colored_label(
                                egui::Color32::YELLOW,
                                format!("{} {}", changed, statics::EN_DIFF_LABEL_CHANGED),
                            );
                        });

                        if !view.groups_added.is_empty() {
                            ui.label(format!(
                                "{} {}",
                                statics::EN_DIFF_GROUPS_ADDED,
                                view.groups_added
                                    .iter()
                                    .map(|g| Self::diff_group_label(g))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                        }
                        if !view.groups_removed.is_empty() {
                            ui.label(format!(
                                "{} {}",
                                statics::EN_DIFF_GROUPS_REMOVED,
                                view.groups_removed
                                    .iter()
                                    .map(|g| Self::diff_group_label(g))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ));
                        }

                        ui.separator();

                        if view.is_empty() {
                            if base.is_empty() {
                                ui.label(statics::EN_DIFF_IDENTICAL);
                            } else {
                                ui.label(statics::EN_DIFF_NO_MATCHES);
                            }
                            return;
                        }

                        egui::ScrollArea::vertical()
                            .id_salt("diff_objects")
                            .auto_shrink([false, false])
                            .max_height(ui.available_height())
                            .show(ui, |ui| {
                                for obj in &view.objects {
                                    let (glyph, color) = Self::diff_kind_visual(obj.kind);
                                    let title = if obj.display_name.is_empty() {
                                        format!(
                                            "{}  {} #{}",
                                            glyph,
                                            Self::diff_group_label(&obj.group),
                                            obj.id
                                        )
                                    } else {
                                        format!(
                                            "{}  {} #{}  ({})",
                                            glyph,
                                            Self::diff_group_label(&obj.group),
                                            obj.id,
                                            obj.display_name
                                        )
                                    };

                                    if obj.field_changes.is_empty() {
                                        ui.label(
                                            egui::RichText::new(title).monospace().color(color),
                                        );
                                    } else {
                                        egui::CollapsingHeader::new(
                                            egui::RichText::new(title).monospace().color(color),
                                        )
                                        .id_salt(("diff_obj", &obj.group, obj.id))
                                        .show(ui, |ui| {
                                            for fc in &obj.field_changes {
                                                let before =
                                                    Self::diff_value_preview(fc.before.as_ref());
                                                let after =
                                                    Self::diff_value_preview(fc.after.as_ref());
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{:<38}  {:<62} ->  {}",
                                                        fc.path, before, after
                                                    ))
                                                    .monospace(),
                                                );
                                            }
                                        });
                                    }
                                }
                            });
                    });
                });
            });

        self.diff_open = open;
        self.diff_hide_noise = hide_noise;
        self.diff_show_all_groups = show_all_groups;
        self.diff_visible_groups = visible_groups;
        self.diff_search = search;
    }

    fn diff_kind_visual(kind: crate::DiffKind) -> (&'static str, egui::Color32) {
        match kind {
            crate::DiffKind::Added => (statics::EN_DIFF_KIND_ADDED, egui::Color32::LIGHT_GREEN),
            crate::DiffKind::Removed => (statics::EN_DIFF_KIND_REMOVED, egui::Color32::LIGHT_RED),
            crate::DiffKind::Changed => (statics::EN_DIFF_KIND_CHANGED, egui::Color32::YELLOW),
        }
    }

    /// Compact one-line preview of a value side in a field change.
    fn diff_value_preview(v: Option<&TiValue>) -> String {
        match v {
            None => statics::EN_LITERAL_MISSING.to_string(),
            Some(val) => {
                let s = val.to_json5_compact();
                const MAX: usize = 60;
                if s.chars().count() > MAX {
                    let truncated: String = s.chars().take(MAX).collect();
                    format!("{truncated}...")
                } else {
                    s
                }
            }
        }
    }

    fn save_file_as(&mut self) {
        let mut dlg = self.file_dialog();
        if let Some(save) = self.save.as_ref()
            && let Some(source_path) = save.source_path.as_ref()
            && let Some(file_name) = source_path.file_name()
        {
            dlg = dlg.set_file_name(file_name.to_string_lossy());
        }

        let Some(path) = dlg.save_file() else {
            return;
        };

        let Some(save) = self.save.as_mut() else {
            return;
        };

        if let Err(e) = save.save_to_path(&path) {
            self.last_error = Some(format!("Failed to save: {e:#}"));
        } else {
            self.dialog_dir = path.parent().map(PathBuf::from);
            self.status = format!("Saved {}", path.display());
            self.last_error = None;
        }
    }

    fn select_object_user(&mut self, group: &str, id: i64) {
        self.select_object_internal(group, id, true, false, false);
    }

    fn select_object_programmatic(
        &mut self,
        group: &str,
        id: i64,
        record_history: bool,
        center_on_target: bool,
    ) {
        self.select_object_internal(group, id, record_history, true, center_on_target);
    }

    fn select_object_internal(
        &mut self,
        group: &str,
        id: i64,
        record_history: bool,
        request_scroll: bool,
        center_on_target: bool,
    ) {
        if record_history
            && let Some(cur) = self.selected_object_id
            && cur != id
        {
            if self.history_back.last().copied() != Some(cur) {
                self.history_back.push(cur);
            }
            self.history_forward.clear();
        }

        self.selected_group = Some(group.to_string());
        self.selected_object_id = Some(id);
        self.selected_property = None;
        self.edit_buffer.clear();
        self.raw_edit_mode = false;

        if request_scroll {
            self.scroll_groups_to_selected = true;
            self.scroll_objects_to_selected = true;
            self.scroll_properties_to_selected = false;
            self.scroll_align_center = center_on_target;
        }
    }

    fn go_back(&mut self) {
        let Some(target) = self.history_back.pop() else {
            return;
        };
        if let Some(cur) = self.selected_object_id {
            self.history_forward.push(cur);
        }
        let group = self
            .save
            .as_ref()
            .and_then(|s| s.index.id_lookup.get(&target))
            .map(|(g, _)| g.clone());
        if let Some(group) = group {
            self.select_object_programmatic(&group, target, false, false);
        } else {
            self.last_error = Some(format!("History target ID {target} not found"));
        }
    }

    fn go_forward(&mut self) {
        let Some(target) = self.history_forward.pop() else {
            return;
        };
        if let Some(cur) = self.selected_object_id {
            self.history_back.push(cur);
        }
        let group = self
            .save
            .as_ref()
            .and_then(|s| s.index.id_lookup.get(&target))
            .map(|(g, _)| g.clone());
        if let Some(group) = group {
            self.select_object_programmatic(&group, target, false, false);
        } else {
            self.last_error = Some(format!("History target ID {target} not found"));
        }
    }

    fn apply_property_edit(&mut self, save: &mut LoadedSave) {
        let Some(group) = self.selected_group.clone() else {
            return;
        };
        let Some(object_id) = self.selected_object_id else {
            return;
        };
        let Some(prop) = self.selected_property.clone() else {
            return;
        };

        let parsed = match TiValue::parse_json5(self.edit_buffer.trim()) {
            Ok(v) => v,
            Err(e) => {
                self.last_error = Some(format!("Invalid JSON5 for property: {e:#}"));
                return;
            }
        };

        let before = save
            .get_object_value(&group, object_id)
            .and_then(|o| o.get(&prop))
            .cloned();

        {
            let Some(value_obj) = save.get_object_value_mut(&group, object_id) else {
                self.last_error = Some(statics::EN_ERR_LOCATE_SELECTED_OBJECT.to_string());
                return;
            };
            value_obj.insert(prop.clone(), parsed.clone());
        }

        save.rebuild_index();
        save.refresh_dirty();

        let is_rel_ref = parsed.is_relational_ref().is_some();
        let structured = matches!(parsed, TiValue::Array(_) | TiValue::Object(_)) && !is_rel_ref;
        let next_buffer = if structured {
            parsed.to_ti_save_pretty()
        } else {
            parsed.to_json5_compact()
        };

        let desc = format!(
            "{} {}: {}",
            statics::EN_SORT_ID,
            object_id,
            Self::describe_change(&prop, before.as_ref(), Some(&parsed))
        );
        self.record_action(EditAction {
            group: group.clone(),
            object_id,
            prop: prop.clone(),
            before,
            after: Some(parsed),
            description: desc.clone(),
        });
        self.status = desc;
        self.last_error = None;

        // Keep the edit buffer in a pleasant display format after applying.
        self.edit_buffer = next_buffer;
        self.raw_edit_mode = structured;

        // Keep the public-opinion helper in sync after applying.
        if prop == statics::TI_PROP_PUBLIC_OPINION
            && let Some(obj) = save.get_object_value(&group, object_id)
        {
            self.refresh_public_opinion_editor(obj, &prop);
        }
    }

    fn set_property_null(&mut self, save: &mut LoadedSave) {
        let Some(group) = self.selected_group.clone() else {
            return;
        };
        let Some(object_id) = self.selected_object_id else {
            return;
        };
        let Some(prop) = self.selected_property.clone() else {
            return;
        };

        let before = save
            .get_object_value(&group, object_id)
            .and_then(|o| o.get(&prop))
            .cloned();

        {
            let Some(value_obj) = save.get_object_value_mut(&group, object_id) else {
                self.last_error = Some(statics::EN_ERR_LOCATE_SELECTED_OBJECT.to_string());
                return;
            };
            value_obj.insert(prop.clone(), TiValue::Null);
        }

        save.rebuild_index();
        save.refresh_dirty();

        let desc = format!(
            "{} {}: {}",
            statics::EN_SORT_ID,
            object_id,
            Self::describe_change(&prop, before.as_ref(), Some(&TiValue::Null))
        );
        self.record_action(EditAction {
            group: group.clone(),
            object_id,
            prop: prop.clone(),
            before,
            after: Some(TiValue::Null),
            description: desc.clone(),
        });
        self.status = desc;
        self.last_error = None;
        self.edit_buffer = statics::EN_LITERAL_NULL.to_string();

        if prop == statics::TI_PROP_PUBLIC_OPINION
            && let Some(obj) = save.get_object_value(&group, object_id)
        {
            self.refresh_public_opinion_editor(obj, &prop);
        }
    }

    /// Sets `factionOrbit` on every checked TIOrgState object to reference `faction_id`,
    /// and keeps `TIFactionState.availableOrgs` in sync (added to the new faction, removed
    /// from any previous faction). Records one undoable EditAction per changed property.
    fn add_selected_orgs_to_faction(&mut self, save: &mut LoadedSave, faction_id: i64) {
        let org_ids: Vec<i64> = self.org_bulk_selected_ids.iter().copied().collect();
        if org_ids.is_empty() {
            return;
        }

        let mut updated = 0usize;
        for object_id in org_ids {
            let before = save
                .get_object_value(statics::TI_GROUP_ORG_STATE, object_id)
                .and_then(|o| o.get(statics::TI_PROP_FACTION_ORBIT))
                .cloned();
            let previous_faction_id = before.as_ref().and_then(TiValue::is_relational_ref);

            let mut ref_map = indexmap::IndexMap::new();
            ref_map.insert(
                statics::TI_REF_FIELD_VALUE.to_string(),
                TiValue::Number(crate::value::TiNumber::I64(faction_id)),
            );
            let after = TiValue::Object(ref_map);

            let Some(obj) = save.get_object_value_mut(statics::TI_GROUP_ORG_STATE, object_id)
            else {
                continue;
            };
            obj.insert(statics::TI_PROP_FACTION_ORBIT.to_string(), after.clone());
            updated += 1;

            let desc = format!(
                "{} {}: {}",
                statics::EN_SORT_ID,
                object_id,
                Self::describe_change(
                    statics::TI_PROP_FACTION_ORBIT,
                    before.as_ref(),
                    Some(&after)
                )
            );
            self.record_action(EditAction {
                group: statics::TI_GROUP_ORG_STATE.to_string(),
                object_id,
                prop: statics::TI_PROP_FACTION_ORBIT.to_string(),
                before,
                after: Some(after),
                description: desc,
            });

            if let Some(prev_faction_id) = previous_faction_id
                && prev_faction_id != faction_id
            {
                self.remove_org_from_faction_available_orgs(save, prev_faction_id, object_id);
            }
            self.add_org_to_faction_available_orgs(save, faction_id, object_id);
        }

        save.rebuild_index();
        save.refresh_dirty();

        self.status = format!("Added {updated} organization(s) to faction {faction_id}");
        self.last_error = None;
    }

    /// Appends `org_id` as a relational ref to `TIFactionState.availableOrgs` for `faction_id`,
    /// unless it's already present. Records an undoable EditAction.
    fn add_org_to_faction_available_orgs(
        &mut self,
        save: &mut LoadedSave,
        faction_id: i64,
        org_id: i64,
    ) {
        let Some(faction_obj) = save.get_object_value(statics::TI_GROUP_FACTION_STATE, faction_id)
        else {
            return;
        };
        let before = faction_obj.get(statics::TI_PROP_AVAILABLE_ORGS).cloned();
        let mut items: Vec<TiValue> = match &before {
            Some(TiValue::Array(items)) => items.clone(),
            _ => Vec::new(),
        };
        if items.iter().any(|v| v.is_relational_ref() == Some(org_id)) {
            return;
        }

        let mut ref_map = indexmap::IndexMap::new();
        ref_map.insert(
            statics::TI_REF_FIELD_VALUE.to_string(),
            TiValue::Number(crate::value::TiNumber::I64(org_id)),
        );
        items.push(TiValue::Object(ref_map));
        let after = TiValue::Array(items);

        let Some(obj) = save.get_object_value_mut(statics::TI_GROUP_FACTION_STATE, faction_id)
        else {
            return;
        };
        obj.insert(statics::TI_PROP_AVAILABLE_ORGS.to_string(), after.clone());

        let desc = format!(
            "{} {}: {}",
            statics::EN_SORT_ID,
            faction_id,
            Self::describe_change(
                statics::TI_PROP_AVAILABLE_ORGS,
                before.as_ref(),
                Some(&after)
            )
        );
        self.record_action(EditAction {
            group: statics::TI_GROUP_FACTION_STATE.to_string(),
            object_id: faction_id,
            prop: statics::TI_PROP_AVAILABLE_ORGS.to_string(),
            before,
            after: Some(after),
            description: desc,
        });
    }

    /// Removes `org_id`'s relational ref from `TIFactionState.availableOrgs` for `faction_id`,
    /// if present. Records an undoable EditAction.
    fn remove_org_from_faction_available_orgs(
        &mut self,
        save: &mut LoadedSave,
        faction_id: i64,
        org_id: i64,
    ) {
        let Some(faction_obj) = save.get_object_value(statics::TI_GROUP_FACTION_STATE, faction_id)
        else {
            return;
        };
        let Some(before) = faction_obj.get(statics::TI_PROP_AVAILABLE_ORGS).cloned() else {
            return;
        };
        let Some(items) = before.as_array() else {
            return;
        };
        if !items.iter().any(|v| v.is_relational_ref() == Some(org_id)) {
            return;
        }

        let after = TiValue::Array(
            items
                .iter()
                .filter(|v| v.is_relational_ref() != Some(org_id))
                .cloned()
                .collect(),
        );

        let Some(obj) = save.get_object_value_mut(statics::TI_GROUP_FACTION_STATE, faction_id)
        else {
            return;
        };
        obj.insert(statics::TI_PROP_AVAILABLE_ORGS.to_string(), after.clone());

        let desc = format!(
            "{} {}: {}",
            statics::EN_SORT_ID,
            faction_id,
            Self::describe_change(statics::TI_PROP_AVAILABLE_ORGS, Some(&before), Some(&after))
        );
        self.record_action(EditAction {
            group: statics::TI_GROUP_FACTION_STATE.to_string(),
            object_id: faction_id,
            prop: statics::TI_PROP_AVAILABLE_ORGS.to_string(),
            before: Some(before),
            after: Some(after),
            description: desc,
        });
    }

    fn refresh_public_opinion_editor(
        &mut self,
        object_value: &indexmap::IndexMap<String, TiValue>,
        prop: &str,
    ) {
        self.public_opinion_inputs.clear();
        self.public_opinion_remainder = None;
        self.public_opinion_drag = None;

        if prop != statics::TI_PROP_PUBLIC_OPINION {
            return;
        }

        let Some(val) = object_value.get(prop) else {
            return;
        };
        let Some(map) = val.as_object() else {
            return;
        };

        // Populate inputs for all factions except the special variable.
        let mut sum = 0.0;
        for (k, v) in map.iter() {
            if k == statics::TI_PUBLIC_OPINION_UNDECIDED {
                continue;
            }
            let num = match v {
                TiValue::Number(n) => match n {
                    crate::value::TiNumber::I64(x) => *x as f64,
                    crate::value::TiNumber::U64(x) => *x as f64,
                    crate::value::TiNumber::F64(x) => *x,
                },
                _ => continue,
            };
            sum += num;
            self.public_opinion_inputs
                .push((k.clone(), num.to_string()));
        }
        self.public_opinion_remainder = Some(1.0 - sum);
    }

    fn render_faction_filter_combobox(
        ui: &mut egui::Ui,
        factions: &[&crate::save::ObjectSummary],
        filter: &mut Option<i64>,
        id_to_display_name: &std::collections::HashMap<i64, String>,
        id_salt: &str,
    ) {
        ui.horizontal(|ui| {
            ui.label(statics::EN_LABEL_FILTER_FACTION);
            let selected_label = filter
                .and_then(|id| id_to_display_name.get(&id))
                .map(String::as_str)
                .unwrap_or(statics::EN_FILTER_ALL_FACTIONS);
            egui::ComboBox::from_id_salt(id_salt)
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(filter, None, statics::EN_FILTER_ALL_FACTIONS);
                    for faction in factions {
                        ui.selectable_value(filter, Some(faction.id), &faction.display_name);
                    }
                });
        });
    }

    fn render_org_faction_filter_combobox(
        ui: &mut egui::Ui,
        factions: &[&crate::save::ObjectSummary],
        filter: &mut OrgFactionFilter,
        id_to_display_name: &std::collections::HashMap<i64, String>,
        id_salt: &str,
    ) {
        ui.horizontal(|ui| {
            ui.label(statics::EN_LABEL_FILTER_FACTION);
            let selected_label = match filter {
                OrgFactionFilter::All => statics::EN_FILTER_ALL_FACTIONS.to_string(),
                OrgFactionFilter::NoFaction => statics::EN_FILTER_NO_FACTION.to_string(),
                OrgFactionFilter::Faction(id) => id_to_display_name
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| statics::EN_FILTER_ALL_FACTIONS.to_string()),
            };
            egui::ComboBox::from_id_salt(id_salt)
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        filter,
                        OrgFactionFilter::All,
                        statics::EN_FILTER_ALL_FACTIONS,
                    );
                    ui.selectable_value(
                        filter,
                        OrgFactionFilter::NoFaction,
                        statics::EN_FILTER_NO_FACTION,
                    );
                    for faction in factions {
                        ui.selectable_value(
                            filter,
                            OrgFactionFilter::Faction(faction.id),
                            &faction.display_name,
                        );
                    }
                });
        });
    }

    fn render_tier_filter_combobox(
        ui: &mut egui::Ui,
        filter: &mut Option<i64>,
        tiers: &[i64],
        id_salt: &str,
    ) {
        ui.horizontal(|ui| {
            ui.label(statics::EN_LABEL_FILTER_TIER);
            let selected_label = filter
                .map(|t| t.to_string())
                .unwrap_or_else(|| statics::EN_FILTER_ALL_TIERS.to_string());
            egui::ComboBox::from_id_salt(id_salt)
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(filter, None, statics::EN_FILTER_ALL_TIERS);
                    for &tier in tiers {
                        ui.selectable_value(filter, Some(tier), tier.to_string());
                    }
                });
        });
    }

    fn render_properties_panel(
        &mut self,
        ui: &mut egui::Ui,
        properties: &[(&String, &TiValue)],
        value_obj: &indexmap::IndexMap<String, TiValue>,
        id_lookup: &std::collections::HashMap<i64, (String, usize)>,
        id_to_display_name: &std::collections::HashMap<i64, String>,
    ) {
        ui.heading(statics::EN_HEADING_PROPERTIES);
        ui.separator();

        // Make the table fill the available width so sizing is stable.
        ui.set_width(ui.available_width());

        let scroll_h = ui.available_height();
        ui.push_id("properties_panel", |ui| {
            egui::ScrollArea::vertical()
                .max_height(scroll_h)
                .show(ui, |ui| {
                    let row_h = ui.text_style_height(&egui::TextStyle::Body) + 6.0;

                    TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::initial(240.0).resizable(true))
                        .column(Column::remainder().resizable(true))
                        .column(Column::initial(80.0).resizable(false))
                        .header(row_h, |mut header| {
                            header.col(|ui| {
                                ui.strong(statics::EN_COL_PROPERTY);
                            });
                            header.col(|ui| {
                                ui.strong(statics::EN_COL_VALUE_REF);
                            });
                            header.col(|ui| {
                                ui.strong(statics::EN_COL_TYPE);
                            });
                        })
                        .body(|mut body| {
                            for (key, val) in properties.iter() {
                                body.row(row_h, |mut row| {
                                    let selected =
                                        self.selected_property.as_deref() == Some(key.as_str());

                                    row.col(|ui| {
                                        let resp = ui.selectable_label(selected, key.as_str());
                                        if selected && self.scroll_properties_to_selected {
                                            let align = if self.scroll_align_center {
                                                egui::Align::Center
                                            } else {
                                                egui::Align::Min
                                            };
                                            resp.scroll_to_me(Some(align));
                                            self.scroll_properties_to_selected = false;
                                            self.scroll_align_center = false;
                                        }
                                        if resp.clicked() {
                                            self.selected_property = Some((*key).to_string());
                                            self.last_error = None;

                                            self.raw_edit_mode =
                                                matches!(
                                                    val,
                                                    TiValue::Array(_) | TiValue::Object(_)
                                                ) && val.is_relational_ref().is_none();

                                            self.edit_buffer = if val.is_relational_ref().is_some()
                                            {
                                                val.to_json5_compact()
                                            } else if self.raw_edit_mode {
                                                val.to_ti_save_pretty()
                                            } else {
                                                val.to_json5_compact()
                                            };

                                            self.refresh_public_opinion_editor(value_obj, key);
                                        }
                                    });

                                    row.col(|ui| {
                                        if let Some(target_id) = val.is_relational_ref() {
                                            let name = id_to_display_name
                                                .get(&target_id)
                                                .map(String::as_str)
                                                .unwrap_or(statics::EN_EMPTY);
                                            ui.horizontal(|ui| {
                                                if ui.small_button(statics::EN_BTN_GO).clicked() {
                                                    if let Some((ref_group, _)) =
                                                        id_lookup.get(&target_id)
                                                    {
                                                        self.select_object_programmatic(
                                                            ref_group, target_id, true, true,
                                                        );
                                                    } else {
                                                        self.last_error = Some(format!(
                                                            "Reference ID {target_id} not found"
                                                        ));
                                                    }
                                                }
                                                if name.is_empty() {
                                                    ui.label(format!("{target_id}"));
                                                } else {
                                                    ui.label(format!("{target_id}: {name}"));
                                                }
                                            });
                                        } else if let Some(ids) = array_of_relational_refs(val) {
                                            ui.label(format!("{} refs", ids.len()));
                                        } else {
                                            ui.label(value_preview(val));
                                        }
                                    });

                                    row.col(|ui| {
                                        ui.monospace(val.type_name());
                                    });
                                });
                            }
                        });
                });
        });
    }

    fn render_editor_panel(
        &mut self,
        ui: &mut egui::Ui,
        value_obj: &indexmap::IndexMap<String, TiValue>,
        save: &mut LoadedSave,
    ) {
        ui.heading(statics::EN_HEADING_EDIT);
        ui.separator();

        let scroll_h = ui.available_height();
        ui.push_id("editor_panel", |ui| {
            egui::ScrollArea::vertical()
                .max_height(scroll_h)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let selected_property = self.selected_property.clone();
                    if let Some(prop) = selected_property.as_deref() {
                        ui.label(format!("Editing: {prop}"));

                        let current_val = value_obj.get(prop);
                        let is_rel_ref = current_val
                            .map(|v| v.is_relational_ref().is_some())
                            .unwrap_or(false);
                        let is_structured =
                            matches!(current_val, Some(TiValue::Array(_) | TiValue::Object(_)))
                                && !is_rel_ref;

                        let show_public_opinion_helper = prop == statics::TI_PROP_PUBLIC_OPINION
                            && current_val.and_then(TiValue::as_object).is_some()
                            && !self.public_opinion_inputs.is_empty();

                        // Avoid duplicate "apply" paths for the Public Opinion helper.
                        // For other properties, keep the standard action row visible.
                        if !show_public_opinion_helper {
                            // Always keep actions visible; putting this row after the large multiline editor
                            // can push it off-screen on smaller windows.
                            ui.horizontal(|ui| {
                                if ui.button(statics::EN_BTN_APPLY_PROPERTY).clicked() {
                                    self.apply_property_edit(save);
                                }

                                if ui.button(statics::EN_BTN_SET_NULL).clicked() {
                                    self.set_property_null(save);
                                }

                                if ui.button(statics::EN_BTN_CHANGE_TYPE).clicked() {
                                    self.change_type_open = true;
                                    self.change_type_preview = None;
                                }

                                if let Some(val) = current_val
                                    && let Some(target_id) = val.is_relational_ref()
                                    && ui.button(statics::EN_BTN_GO_TO_REF).clicked()
                                {
                                    if let Some((ref_group, _)) =
                                        save.index.id_lookup.get(&target_id)
                                    {
                                        self.select_object_programmatic(
                                            ref_group, target_id, true, true,
                                        );
                                    } else {
                                        self.last_error =
                                            Some(format!("Reference ID {target_id} not found"));
                                    }
                                }
                            });
                            ui.separator();
                        }

                        if self.change_type_open {
                            let mut open = self.change_type_open;
                            let mut close_requested = false;

                            // Determine the best-available current value to convert:
                            // prefer the staged edit buffer if it parses; otherwise fall back to the actual value.
                            let source_value = TiValue::parse_json5(self.edit_buffer.trim())
                                .ok()
                                .or_else(|| current_val.cloned())
                                .unwrap_or(TiValue::Null);

                            egui::Window::new(statics::EN_WINDOW_CHANGE_TYPE)
                                .collapsible(false)
                                .open(&mut open)
                                .show(ui.ctx(), |ui| {
                                    ui.label(statics::EN_LABEL_PICK_TYPE);
                                    ui.separator();

                                    let type_labels: [&str; 9] = [
                                        statics::EN_TYPE_NULL,
                                        statics::EN_TYPE_BOOL,
                                        statics::EN_TYPE_I64,
                                        statics::EN_TYPE_U64,
                                        statics::EN_TYPE_F64,
                                        statics::EN_TYPE_STRING,
                                        statics::EN_TYPE_ARRAY,
                                        statics::EN_TYPE_OBJECT,
                                        statics::EN_TYPE_REFERENCE,
                                    ];

                                    egui::Grid::new("change_type_grid")
                                        .num_columns(3)
                                        .spacing([10.0, 6.0])
                                        .show(ui, |ui| {
                                            for (i, label) in type_labels.iter().enumerate() {
                                                if ui.button(*label).clicked() {
                                                    self.change_type_preview =
                                                        Some(Self::coerce_value_to_type(
                                                            label,
                                                            &source_value,
                                                        ));
                                                    self.last_error = None;
                                                }

                                                if (i + 1) % 3 == 0 {
                                                    ui.end_row();
                                                }
                                            }
                                        });

                                    ui.separator();
                                    ui.label(statics::EN_LABEL_PREVIEW);

                                    let preview_text = self
                                        .change_type_preview
                                        .as_ref()
                                        .map(Self::value_for_editing)
                                        .unwrap_or_default();

                                    let mut preview_buf = preview_text.clone();
                                    ui.add_enabled(
                                        false,
                                        egui::TextEdit::multiline(&mut preview_buf)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_rows(6),
                                    );

                                    ui.separator();
                                    ui.horizontal(|ui| {
                                        let can_apply = self.change_type_preview.is_some();
                                        if ui
                                            .add_enabled(
                                                can_apply,
                                                egui::Button::new(statics::EN_BTN_APPLY),
                                            )
                                            .clicked()
                                            && let Some(v) = self.change_type_preview.clone()
                                        {
                                            self.edit_buffer = Self::value_for_editing(&v);
                                            self.raw_edit_mode =
                                                matches!(v, TiValue::Array(_) | TiValue::Object(_))
                                                    && v.is_relational_ref().is_none();
                                            self.apply_property_edit(save);
                                            close_requested = true;
                                        }

                                        if ui.button(statics::EN_BTN_CANCEL).clicked() {
                                            close_requested = true;
                                        }
                                    });
                                });

                            if close_requested {
                                open = false;
                            }

                            self.change_type_open = open;
                            if !self.change_type_open {
                                self.change_type_preview = None;
                            }
                        }

                        if is_structured {
                            self.raw_edit_mode = true;
                        }

                        if is_rel_ref {
                            let fallback_id =
                                current_val.and_then(|v| v.is_relational_ref()).unwrap_or(0);

                            let mut target_id = fallback_id;
                            let mut type_hint: Option<String> = None;

                            if let Ok(TiValue::Object(map)) =
                                TiValue::parse_json5(self.edit_buffer.trim())
                            {
                                if let Some(TiValue::String(t)) =
                                    map.get(statics::TI_REF_FIELD_TYPE)
                                {
                                    type_hint = Some(t.clone());
                                }
                                if let Some(TiValue::Number(n)) =
                                    map.get(statics::TI_REF_FIELD_VALUE)
                                    && let Some(id) = n.as_i64()
                                {
                                    target_id = id;
                                }
                            }

                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(statics::EN_LABEL_REFERENCE_ID);
                                    let resp =
                                        ui.add(egui::DragValue::new(&mut target_id).speed(1.0));
                                    if resp.changed() {
                                        let mut map = indexmap::IndexMap::new();
                                        if let Some(t) = type_hint.clone() {
                                            map.insert(
                                                statics::TI_REF_FIELD_TYPE.to_string(),
                                                TiValue::String(t),
                                            );
                                        }
                                        map.insert(
                                            statics::TI_REF_FIELD_VALUE.to_string(),
                                            TiValue::Number(crate::value::TiNumber::I64(target_id)),
                                        );
                                        self.edit_buffer = TiValue::Object(map).to_json5_compact();
                                        self.last_error = None;
                                    }

                                    if ui.small_button(statics::EN_BTN_GO).clicked() {
                                        if let Some((ref_group, _)) =
                                            save.index.id_lookup.get(&target_id)
                                        {
                                            self.select_object_programmatic(
                                                ref_group, target_id, true, true,
                                            );
                                        } else {
                                            self.last_error =
                                                Some(format!("Reference ID {target_id} not found"));
                                        }
                                    }
                                });

                                ui.checkbox(
                                    &mut self.raw_edit_mode,
                                    statics::EN_CHECKBOX_RAW_JSON5,
                                );
                                if self.raw_edit_mode {
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.edit_buffer)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(ui.available_width()),
                                    );
                                } else {
                                    let mut preview = self.edit_buffer.clone();
                                    ui.add_enabled(
                                        false,
                                        egui::TextEdit::singleline(&mut preview)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(ui.available_width()),
                                    );
                                }
                            });
                            ui.separator();
                        }

                        if let Some(current_val) = current_val {
                            match current_val {
                                TiValue::Bool(b) => {
                                    let staged = match TiValue::parse_json5(self.edit_buffer.trim())
                                    {
                                        Ok(TiValue::Bool(v)) => v,
                                        _ => *b,
                                    };
                                    let mut v = staged;
                                    let resp = ui.add_enabled(
                                        !self.raw_edit_mode,
                                        egui::Checkbox::new(&mut v, statics::EN_LABEL_VALUE),
                                    );
                                    if resp.changed() {
                                        self.edit_buffer =
                                            if v { "true" } else { "false" }.to_string();
                                    }
                                    let mut preview = v.to_string();
                                    ui.add_enabled(false, egui::TextEdit::singleline(&mut preview));
                                    ui.separator();
                                }
                                TiValue::Number(n) => {
                                    use crate::value::TiNumber;

                                    let staged = match TiValue::parse_json5(self.edit_buffer.trim())
                                    {
                                        Ok(TiValue::Number(v)) => Some(v),
                                        _ => None,
                                    };

                                    match n {
                                        TiNumber::I64(orig) => {
                                            let mut v = match &staged {
                                                Some(TiNumber::I64(x)) => *x,
                                                _ => *orig,
                                            };
                                            let resp = ui.add_enabled(
                                                !self.raw_edit_mode,
                                                egui::DragValue::new(&mut v)
                                                    .speed(1)
                                                    .prefix(statics::EN_PREFIX_VALUE),
                                            );
                                            if resp.changed() {
                                                self.edit_buffer =
                                                    TiValue::Number(TiNumber::I64(v))
                                                        .to_json5_compact();
                                            }
                                            let mut preview = TiValue::Number(TiNumber::I64(v))
                                                .to_json5_compact();
                                            ui.add_enabled(
                                                false,
                                                egui::TextEdit::singleline(&mut preview),
                                            );
                                        }
                                        TiNumber::U64(orig) => {
                                            let mut v = match &staged {
                                                Some(TiNumber::U64(x)) => *x,
                                                _ => *orig,
                                            };
                                            let resp = ui.add_enabled(
                                                !self.raw_edit_mode,
                                                egui::DragValue::new(&mut v)
                                                    .speed(1)
                                                    .prefix(statics::EN_PREFIX_VALUE),
                                            );
                                            if resp.changed() {
                                                self.edit_buffer =
                                                    TiValue::Number(TiNumber::U64(v))
                                                        .to_json5_compact();
                                            }
                                            let mut preview = TiValue::Number(TiNumber::U64(v))
                                                .to_json5_compact();
                                            ui.add_enabled(
                                                false,
                                                egui::TextEdit::singleline(&mut preview),
                                            );
                                        }
                                        TiNumber::F64(orig) => {
                                            let mut v = match &staged {
                                                Some(TiNumber::F64(x)) => *x,
                                                _ => *orig,
                                            };
                                            let resp = ui.add_enabled(
                                                !self.raw_edit_mode,
                                                egui::DragValue::new(&mut v)
                                                    .speed(0.1)
                                                    .range(f64::NEG_INFINITY..=f64::INFINITY)
                                                    .prefix(statics::EN_PREFIX_VALUE),
                                            );
                                            if resp.changed() {
                                                self.edit_buffer =
                                                    TiValue::Number(TiNumber::F64(v))
                                                        .to_json5_compact();
                                            }
                                            let mut preview = TiValue::Number(TiNumber::F64(v))
                                                .to_json5_compact();
                                            ui.add_enabled(
                                                false,
                                                egui::TextEdit::singleline(&mut preview),
                                            );
                                        }
                                    }

                                    ui.separator();
                                }
                                TiValue::String(s) => {
                                    let staged = match TiValue::parse_json5(self.edit_buffer.trim())
                                    {
                                        Ok(TiValue::String(v)) => v,
                                        _ => s.clone(),
                                    };
                                    let mut v = staged;
                                    let resp = ui.add_enabled(
                                        !self.raw_edit_mode,
                                        egui::TextEdit::singleline(&mut v)
                                            .hint_text(statics::EN_HINT_VALUE),
                                    );
                                    if resp.changed() {
                                        self.edit_buffer = format!(
                                            "\"{}\"",
                                            v.replace('\\', "\\\\").replace('"', "\\\"")
                                        );
                                    }
                                    let mut preview = v;
                                    ui.add_enabled(false, egui::TextEdit::singleline(&mut preview));
                                    ui.separator();
                                }
                                _ => {}
                            }
                        }

                        if prop == statics::TI_PROP_PUBLIC_OPINION
                            && !self.public_opinion_inputs.is_empty()
                        {
                            ui.group(|ui| {
                                ui.label(statics::EN_PUBLIC_OPINION_HELPER);

                                let mut keys: Vec<String> =
                                    Vec::with_capacity(self.public_opinion_inputs.len());
                                ui.columns(2, |cols| {
                                    // Left: input grid
                                    egui::Grid::new("public_opinion_grid")
                                        .num_columns(2)
                                        .striped(true)
                                        .show(&mut cols[0], |ui| {
                                            for (k, v) in self.public_opinion_inputs.iter_mut() {
                                                keys.push(k.clone());
                                                ui.label(k.as_str());
                                                ui.add(
                                                    egui::TextEdit::singleline(v)
                                                        .desired_width(140.0),
                                                );
                                                ui.end_row();
                                            }
                                        });

                                    // Parse after editing (so both the totals and the chart reflect the current text).
                                    let mut values: Vec<f64> =
                                        Vec::with_capacity(self.public_opinion_inputs.len());
                                    let mut sum = 0.0f64;
                                    let mut parse_error: Option<String> = None;
                                    for (k, s) in self.public_opinion_inputs.iter() {
                                        if parse_error.is_some() {
                                            break;
                                        }
                                        match s.trim().parse::<f64>() {
                                            Ok(x) if x.is_finite() && x >= 0.0 => {
                                                sum += x;
                                                values.push(x);
                                            }
                                            _ => {
                                                parse_error =
                                                    Some(format!("Invalid float for '{k}'"));
                                            }
                                        }
                                    }
                                    while values.len() < self.public_opinion_inputs.len() {
                                        values.push(0.0);
                                    }
                                    // Tolerate tiny floating/rounding error, especially when
                                    // dragging Undecided down to ~0.
                                    let eps = 1e-6;
                                    let mut remainder = 1.0 - sum;
                                    let total_exceeds = remainder < -eps;
                                    if !total_exceeds && remainder.abs() < eps {
                                        sum = 1.0;
                                        remainder = 0.0;
                                    }

                                    cols[0].separator();
                                    cols[0].horizontal(|ui| {
                                        ui.label(format!(
                                            "Total: {}",
                                            Self::format_public_opinion_value(sum)
                                        ));
                                        ui.separator();
                                        ui.label(format!(
                                            "Undecided: {}",
                                            Self::format_public_opinion_value(remainder)
                                        ));
                                    });
                                    if let Some(err) = parse_error.clone() {
                                        cols[0].colored_label(egui::Color32::RED, err);
                                    } else if total_exceeds {
                                        cols[0].colored_label(
                                            egui::Color32::RED,
                                            statics::EN_PUBLIC_OPINION_ERR_TOTAL_EXCEEDS,
                                        );
                                    }

                                    // Right: pie chart + drag editor.
                                    let enabled = parse_error.is_none() && !total_exceeds;
                                    let changed = self.render_public_opinion_pie(
                                        &mut cols[1],
                                        &keys,
                                        &mut values,
                                        &mut remainder,
                                        enabled,
                                    );
                                    if changed {
                                        for (i, (_k, s)) in
                                            self.public_opinion_inputs.iter_mut().enumerate()
                                        {
                                            s.clear();
                                            s.push_str(&Self::format_public_opinion_value(
                                                values[i],
                                            ));
                                        }
                                        self.public_opinion_remainder = Some(remainder);
                                    } else {
                                        self.public_opinion_remainder = Some(remainder);
                                    }

                                    // When valid, keep `edit_buffer` in sync so the standard Apply Property flow works.
                                    if enabled {
                                        let mut by_key: std::collections::HashMap<String, f64> =
                                            std::collections::HashMap::new();
                                        for (k, v) in keys.iter().zip(values.iter()) {
                                            by_key.insert(k.clone(), *v);
                                        }

                                        if let Some(existing_map) =
                                            current_val.and_then(TiValue::as_object)
                                        {
                                            let mut new_map = indexmap::IndexMap::new();
                                            for (k, v) in existing_map.iter() {
                                                if k == statics::TI_PUBLIC_OPINION_UNDECIDED {
                                                    continue;
                                                }
                                                if let Some(x) = by_key.get(k) {
                                                    new_map.insert(
                                                        k.clone(),
                                                        TiValue::Number(
                                                            crate::value::TiNumber::F64(*x),
                                                        ),
                                                    );
                                                } else {
                                                    new_map.insert(k.clone(), v.clone());
                                                }
                                            }
                                            for (k, x) in by_key.into_iter() {
                                                if !new_map.contains_key(&k) {
                                                    new_map.insert(
                                                        k,
                                                        TiValue::Number(
                                                            crate::value::TiNumber::F64(x),
                                                        ),
                                                    );
                                                }
                                            }
                                            new_map.insert(
                                                statics::TI_PUBLIC_OPINION_UNDECIDED.to_string(),
                                                TiValue::Number(crate::value::TiNumber::F64(
                                                    remainder,
                                                )),
                                            );

                                            self.edit_buffer =
                                                TiValue::Object(new_map).to_ti_save_pretty();
                                            self.raw_edit_mode = true;
                                            self.last_error = None;
                                        }
                                    }
                                });

                                ui.separator();
                                ui.horizontal(|ui| {
                                    if ui.button(statics::EN_BTN_APPLY_PROPERTY).clicked() {
                                        self.apply_property_edit(save);
                                    }
                                    if ui.button(statics::EN_BTN_SET_NULL).clicked() {
                                        self.set_property_null(save);
                                    }
                                });

                                // Optional raw view (collapsed by default) to avoid duplicate UI.
                                ui.collapsing(statics::EN_LABEL_JSON5, |ui| {
                                    let editor = egui::TextEdit::multiline(&mut self.edit_buffer)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_rows(8)
                                        .lock_focus(true)
                                        .interactive(true);
                                    let editor_h = 180.0;
                                    let resp =
                                        ui.add_sized([ui.available_width(), editor_h], editor);
                                    if resp.lost_focus()
                                        && let Ok(v) = TiValue::parse_json5(self.edit_buffer.trim())
                                    {
                                        self.edit_buffer = v.to_ti_save_pretty();
                                        self.last_error = None;
                                    }
                                });
                            });
                            ui.separator();
                        }

                        if is_structured && prop != statics::TI_PROP_PUBLIC_OPINION {
                            // Attempt to show structured values in a more readable way.
                            if let Ok(mut staged) = TiValue::parse_json5(self.edit_buffer.trim()) {
                                if let Some(ids) = array_of_relational_refs(&staged) {
                                    ui.group(|ui| {
                                        ui.label(format!("References ({})", ids.len()));
                                        self.render_ref_list_table(
                                            ui,
                                            &ids,
                                            &save.index.id_lookup,
                                            &save.index.id_to_display_name,
                                        );
                                    });
                                    ui.separator();
                                } else if let Some(rows) = array_of_key_value_refs(&staged) {
                                    ui.group(|ui| {
                                        ui.label(format!("Entries ({})", rows.len()));
                                        self.render_key_value_ref_table(
                                            ui,
                                            &rows,
                                            &save.index.id_lookup,
                                            &save.index.id_to_display_name,
                                        );
                                    });
                                    ui.separator();
                                }

                                let trait_helper = prop == statics::TI_PROP_TRAIT_TEMPLATE_NAMES
                                    && matches!(&staged, TiValue::Array(a)
                                        if a.iter().all(|v| matches!(v, TiValue::String(_))));
                                if trait_helper && let TiValue::Array(arr) = &mut staged {
                                    let game_data = self.game_data_snapshot();
                                    let catalog = game_data
                                        .as_deref()
                                        .map(|g| g.names(crate::gamedata::TEMPLATE_TRAITS))
                                        .filter(|c| !c.is_empty());
                                    let mut changed = false;
                                    ui.group(|ui| {
                                        ui.label(statics::EN_TRAIT_EDITOR);
                                        changed = Self::render_trait_editor(
                                            ui,
                                            arr,
                                            catalog,
                                            &mut self.trait_add_filter,
                                        );
                                    });
                                    if changed {
                                        self.edit_buffer = staged.to_ti_save_pretty();
                                        self.last_error = None;
                                    }
                                    ui.separator();
                                }

                                if !trait_helper
                                    && let TiValue::Array(arr) = &mut staged
                                    && Self::is_simple_list(arr)
                                {
                                    let mut changed = false;
                                    ui.group(|ui| {
                                        ui.label(statics::EN_SIMPLE_LIST_EDITOR);
                                        changed = Self::render_simple_list_editor(ui, arr);
                                    });
                                    if changed {
                                        self.edit_buffer = staged.to_ti_save_pretty();
                                        self.last_error = None;
                                    }
                                    ui.separator();
                                }

                                if let TiValue::Object(map) = &mut staged
                                    && Self::is_simple_object(map)
                                {
                                    let mut changed = false;
                                    ui.group(|ui| {
                                        ui.label(statics::EN_SIMPLE_OBJECT_EDITOR);
                                        changed = Self::render_simple_object_editor(ui, map);
                                    });
                                    if changed {
                                        self.edit_buffer = staged.to_ti_save_pretty();
                                        self.last_error = None;
                                    }
                                    ui.separator();
                                }

                                if let TiValue::Object(map) = &mut staged
                                    && !Self::is_simple_object(map)
                                {
                                    let mut changed = false;
                                    ui.group(|ui| {
                                        ui.label(statics::EN_MIXED_OBJECT_EDITOR);
                                        changed = self.render_mixed_object_editor(ui, prop, map);
                                    });
                                    if changed {
                                        self.edit_buffer = staged.to_ti_save_pretty();
                                        self.last_error = None;
                                    }
                                    ui.separator();
                                }
                            }

                            // Always display arrays/objects in a formatted multiline text box.
                            // Leave a small safety margin so we don't spill outside the viewport on
                            // some platforms/window configurations.
                            let editor_h = (ui.available_height() - 8.0).max(120.0);
                            let editor = egui::TextEdit::multiline(&mut self.edit_buffer)
                                .font(egui::TextStyle::Monospace)
                                .desired_rows(10)
                                .lock_focus(true)
                                .interactive(true);
                            let resp = ui.add_sized([ui.available_width(), editor_h], editor);
                            if resp.lost_focus()
                                && let Ok(v) = TiValue::parse_json5(self.edit_buffer.trim())
                            {
                                self.edit_buffer = v.to_ti_save_pretty();
                                self.last_error = None;
                            }
                        }
                    } else {
                        ui.label(statics::EN_SELECT_PROPERTY);
                    }
                });
        });
    }

    fn render_ref_list_table(
        &mut self,
        ui: &mut egui::Ui,
        ids: &[i64],
        id_lookup: &std::collections::HashMap<i64, (String, usize)>,
        id_to_display_name: &std::collections::HashMap<i64, String>,
    ) {
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 6.0;
        ui.push_id("ref_list_table", |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(60.0).resizable(false))
                .column(Column::remainder().resizable(true))
                .header(row_h, |#[allow(unused_mut)] mut header| {
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong(statics::EN_COL_REF);
                    });
                })
                .body(|#[allow(unused_mut)] mut body| {
                    for id in ids {
                        body.row(row_h, |#[allow(unused_mut)] mut row| {
                            row.col(|ui| {
                                if ui.small_button(statics::EN_BTN_GO).clicked() {
                                    if let Some((ref_group, _)) = id_lookup.get(id) {
                                        self.select_object_programmatic(ref_group, *id, true, true);
                                    } else {
                                        self.last_error =
                                            Some(format!("Reference ID {id} not found"));
                                    }
                                }
                            });
                            row.col(|ui| {
                                let name = id_to_display_name
                                    .get(id)
                                    .map(String::as_str)
                                    .unwrap_or(statics::EN_EMPTY);
                                if name.is_empty() {
                                    ui.label(format!("{id}"));
                                } else {
                                    ui.label(format!("{id}: {name}"));
                                }
                            });
                        });
                    }
                });
        });
    }

    fn render_key_value_ref_table(
        &mut self,
        ui: &mut egui::Ui,
        rows: &[(i64, String)],
        id_lookup: &std::collections::HashMap<i64, (String, usize)>,
        id_to_display_name: &std::collections::HashMap<i64, String>,
    ) {
        let row_h = ui.text_style_height(&egui::TextStyle::Body) + 6.0;
        ui.push_id("key_value_ref_table", |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(60.0).resizable(false))
                .column(Column::remainder().resizable(true))
                .column(Column::initial(120.0).resizable(true))
                .header(row_h, |#[allow(unused_mut)] mut header| {
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong(statics::EN_COL_KEY);
                    });
                    header.col(|ui| {
                        ui.strong(statics::EN_COL_VALUE);
                    });
                })
                .body(|#[allow(unused_mut)] mut body| {
                    for (id, v) in rows {
                        body.row(row_h, |#[allow(unused_mut)] mut row| {
                            row.col(|ui| {
                                if ui.small_button(statics::EN_BTN_GO).clicked() {
                                    if let Some((ref_group, _)) = id_lookup.get(id) {
                                        self.select_object_programmatic(ref_group, *id, true, true);
                                    } else {
                                        self.last_error =
                                            Some(format!("Reference ID {id} not found"));
                                    }
                                }
                            });
                            row.col(|ui| {
                                let name = id_to_display_name
                                    .get(id)
                                    .map(String::as_str)
                                    .unwrap_or(statics::EN_EMPTY);
                                if name.is_empty() {
                                    ui.label(format!("{id}"));
                                } else {
                                    ui.label(format!("{id}: {name}"));
                                }
                            });
                            row.col(|ui| {
                                ui.monospace(v);
                            });
                        });
                    }
                });
        });
    }
}

fn value_preview(val: &TiValue) -> String {
    match val {
        TiValue::Null => "null".to_string(),
        TiValue::Bool(v) => v.to_string(),
        TiValue::Number(n) => match n {
            crate::value::TiNumber::I64(v) => v.to_string(),
            crate::value::TiNumber::U64(v) => v.to_string(),
            crate::value::TiNumber::F64(v) => {
                if v.is_nan() {
                    "NaN".to_string()
                } else if v.is_infinite() {
                    if v.is_sign_negative() {
                        "-Infinity".to_string()
                    } else {
                        "Infinity".to_string()
                    }
                } else {
                    let mut buf = ryu::Buffer::new();
                    let s = buf.format(*v);
                    if s.contains('e') {
                        s.replace('e', "E")
                    } else {
                        s.to_string()
                    }
                }
            }
        },
        TiValue::String(s) => {
            let mut s = s.clone();
            if s.len() > 60 {
                s.truncate(57);
                s.push_str("...");
            }
            s
        }
        TiValue::Array(values) => format!("[{}]", values.len()),
        TiValue::Object(map) => format!("{{{}}}", map.len()),
    }
}

fn direct_faction_id(save: &crate::LoadedSave, group: &str, object_id: i64) -> Option<i64> {
    save.get_object_value(group, object_id)?
        .get(statics::TI_PROP_FACTION)?
        .is_relational_ref()
}

fn hab_faction_id(save: &crate::LoadedSave, hab_id: i64) -> Option<i64> {
    direct_faction_id(save, statics::TI_GROUP_HAB_STATE, hab_id)
}

fn councilor_faction_id(save: &crate::LoadedSave, councilor_id: i64) -> Option<i64> {
    direct_faction_id(save, statics::TI_GROUP_COUNCILOR_STATE, councilor_id)
}

fn org_faction_id(save: &crate::LoadedSave, org_id: i64) -> Option<i64> {
    save.get_object_value(statics::TI_GROUP_ORG_STATE, org_id)?
        .get(statics::TI_PROP_FACTION_ORBIT)?
        .is_relational_ref()
}

fn org_tier(save: &crate::LoadedSave, org_id: i64) -> Option<i64> {
    match save
        .get_object_value(statics::TI_GROUP_ORG_STATE, org_id)?
        .get(statics::TI_PROP_TIER)?
    {
        crate::TiValue::Number(n) => n.as_i64(),
        _ => None,
    }
}

fn hab_module_faction_id(save: &crate::LoadedSave, module_id: i64) -> Option<i64> {
    let module_val = save.get_object_value(statics::TI_GROUP_HAB_MODULE_STATE, module_id)?;
    let sector_id = module_val
        .get(statics::TI_PROP_SECTOR)?
        .is_relational_ref()?;
    let sector_val = save.get_object_value(statics::TI_GROUP_SECTOR_STATE, sector_id)?;
    sector_val
        .get(statics::TI_PROP_FACTION)?
        .is_relational_ref()
}

fn hab_module_construction_completed(save: &crate::LoadedSave, module_id: i64) -> Option<bool> {
    let module_val = save.get_object_value(statics::TI_GROUP_HAB_MODULE_STATE, module_id)?;
    match module_val.get(statics::TI_PROP_CONSTRUCTION_COMPLETED)? {
        crate::TiValue::Bool(b) => Some(*b),
        _ => None,
    }
}

fn array_of_relational_refs(val: &TiValue) -> Option<Vec<i64>> {
    let TiValue::Array(items) = val else {
        return None;
    };
    if items.is_empty() {
        return None;
    }

    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(item.is_relational_ref()?);
    }
    Some(out)
}

fn array_of_key_value_refs(val: &TiValue) -> Option<Vec<(i64, String)>> {
    let TiValue::Array(items) = val else {
        return None;
    };
    if items.is_empty() {
        return None;
    }

    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let TiValue::Object(map) = item else {
            return None;
        };

        let key = map.get(statics::TI_FIELD_KEY_CAP)?;
        let key_id = key.is_relational_ref()?;

        let value = map.get(statics::TI_FIELD_VALUE_CAP)?;
        let value_s = value_preview(value);

        out.push((key_id, value_s));
    }
    Some(out)
}

impl TiseApp {
    /// Render the "update available" banner directly under the menu bar, if the
    /// background check found a newer release and the user hasn't dismissed it.
    /// No-op while the check is pending, failed, or reported up-to-date.
    fn show_update_banner(&mut self, ctx: &egui::Context) {
        if self.update_dismissed {
            return;
        }
        // Poll the check; bail unless a newer version is known. Clone the tag so
        // we don't hold an immutable borrow of self across the mutable UI calls.
        let latest_tag = match self.update_check.as_mut().and_then(|c| c.poll()) {
            Some(update) => update.latest_tag.clone(),
            None => return,
        };

        let mut dismiss = false;
        egui::TopBottomPanel::top("update_banner").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("{} {}", statics::EN_UPDATE_PREFIX, latest_tag));
                ui.hyperlink_to(statics::EN_UPDATE_DOWNLOAD, crate::update::RELEASES_PAGE);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(statics::EN_UPDATE_DISMISS).clicked() {
                        dismiss = true;
                    }
                });
            });
        });
        if dismiss {
            self.update_dismissed = true;
        }
    }
}

impl TiseApp {
    /// Apply the loaded config at startup: start loading game data if the saved
    /// folder is still valid, otherwise show the first-run prompt (once).
    fn init_game_data(&mut self, config: crate::gamedata::Config) {
        self.app_config = config;
        let paths = self
            .app_config
            .game_dir
            .as_deref()
            .and_then(crate::gamedata::resolve_game_paths);
        if let Some(paths) = paths {
            self.game_data = Some(crate::gamedata::GameDataLoad::spawn(paths));
        } else if !self.app_config.prompt_dismissed {
            self.open_game_dir_dialog();
        }
    }

    fn open_game_dir_dialog(&mut self) {
        self.game_dir_input = self
            .app_config
            .game_dir
            .clone()
            .or_else(crate::gamedata::detect_game_dir)
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        self.game_dir_error = None;
        self.game_dir_dialog_open = true;
    }

    fn persist_config(&mut self) {
        if let Err(e) = self.app_config.save() {
            self.last_error = Some(format!("Could not save settings: {e}"));
        }
    }

    /// Validate and adopt the folder typed/picked in the dialog.
    fn apply_game_dir(&mut self) {
        let chosen = PathBuf::from(self.game_dir_input.trim());
        match crate::gamedata::resolve_game_paths(&chosen) {
            Some(paths) => {
                self.app_config.game_dir = Some(chosen);
                self.app_config.prompt_dismissed = false;
                self.persist_config();
                self.game_data = Some(crate::gamedata::GameDataLoad::spawn(paths));
                self.game_dir_dialog_open = false;
                self.game_dir_error = None;
            }
            None => self.game_dir_error = Some(statics::EN_GAME_FOLDER_INVALID.to_string()),
        }
    }

    /// Loaded game data, if the background load has finished.
    fn game_data_snapshot(&mut self) -> Option<std::sync::Arc<crate::gamedata::GameData>> {
        self.game_data.as_mut().and_then(|g| g.poll())
    }

    fn show_game_dir_dialog(&mut self, ctx: &egui::Context) {
        if !self.game_dir_dialog_open {
            return;
        }
        let loaded = self.game_data_snapshot().is_some();
        let mut open = true;
        let mut apply = false;
        let mut close = false;
        let mut skip = false;
        let mut clear = false;
        egui::Window::new(statics::EN_WINDOW_GAME_FOLDER)
            .collapsible(false)
            .open(&mut open)
            .default_width(560.0)
            .show(ctx, |ui| {
                ui.label(statics::EN_GAME_FOLDER_INTRO);
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.game_dir_input)
                            .hint_text(statics::EN_GAME_FOLDER_HINT)
                            .desired_width(ui.available_width() - 90.0),
                    );
                    if ui.button(statics::EN_GAME_FOLDER_BROWSE).clicked()
                        && let Some(dir) = rfd::FileDialog::new().pick_folder()
                    {
                        self.game_dir_input = dir.display().to_string();
                        self.game_dir_error = None;
                    }
                });
                if let Some(err) = &self.game_dir_error {
                    ui.colored_label(egui::Color32::RED, err);
                }
                if let Some(dir) = &self.app_config.game_dir {
                    if loaded {
                        ui.label(format!(
                            "{} {}",
                            statics::EN_GAME_FOLDER_LOADED,
                            dir.display()
                        ));
                    } else {
                        ui.label(statics::EN_GAME_FOLDER_LOADING);
                    }
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button(statics::EN_GAME_FOLDER_USE).clicked() {
                        apply = true;
                    }
                    if ui.button(statics::EN_GAME_FOLDER_DETECT).clicked() {
                        match crate::gamedata::detect_game_dir() {
                            Some(p) => {
                                self.game_dir_input = p.display().to_string();
                                self.game_dir_error = None;
                            }
                            None => {
                                self.game_dir_error =
                                    Some(statics::EN_GAME_FOLDER_DETECT_FAILED.to_string());
                            }
                        }
                    }
                    if self.app_config.game_dir.is_some() {
                        if ui.button(statics::EN_GAME_FOLDER_CLEAR).clicked() {
                            clear = true;
                        }
                        if ui.button(statics::EN_GAME_FOLDER_CANCEL).clicked() {
                            close = true;
                        }
                    } else if ui.button(statics::EN_GAME_FOLDER_SKIP).clicked() {
                        skip = true;
                    }
                });
            });
        if apply {
            self.apply_game_dir();
        }
        if clear {
            self.app_config.game_dir = None;
            self.app_config.prompt_dismissed = true;
            self.persist_config();
            self.game_data = None;
            close = true;
        }
        if skip {
            self.app_config.prompt_dismissed = true;
            self.persist_config();
            close = true;
        }
        if close || !open {
            self.game_dir_dialog_open = false;
        }
    }

    /// Chip-style editor for `traitTemplateNames`: click a trait to remove it,
    /// search the game's trait list to add one. `catalog` is `None` when no game
    /// folder is configured; free-text add still works then. Returns true if
    /// `arr` changed.
    fn render_trait_editor(
        ui: &mut egui::Ui,
        arr: &mut Vec<TiValue>,
        catalog: Option<&[String]>,
        filter: &mut String,
    ) -> bool {
        let mut changed = false;
        let current: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();

        const PANE_HEIGHT: f32 = 200.0;
        let mut remove: Option<usize> = None;
        let mut add: Option<String> = None;

        // Two side-by-side panes: what the councillor has, and what can be added.
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.strong(format!("{} ({})", statics::EN_TRAIT_CURRENT, current.len()));
                ui.weak(statics::EN_TRAIT_EDITOR_REMOVE_HINT);
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("trait_current")
                    .min_scrolled_height(PANE_HEIGHT)
                    .max_height(PANE_HEIGHT)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for (i, name) in current.iter().enumerate() {
                                let duplicate = current[..i].contains(name);
                                let unknown = catalog.is_some_and(|c| !c.contains(name));
                                let mut text = egui::RichText::new(format!("{name}  \u{2715}"));
                                let mut tip = None;
                                if duplicate {
                                    text = text.color(egui::Color32::LIGHT_RED);
                                    tip = Some(statics::EN_TRAIT_DUPLICATE);
                                } else if unknown {
                                    text = text.color(egui::Color32::ORANGE);
                                    tip = Some(statics::EN_TRAIT_UNKNOWN);
                                }
                                let mut resp = ui.button(text);
                                if let Some(tip) = tip {
                                    resp = resp.on_hover_text(tip);
                                }
                                if resp.clicked() {
                                    remove = Some(i);
                                }
                            }
                        });
                    });
            });

            cols[1].group(|ui| {
                ui.strong(statics::EN_TRAIT_ADD);
                ui.add(
                    egui::TextEdit::singleline(filter)
                        .hint_text(statics::EN_TRAIT_SEARCH_HINT)
                        .desired_width(f32::INFINITY),
                );
                ui.separator();

                let needle = filter.trim().to_lowercase();
                if let Some(catalog) = catalog {
                    egui::ScrollArea::vertical()
                        .id_salt("trait_picker")
                        .min_scrolled_height(PANE_HEIGHT - 28.0)
                        .max_height(PANE_HEIGHT - 28.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for name in catalog
                                .iter()
                                .filter(|n| !current.contains(n))
                                .filter(|n| needle.is_empty() || n.to_lowercase().contains(&needle))
                            {
                                let row = egui::Button::new(format!("+  {name}"))
                                    .frame(false)
                                    .min_size(egui::vec2(ui.available_width(), 0.0));
                                if ui.add(row).clicked() {
                                    add = Some(name.clone());
                                }
                            }
                        });
                } else {
                    ui.weak(statics::EN_TRAIT_NO_GAME_DATA);
                }

                let typed = filter.trim();
                let offer_typed = !typed.is_empty()
                    && !current.iter().any(|n| n == typed)
                    && !catalog.is_some_and(|c| c.iter().any(|n| n == typed));
                if offer_typed
                    && ui
                        .button(format!("{} {}", statics::EN_TRAIT_ADD_TYPED, typed))
                        .clicked()
                {
                    add = Some(typed.to_string());
                }
            });
        });

        if let Some(i) = remove {
            arr.remove(i);
            changed = true;
        }

        if let Some(name) = add {
            arr.push(TiValue::String(name));
            filter.clear();
            changed = true;
        }
        changed
    }
}

impl eframe::App for TiseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Undo/Redo shortcuts.
        // Note: we explicitly consume these keys so egui text editors don't also apply their own
        // internal undo/redo to our edit buffers.
        let mut do_undo = false;
        let mut do_redo = false;
        ctx.input_mut(|i| {
            let ctrl_shift = egui::Modifiers {
                shift: true,
                ..egui::Modifiers::CTRL
            };
            if i.consume_key(ctrl_shift, egui::Key::Z) {
                do_redo = true;
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Y) {
                do_redo = true;
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Z) {
                do_undo = true;
            }
        });
        if do_undo {
            self.undo();
            ctx.request_repaint();
        }
        if do_redo {
            self.redo();
            ctx.request_repaint();
        }

        // Keyboard shortcuts for history navigation.
        if ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowLeft)) {
            self.go_back();
        }
        if ctx.input(|i| i.modifiers.alt && i.key_pressed(egui::Key::ArrowRight)) {
            self.go_forward();
        }

        // Mouse back/forward buttons (common on Windows/Linux).
        if ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Extra1)) {
            self.go_back();
        }
        if ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Extra2)) {
            self.go_forward();
        }

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if ui.button(statics::EN_BTN_OPEN).clicked() {
                    self.open_file();
                }

                let has_save = self.save.is_some();
                if ui
                    .add_enabled(has_save, egui::Button::new(statics::EN_BTN_SAVE_AS))
                    .clicked()
                {
                    self.save_file();
                }

                if ui
                    .add_enabled(has_save, egui::Button::new(statics::EN_BTN_COMPARE))
                    .clicked()
                {
                    self.compare_against();
                }

                if ui.button(statics::EN_BTN_GAME_FOLDER).clicked() {
                    self.open_game_dir_dialog();
                }

                if ui.button(statics::EN_BTN_ABOUT).clicked() {
                    self.about_open = true;
                }

                if ui.button(statics::EN_BTN_TOGGLE_THEME).clicked() {
                    self.theme_dark = !self.theme_dark;
                    if self.theme_dark {
                        ctx.set_visuals(egui::Visuals::dark());
                    } else {
                        ctx.set_visuals(egui::Visuals::light());
                    }
                }

                ui.separator();
                let can_undo = self.save.is_some() && !self.undo_stack.is_empty();
                let can_redo = self.save.is_some() && !self.redo_stack.is_empty();
                if ui
                    .add_enabled(can_undo, egui::Button::new(statics::EN_BTN_UNDO))
                    .clicked()
                {
                    self.undo();
                }
                if ui
                    .add_enabled(can_redo, egui::Button::new(statics::EN_BTN_REDO))
                    .clicked()
                {
                    self.redo();
                }
                let can_changes = self.save.is_some();
                if ui
                    .add_enabled(can_changes, egui::Button::new(statics::EN_BTN_CHANGES))
                    .clicked()
                {
                    self.changes_open = true;
                }

                // Always-visible nav buttons (easier to discover than a menu).
                ui.separator();
                let can_back = !self.history_back.is_empty();
                let can_fwd = !self.history_forward.is_empty();
                if ui
                    .add_enabled(can_back, egui::Button::new(statics::EN_NAV_BACK))
                    .clicked()
                {
                    self.go_back();
                }
                if ui
                    .add_enabled(can_fwd, egui::Button::new(statics::EN_NAV_FORWARD))
                    .clicked()
                {
                    self.go_forward();
                }
                let has_save = self.save.is_some();
                if ui
                    .add_enabled(has_save, egui::Button::new(statics::EN_NAV_GO_TO_ID))
                    .clicked()
                {
                    self.go_to_id_open = true;
                    self.go_to_id_input.clear();
                    self.go_to_id_request_focus = true;
                }
                if ui
                    .add_enabled(
                        has_save,
                        egui::Button::new(statics::EN_BTN_SEARCH_REF_BROWSER),
                    )
                    .clicked()
                {
                    self.search_ref_browser_open = true;
                    self.search_ref_browser_request_focus = true;
                }
                if ui
                    .add_enabled(has_save, egui::Button::new(statics::EN_BTN_SEARCH_ITEMS))
                    .clicked()
                {
                    self.search_items_open = true;
                    self.search_items_request_focus = true;
                }

                if !self.status.is_empty() {
                    ui.separator();
                    ui.label(&self.status);
                }
            });
        });

        self.show_update_banner(ctx);
        self.show_game_dir_dialog(ctx);

        if self.changes_open {
            let mut open = self.changes_open;
            let mut go_to_action_idx = None;

            egui::Window::new(statics::EN_WINDOW_CHANGES)
                .collapsible(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    if self.undo_stack.is_empty() {
                        ui.label(statics::EN_CHANGES_NONE);
                    } else {
                        ui.push_id("changes_scroll", |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (i, action) in self.undo_stack.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}.", i + 1));
                                        if ui.small_button(statics::EN_BTN_GO).clicked() {
                                            go_to_action_idx = Some(i);
                                        }

                                        let mut text = if let (Some(b), Some(a)) =
                                            (&action.before, &action.after)
                                        {
                                            let s_b = value_preview(b);
                                            let s_a = value_preview(a);
                                            format!(
                                                "{}.{}: {} -> {}",
                                                action.object_id, action.prop, s_b, s_a
                                            )
                                        } else {
                                            // Fallback if values missing (should be rare/legacy).
                                            format!(
                                                "{}.{}: {}",
                                                action.object_id, action.prop, action.description
                                            )
                                        };

                                        // Limit line length as requested.
                                        if text.len() > 100 {
                                            text.truncate(97);
                                            text.push_str("...");
                                        }
                                        ui.label(text);
                                    });
                                }
                            });
                        });
                    }
                    ui.separator();
                    ui.label(statics::EN_CHANGES_TIP);
                });
            self.changes_open = open;

            if let Some(idx) = go_to_action_idx
                && let Some(save) = self.save.take()
            {
                let action = self.undo_stack[idx].clone();
                self.navigate_to_action_target(&save, &action);
                self.save = Some(save);
            }
        }

        if self.about_open {
            let mut open = self.about_open;
            egui::Window::new(statics::EN_WINDOW_ABOUT)
                .collapsible(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.heading(statics::EN_ABOUT_HEADING);
                    ui.label(format!(
                        "{} {}",
                        statics::EN_ABOUT_VERSION,
                        env!("CARGO_PKG_VERSION")
                    ));
                    ui.separator();
                    ui.label(statics::EN_ABOUT_SHORTCUTS);
                    ui.label(statics::EN_ABOUT_SHORTCUT_ALT);
                    ui.label(statics::EN_ABOUT_SHORTCUT_MOUSE);
                    ui.separator();
                    ui.hyperlink_to(
                        format!("{} @ {}", statics::EN_PROJECT_REPO, statics::GITHUB_URL),
                        statics::GITHUB_URL,
                    );
                });
            self.about_open = open;
        }

        if self.diff_open {
            self.show_diff_window(ctx);
        }

        if let Some(err) = self.last_error.clone() {
            egui::TopBottomPanel::top("error_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(statics::EN_BTN_CLEAR).clicked() {
                            self.last_error = None;
                        }
                    });
                });
            });
        }

        if self.save.is_none() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading(statics::EN_HOME_HEADING);
                ui.label(statics::EN_HOME_INSTRUCTIONS);
            });
            return;
        }

        let mut save = self.save.take().expect("checked above");

        // We clone groups (List of strings) to allow sorting in UI (cheap).
        // Larger maps are referenced directly from `save.index`.
        let mut groups = save.index.groups.clone();

        // Use references for the massive maps.
        let objects_by_group = &save.index.objects_by_group;
        let id_lookup = &save.index.id_lookup;
        let id_to_display_name = &save.index.id_to_display_name;

        let save_format = save.format;
        let dirty = save.dirty;
        let game_id = save.game_id();

        // Match Python UX: groups sorted by display name (namespace stripped).
        groups.sort_by_key(|g| LoadedSave::group_display_name(g).to_lowercase());

        if self.search_ref_browser_open {
            let mut open = self.search_ref_browser_open;
            egui::Window::new(statics::EN_WINDOW_SEARCH_REF_BROWSER)
                .collapsible(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(statics::EN_LABEL_SEARCH);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.search_ref_browser_query)
                                .hint_text(statics::EN_HINT_SEARCH),
                        );
                        if self.search_ref_browser_request_focus {
                            resp.request_focus();
                            self.search_ref_browser_request_focus = false;
                        }
                        if ui.small_button(statics::EN_BTN_CLEAR).clicked() {
                            self.search_ref_browser_query.clear();
                        }
                    });
                    ui.separator();

                    if self.search_ref_cache.is_none()
                        || self.search_ref_cache_query != self.search_ref_browser_query
                    {
                        let query = self.search_ref_browser_query.trim();
                        let query_lower = query.to_lowercase();

                        let mut ids: Vec<i64> = id_to_display_name.keys().copied().collect();
                        ids.sort_unstable();

                        let filtered_ids: Vec<i64> = if query.is_empty() {
                            ids
                        } else {
                            ids.into_iter()
                                .filter(|id| {
                                    let name = id_to_display_name
                                        .get(id)
                                        .map(String::as_str)
                                        .unwrap_or(statics::EN_EMPTY);

                                    id.to_string().contains(query)
                                        || name.to_lowercase().contains(&query_lower)
                                })
                                .collect()
                        };
                        self.search_ref_cache = Some(filtered_ids);
                        self.search_ref_cache_query = self.search_ref_browser_query.clone();
                    }

                    // To avoid borrow checker conflict, we clone the ids out of the cache.
                    // (Cloning a list of i64 is cheap, unlike processing a massive map).
                    let filtered_ids = self.search_ref_cache.as_ref().unwrap().clone();

                    if filtered_ids.is_empty() {
                        ui.label(statics::EN_SEARCH_NO_MATCHES);
                        return;
                    }

                    ui.label(format!("{} results found", filtered_ids.len()));

                    let row_h = ui.text_style_height(&egui::TextStyle::Body) + 6.0;
                    ui.push_id("search_ref_browser_scroll", |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.push_id("search_ref_browser_table", |ui| {
                                    TableBuilder::new(ui)
                                        .striped(true)
                                        .cell_layout(egui::Layout::left_to_right(
                                            egui::Align::Center,
                                        ))
                                        .column(Column::initial(60.0).resizable(false))
                                        .column(Column::initial(90.0).resizable(true))
                                        .column(Column::remainder().resizable(true))
                                        .header(row_h, |#[allow(unused_mut)] mut header| {
                                            header.col(|ui| {
                                                ui.strong("");
                                            });
                                            header.col(|ui| {
                                                ui.strong(statics::EN_COL_ID);
                                            });
                                            header.col(|ui| {
                                                ui.strong(statics::EN_COL_NAME);
                                            });
                                        })
                                        .body(|#[allow(unused_mut)] mut body| {
                                            body.rows(
                                                row_h,
                                                filtered_ids.len(),
                                                |#[allow(unused_mut)] mut row| {
                                                    let id = filtered_ids[row.index()];
                                                    let name = id_to_display_name
                                                        .get(&id)
                                                        .map(String::as_str)
                                                        .unwrap_or(statics::EN_EMPTY);

                                                    row.col(|ui| {
                                                        if ui
                                                            .small_button(statics::EN_BTN_GO)
                                                            .clicked()
                                                        {
                                                            if let Some((ref_group, _)) =
                                                                save.index.id_lookup.get(&id)
                                                            {
                                                                self.select_object_programmatic(
                                                                    ref_group, id, true, true,
                                                                );
                                                            } else {
                                                                self.last_error = Some(format!(
                                                                    "Reference ID {id} not found"
                                                                ));
                                                            }
                                                        }
                                                    });
                                                    row.col(|ui| {
                                                        ui.monospace(id.to_string());
                                                    });
                                                    row.col(|ui| {
                                                        ui.label(name);
                                                    });
                                                },
                                            );
                                        });
                                });
                            });
                    });
                });

            self.search_ref_browser_open = open;
        }

        if self.search_items_open {
            let mut open = self.search_items_open;
            egui::Window::new(statics::EN_WINDOW_SEARCH_ITEMS)
                .collapsible(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(statics::EN_LABEL_SEARCH);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.search_items_query)
                                .hint_text(statics::EN_HINT_SEARCH_ITEMS),
                        );
                        if self.search_items_request_focus {
                            resp.request_focus();
                            self.search_items_request_focus = false;
                        }
                        if ui.small_button(statics::EN_BTN_CLEAR).clicked() {
                            self.search_items_query.clear();
                        }
                    });
                    ui.separator();

                    if self.search_items_cache.is_none()
                        || self.search_items_cache_query != self.search_items_query
                    {
                        let query = self.search_items_query.trim();
                        if !query.is_empty() {
                            // Cap results to keep the UI responsive on very large saves.
                            // Pass our local `save` reference directly.
                            let mut hits = Self::compute_item_search_hits(&save, query, 5_000);
                            Self::sort_item_search_hits(
                                &mut hits,
                                self.search_items_sort_key,
                                self.search_items_sort_asc,
                            );
                            self.search_items_cache = Some(hits);
                            self.search_items_cache_query = self.search_items_query.clone();
                        } else {
                            self.search_items_cache = Some(Vec::new());
                            self.search_items_cache_query = String::new();
                        }
                    }

                    // To avoid borrow checker conflict, we retrieve the hits.
                    // However, cloning `ItemSearchHit` (contains Strings) is not as cheap.
                    // But `self.search_items_cache` is inside `self`.
                    // We need to avoid holding a reference to `self` while also mutating `self`.
                    // The best way here is unfortunately to clone the filtered view for the UI loop,
                    // or restructure to not need `&self` for `select_object_programmatic`.
                    // Since the hit count is capped (5000), cloning 5000 structs is acceptable.
                    let hits = self.search_items_cache.as_ref().unwrap().clone();

                    if hits.is_empty() {
                        if self.search_items_query.trim().is_empty() {
                            ui.label(statics::EN_SEARCH_ENTER_QUERY);
                        } else {
                            ui.label(statics::EN_SEARCH_NO_MATCHES);
                        }
                        return;
                    }

                    ui.label(format!("{} results found", hits.len()));

                    let row_h = ui.text_style_height(&egui::TextStyle::Body) + 6.0;
                    let mut resort_requested = false;
                    ui.push_id("search_items_scroll", |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.push_id("search_items_table", |ui| {
                                    TableBuilder::new(ui)
                                        .striped(true)
                                        .cell_layout(egui::Layout::left_to_right(
                                            egui::Align::Center,
                                        ))
                                        .column(Column::initial(60.0).resizable(false))
                                        .column(Column::initial(180.0).resizable(true))
                                        .column(Column::initial(90.0).resizable(true))
                                        .column(Column::initial(220.0).resizable(true))
                                        .column(Column::remainder().resizable(true))
                                        .header(row_h, |#[allow(unused_mut)] mut header| {
                                            header.col(|ui| {
                                                ui.strong("");
                                            });
                                            header.col(|ui| {
                                                let clicked = ui
                                                    .add(
                                                        egui::Button::new(statics::EN_COL_GROUP)
                                                            .frame(false),
                                                    )
                                                    .clicked();
                                                if self.search_items_sort_key == ItemSortKey::Group
                                                {
                                                    ui.label(if self.search_items_sort_asc {
                                                        statics::EN_GLYPH_SORT_ASC
                                                    } else {
                                                        statics::EN_GLYPH_SORT_DESC
                                                    });
                                                }
                                                if clicked {
                                                    if self.search_items_sort_key
                                                        == ItemSortKey::Group
                                                    {
                                                        self.search_items_sort_asc =
                                                            !self.search_items_sort_asc;
                                                    } else {
                                                        self.search_items_sort_key =
                                                            ItemSortKey::Group;
                                                        self.search_items_sort_asc = true;
                                                    }
                                                    // Force re-sort of cache
                                                    if let Some(hits) =
                                                        self.search_items_cache.as_mut()
                                                    {
                                                        Self::sort_item_search_hits(
                                                            hits,
                                                            self.search_items_sort_key,
                                                            self.search_items_sort_asc,
                                                        );
                                                    }
                                                    resort_requested = true;
                                                }
                                            });
                                            header.col(|ui| {
                                                let clicked = ui
                                                    .add(
                                                        egui::Button::new(statics::EN_COL_ID)
                                                            .frame(false),
                                                    )
                                                    .clicked();
                                                if self.search_items_sort_key == ItemSortKey::Id {
                                                    ui.label(if self.search_items_sort_asc {
                                                        statics::EN_GLYPH_SORT_ASC
                                                    } else {
                                                        statics::EN_GLYPH_SORT_DESC
                                                    });
                                                }
                                                if clicked {
                                                    if self.search_items_sort_key == ItemSortKey::Id
                                                    {
                                                        self.search_items_sort_asc =
                                                            !self.search_items_sort_asc;
                                                    } else {
                                                        self.search_items_sort_key =
                                                            ItemSortKey::Id;
                                                        self.search_items_sort_asc = true;
                                                    }
                                                    // Force re-sort of cache
                                                    if let Some(hits) =
                                                        self.search_items_cache.as_mut()
                                                    {
                                                        Self::sort_item_search_hits(
                                                            hits,
                                                            self.search_items_sort_key,
                                                            self.search_items_sort_asc,
                                                        );
                                                    }
                                                    resort_requested = true;
                                                }
                                            });
                                            header.col(|ui| {
                                                let clicked = ui
                                                    .add(
                                                        egui::Button::new(statics::EN_COL_PROPERTY)
                                                            .frame(false),
                                                    )
                                                    .clicked();
                                                if self.search_items_sort_key
                                                    == ItemSortKey::Property
                                                {
                                                    ui.label(if self.search_items_sort_asc {
                                                        statics::EN_GLYPH_SORT_ASC
                                                    } else {
                                                        statics::EN_GLYPH_SORT_DESC
                                                    });
                                                }
                                                if clicked {
                                                    if self.search_items_sort_key
                                                        == ItemSortKey::Property
                                                    {
                                                        self.search_items_sort_asc =
                                                            !self.search_items_sort_asc;
                                                    } else {
                                                        self.search_items_sort_key =
                                                            ItemSortKey::Property;
                                                        self.search_items_sort_asc = true;
                                                    }
                                                    // Force re-sort of cache
                                                    if let Some(hits) =
                                                        self.search_items_cache.as_mut()
                                                    {
                                                        Self::sort_item_search_hits(
                                                            hits,
                                                            self.search_items_sort_key,
                                                            self.search_items_sort_asc,
                                                        );
                                                    }
                                                    resort_requested = true;
                                                }
                                            });
                                            header.col(|ui| {
                                                let clicked = ui
                                                    .add(
                                                        egui::Button::new(statics::EN_COL_VALUE)
                                                            .frame(false),
                                                    )
                                                    .clicked();
                                                if self.search_items_sort_key == ItemSortKey::Value
                                                {
                                                    ui.label(if self.search_items_sort_asc {
                                                        statics::EN_GLYPH_SORT_ASC
                                                    } else {
                                                        statics::EN_GLYPH_SORT_DESC
                                                    });
                                                }
                                                if clicked {
                                                    if self.search_items_sort_key
                                                        == ItemSortKey::Value
                                                    {
                                                        self.search_items_sort_asc =
                                                            !self.search_items_sort_asc;
                                                    } else {
                                                        self.search_items_sort_key =
                                                            ItemSortKey::Value;
                                                        self.search_items_sort_asc = true;
                                                    }
                                                    // Force re-sort of cache
                                                    if let Some(hits) =
                                                        self.search_items_cache.as_mut()
                                                    {
                                                        Self::sort_item_search_hits(
                                                            hits,
                                                            self.search_items_sort_key,
                                                            self.search_items_sort_asc,
                                                        );
                                                    }
                                                    resort_requested = true;
                                                }
                                            });
                                        })
                                        .body(|#[allow(unused_mut)] mut body| {
                                            body.rows(
                                                row_h,
                                                hits.len(),
                                                |#[allow(unused_mut)] mut row| {
                                                    let hit = &hits[row.index()];
                                                    row.col(|ui| {
                                                    if ui.small_button(statics::EN_BTN_GO).clicked()
                                                    {
                                                        self.select_object_programmatic(
                                                            &hit.group,
                                                            hit.object_id,
                                                            true,
                                                            true,
                                                        );
                                                        self.selected_property =
                                                            Some(hit.prop.clone());
                                                        self.scroll_properties_to_selected = true;
                                                        self.scroll_align_center = true;
                                                        self.refresh_selected_property_from_save(
                                                            &save,
                                                        );
                                                    }
                                                });
                                                    row.col(|ui| {
                                                        ui.label(&hit.group_display);
                                                    });
                                                    row.col(|ui| {
                                                        ui.monospace(hit.object_id.to_string());
                                                    });
                                                    row.col(|ui| {
                                                        ui.monospace(&hit.prop);
                                                    });
                                                    row.col(|ui| {
                                                        ui.label(&hit.value_preview);
                                                    });
                                                },
                                            );
                                        });
                                });
                            });
                    });

                    if resort_requested {
                        ui.ctx().request_repaint();
                    }
                });

            self.search_items_open = open;
        }

        if self.go_to_id_open {
            let mut open = self.go_to_id_open;
            let mut close_requested = false;
            egui::Window::new(statics::EN_WINDOW_GO_TO_ID)
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(statics::EN_GO_TO_ID_PROMPT);
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.go_to_id_input)
                            .hint_text(statics::EN_GO_TO_ID_HINT),
                    );
                    if self.go_to_id_request_focus {
                        resp.request_focus();
                        self.go_to_id_request_focus = false;
                    }
                    let pressed_enter =
                        resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    ui.horizontal(|ui| {
                        let go_clicked = ui.button(statics::EN_BTN_GO).clicked() || pressed_enter;
                        if go_clicked {
                            match self.go_to_id_input.trim().parse::<i64>() {
                                Ok(id) => {
                                    if let Some((group, _)) = save.index.id_lookup.get(&id) {
                                        self.select_object_programmatic(group, id, true, false);
                                        close_requested = true;
                                        self.last_error = None;
                                    } else {
                                        self.last_error = Some(format!("ID {id} not found"));
                                    }
                                }
                                Err(_) => {
                                    self.last_error =
                                        Some(statics::EN_ERR_INVALID_ID_INTEGER.to_string());
                                }
                            }
                        }
                        if ui.button(statics::EN_BTN_CANCEL).clicked() {
                            close_requested = true;
                        }
                    });
                });

            if close_requested {
                open = false;
            }
            self.go_to_id_open = open;
        }

        // The bottom status bar must be shown before side/central panels so it reserves
        // space across the full window width (otherwise it only spans the remaining
        // central area after left side panels are laid out).
        egui::TopBottomPanel::bottom("bottom_status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let file_label = save
                    .source_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| statics::EN_PLACEHOLDER_UNSAVED.to_string());
                ui.label(file_label);
                ui.separator();
                ui.label(format!("format: {:?}", save_format));
                if let Some(gid) = game_id {
                    ui.separator();
                    ui.label(format!("game id: {gid}"));
                }
                ui.separator();
                ui.label(format!("groups: {}", groups.len()));
                ui.separator();
                ui.label(format!("objects: {}", id_lookup.len()));
                ui.separator();
                ui.label(format!(
                    "{} {} {} {}",
                    statics::EN_HISTORY_LABEL,
                    self.history_back.len(),
                    statics::EN_HISTORY_BACK,
                    self.history_forward.len()
                ));
                ui.separator();
                ui.label(format!(
                    "{} {}",
                    statics::EN_LABEL_CHANGES_COUNT,
                    self.undo_stack.len()
                ));
                if dirty {
                    ui.separator();
                    ui.colored_label(egui::Color32::YELLOW, statics::EN_BADGE_DIRTY);
                }
            });
        });

        egui::SidePanel::left("groups_panel")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.heading(statics::EN_HEADING_GROUPS);
                ui.separator();
                let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                ui.push_id("groups_scroll", |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let mut by_category: BTreeMap<GroupCategory, Vec<&String>> =
                                BTreeMap::new();
                            for group in &groups {
                                by_category
                                    .entry(LoadedSave::group_category(group))
                                    .or_default()
                                    .push(group);
                            }
                            let selected_category = self
                                .selected_group
                                .as_deref()
                                .map(LoadedSave::group_category);
                            for (category, cat_groups) in by_category {
                                // Expand the selected group's category so it can be scrolled to.
                                let force_open = (self.scroll_groups_to_selected
                                    && selected_category == Some(category))
                                .then_some(true);
                                egui::CollapsingHeader::new(format!(
                                    "{} ({})",
                                    category.label(),
                                    cat_groups.len()
                                ))
                                .id_salt(("group_category", category))
                                .default_open(true)
                                .open(force_open)
                                .show(ui, |ui| {
                                    for group in cat_groups {
                                        let label = LoadedSave::group_display_name(group);
                                        let selected =
                                            self.selected_group.as_deref() == Some(group.as_str());
                                        let resp =
                                            Self::selectable_row_left(ui, selected, label, row_h);
                                        if selected && self.scroll_groups_to_selected {
                                            let align = if self.scroll_align_center {
                                                egui::Align::Center
                                            } else {
                                                egui::Align::Min
                                            };
                                            resp.scroll_to_me(Some(align));
                                            self.scroll_groups_to_selected = false;
                                            self.scroll_align_center = false;
                                        }
                                        if resp.clicked() {
                                            self.selected_group = Some(group.clone());
                                            self.selected_object_id = None;
                                            self.selected_property = None;
                                            self.edit_buffer.clear();
                                            self.raw_edit_mode = false;
                                            self.scroll_groups_to_selected = false;
                                            self.scroll_objects_to_selected = false;
                                            self.scroll_properties_to_selected = false;
                                            if group != statics::TI_GROUP_ORG_STATE {
                                                self.org_bulk_selected_ids.clear();
                                            }
                                        }
                                    }
                                });
                            }
                        });
                });
            });

        let mut org_add_faction_request: Option<i64> = None;

        // Singleton groups (e.g. TIGlobalResearchState) have nothing to pick from, so
        // select their only object and skip the objects pane.
        let singleton_object = self.selected_group.as_ref().and_then(|g| {
            match objects_by_group.get(g).map(Vec::as_slice) {
                Some([only]) => Some((g.clone(), only.id)),
                _ => None,
            }
        });
        if let Some((group, id)) = &singleton_object
            && self.selected_object_id != Some(*id)
        {
            self.select_object_internal(group, *id, false, false, false);
        }

        if singleton_object.is_none() {
            egui::SidePanel::left("objects_panel")
                .resizable(true)
                .default_width(360.0)
                .show(ctx, |ui| {
                    ui.heading(statics::EN_HEADING_OBJECTS);
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(statics::EN_LABEL_SORT);
                        ui.selectable_value(
                            &mut self.sort_objects_by_id,
                            false,
                            statics::EN_SORT_NAME,
                        );
                        ui.selectable_value(
                            &mut self.sort_objects_by_id,
                            true,
                            statics::EN_SORT_ID,
                        );
                    });
                    ui.separator();

                    let Some(group) = self.selected_group.clone() else {
                        ui.label(statics::EN_SELECT_GROUP);
                        return;
                    };

                    if group == statics::TI_GROUP_HAB_STATE {
                        let mut factions: Vec<_> = objects_by_group
                            .get(statics::TI_GROUP_FACTION_STATE)
                            .map(|v| v.iter().collect())
                            .unwrap_or_default();
                        factions.sort_by_key(|f| f.display_name.to_lowercase());
                        Self::render_faction_filter_combobox(
                            ui,
                            &factions,
                            &mut self.hab_faction_filter,
                            id_to_display_name,
                            "hab_faction_filter",
                        );
                        ui.separator();
                    }

                    if group == statics::TI_GROUP_COUNCILOR_STATE {
                        let mut factions: Vec<_> = objects_by_group
                            .get(statics::TI_GROUP_FACTION_STATE)
                            .map(|v| v.iter().collect())
                            .unwrap_or_default();
                        factions.sort_by_key(|f| f.display_name.to_lowercase());
                        Self::render_faction_filter_combobox(
                            ui,
                            &factions,
                            &mut self.councilor_faction_filter,
                            id_to_display_name,
                            "councilor_faction_filter",
                        );
                        ui.separator();
                    }

                    if group == statics::TI_GROUP_ORG_STATE {
                        let mut factions: Vec<_> = objects_by_group
                            .get(statics::TI_GROUP_FACTION_STATE)
                            .map(|v| v.iter().collect())
                            .unwrap_or_default();
                        factions.sort_by_key(|f| f.display_name.to_lowercase());
                        Self::render_org_faction_filter_combobox(
                            ui,
                            &factions,
                            &mut self.org_faction_filter,
                            id_to_display_name,
                            "org_faction_filter",
                        );
                        Self::render_tier_filter_combobox(
                            ui,
                            &mut self.org_tier_filter,
                            &[1, 2, 3, 4, 5],
                            "org_tier_filter",
                        );
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label(statics::EN_LABEL_ADD_TO_FACTION);
                            let selected_label = self
                                .org_add_faction_target
                                .and_then(|id| id_to_display_name.get(&id))
                                .cloned()
                                .unwrap_or_else(|| statics::EN_FILTER_SELECT_FACTION.to_string());
                            egui::ComboBox::from_id_salt("org_add_faction_target")
                                .selected_text(selected_label)
                                .show_ui(ui, |ui| {
                                    for faction in &factions {
                                        ui.selectable_value(
                                            &mut self.org_add_faction_target,
                                            Some(faction.id),
                                            &faction.display_name,
                                        );
                                    }
                                });
                        });

                        let selected_count = self.org_bulk_selected_ids.len();
                        ui.label(format!(
                            "{selected_count} {}",
                            statics::EN_LABEL_ORGS_SELECTED
                        ));

                        let can_add = selected_count > 0 && self.org_add_faction_target.is_some();
                        if ui
                            .add_enabled(can_add, egui::Button::new(statics::EN_BTN_ADD_TO_FACTION))
                            .clicked()
                            && let Some(faction_id) = self.org_add_faction_target
                        {
                            org_add_faction_request = Some(faction_id);
                        }
                        ui.separator();
                    }

                    if group == statics::TI_GROUP_HAB_MODULE_STATE {
                        let mut factions: Vec<_> = objects_by_group
                            .get(statics::TI_GROUP_FACTION_STATE)
                            .map(|v| v.iter().collect())
                            .unwrap_or_default();
                        factions.sort_by_key(|f| f.display_name.to_lowercase());
                        Self::render_faction_filter_combobox(
                            ui,
                            &factions,
                            &mut self.hab_module_faction_filter,
                            id_to_display_name,
                            "hab_module_faction_filter",
                        );
                        ui.horizontal(|ui| {
                            ui.label(statics::EN_LABEL_FILTER_CONSTRUCTION);
                            egui::ComboBox::from_id_salt("hab_module_construction_filter")
                                .selected_text(match self.hab_module_construction_filter {
                                    HabModuleConstructionFilter::All => {
                                        statics::EN_FILTER_CONSTRUCTION_ALL
                                    }
                                    HabModuleConstructionFilter::Completed => {
                                        statics::EN_FILTER_CONSTRUCTION_COMPLETED
                                    }
                                    HabModuleConstructionFilter::InProgress => {
                                        statics::EN_FILTER_CONSTRUCTION_IN_PROGRESS
                                    }
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.hab_module_construction_filter,
                                        HabModuleConstructionFilter::All,
                                        statics::EN_FILTER_CONSTRUCTION_ALL,
                                    );
                                    ui.selectable_value(
                                        &mut self.hab_module_construction_filter,
                                        HabModuleConstructionFilter::Completed,
                                        statics::EN_FILTER_CONSTRUCTION_COMPLETED,
                                    );
                                    ui.selectable_value(
                                        &mut self.hab_module_construction_filter,
                                        HabModuleConstructionFilter::InProgress,
                                        statics::EN_FILTER_CONSTRUCTION_IN_PROGRESS,
                                    );
                                });
                        });
                        ui.separator();
                    }

                    let mut objects: Vec<_> = objects_by_group
                        .get(&group)
                        .map(|v| v.iter().collect())
                        .unwrap_or_default();

                    if self.sort_objects_by_id {
                        objects.sort_by_key(|o| o.id);
                    } else {
                        objects.sort_by_key(|o| o.display_name.to_lowercase());
                    }

                    if group == statics::TI_GROUP_HAB_STATE
                        && let Some(faction_id) = self.hab_faction_filter
                    {
                        objects.retain(|obj| hab_faction_id(&save, obj.id) == Some(faction_id));
                    }

                    if group == statics::TI_GROUP_COUNCILOR_STATE
                        && let Some(faction_id) = self.councilor_faction_filter
                    {
                        objects
                            .retain(|obj| councilor_faction_id(&save, obj.id) == Some(faction_id));
                    }

                    if group == statics::TI_GROUP_ORG_STATE {
                        match self.org_faction_filter {
                            OrgFactionFilter::All => {}
                            OrgFactionFilter::NoFaction => {
                                objects.retain(|obj| org_faction_id(&save, obj.id).is_none());
                            }
                            OrgFactionFilter::Faction(faction_id) => {
                                objects.retain(|obj| {
                                    org_faction_id(&save, obj.id) == Some(faction_id)
                                });
                            }
                        }
                        if let Some(tier) = self.org_tier_filter {
                            objects.retain(|obj| org_tier(&save, obj.id) == Some(tier));
                        }
                    }

                    if group == statics::TI_GROUP_HAB_MODULE_STATE {
                        if let Some(faction_id) = self.hab_module_faction_filter {
                            objects.retain(|obj| {
                                hab_module_faction_id(&save, obj.id) == Some(faction_id)
                            });
                        }
                        match self.hab_module_construction_filter {
                            HabModuleConstructionFilter::All => {}
                            HabModuleConstructionFilter::Completed => {
                                objects.retain(|obj| {
                                    hab_module_construction_completed(&save, obj.id) == Some(true)
                                });
                            }
                            HabModuleConstructionFilter::InProgress => {
                                objects.retain(|obj| {
                                    hab_module_construction_completed(&save, obj.id) != Some(true)
                                });
                            }
                        }
                    }

                    let row_h = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
                    ui.push_id("objects_scroll", |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for obj in objects {
                                    let selected = self.selected_object_id == Some(obj.id);
                                    let text = format!("{}: {}", obj.id, obj.display_name);
                                    let is_org_group = group == statics::TI_GROUP_ORG_STATE;

                                    let resp = if is_org_group {
                                        let mut checked =
                                            self.org_bulk_selected_ids.contains(&obj.id);
                                        ui.horizontal(|ui| {
                                            if ui.checkbox(&mut checked, "").changed() {
                                                if checked {
                                                    self.org_bulk_selected_ids.insert(obj.id);
                                                } else {
                                                    self.org_bulk_selected_ids.remove(&obj.id);
                                                }
                                            }
                                            Self::selectable_row_left(
                                                ui,
                                                selected,
                                                text.as_str(),
                                                row_h,
                                            )
                                        })
                                        .inner
                                    } else {
                                        Self::selectable_row_left(
                                            ui,
                                            selected,
                                            text.as_str(),
                                            row_h,
                                        )
                                    };

                                    if selected && self.scroll_objects_to_selected {
                                        let align = if self.scroll_align_center {
                                            egui::Align::Center
                                        } else {
                                            egui::Align::Min
                                        };
                                        resp.scroll_to_me(Some(align));
                                        self.scroll_objects_to_selected = false;
                                        self.scroll_align_center = false;
                                    }
                                    if resp.clicked() {
                                        self.select_object_user(&group, obj.id);
                                    }
                                }
                            });
                    });
                });
        }

        if let Some(faction_id) = org_add_faction_request {
            self.add_selected_orgs_to_faction(&mut save, faction_id);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(group) = self.selected_group.clone() else {
                ui.label(statics::EN_SELECT_GROUP_LEFT);
                return;
            };
            let Some(object_id) = self.selected_object_id else {
                ui.label(statics::EN_SELECT_OBJECT);
                return;
            };

            let value_obj = save.get_object_value(&group, object_id).cloned();
            let Some(value_obj) = value_obj else {
                ui.colored_label(egui::Color32::RED, statics::EN_ERR_OBJECT_VALUE_MISSING);
                return;
            };

            ui.horizontal(|ui| {
                ui.heading(LoadedSave::group_display_name(&group));
                ui.separator();
                ui.label(format!("ID {object_id}"));
                if dirty {
                    ui.separator();
                    ui.colored_label(egui::Color32::YELLOW, statics::EN_BADGE_MODIFIED);
                }
            });
            ui.separator();

            let mut properties: Vec<_> = value_obj.iter().collect();
            properties.sort_by_key(|(k, _)| (*k).to_lowercase());

            // Movable horizontal split between Properties (top) and Edit (bottom).
            let total_h = ui.available_height();
            let default_h = (total_h * 0.70).max(200.0);
            let max_h = (total_h - 140.0).max(140.0);
            egui::Resize::default()
                .id_salt("properties_resize")
                .default_height(default_h)
                .min_height(140.0)
                .max_height(max_h)
                .resizable(true)
                .show(ui, |ui| {
                    self.render_properties_panel(
                        ui,
                        &properties,
                        &value_obj,
                        &save.index.id_lookup,
                        &save.index.id_to_display_name,
                    );
                });

            ui.separator();
            self.render_editor_panel(ui, &value_obj, &mut save);
        });

        self.save = Some(save);
    }
}

#[cfg(test)]
mod tests {
    use super::TiseApp;
    use super::{
        ItemSearchHit, ItemSortKey, hab_module_construction_completed, hab_module_faction_id,
    };
    use crate::statics;
    use crate::{LoadedSave, TiValue, value::TiNumber};
    use indexmap::IndexMap;

    fn make_ref(id: i64) -> TiValue {
        let mut map = IndexMap::new();
        map.insert(
            statics::TI_REF_FIELD_VALUE.to_string(),
            TiValue::Number(TiNumber::I64(id)),
        );
        TiValue::Object(map)
    }

    fn make_ti_entry(id: i64, props: IndexMap<String, TiValue>) -> TiValue {
        let mut key_ref = IndexMap::new();
        key_ref.insert(
            statics::TI_REF_FIELD_VALUE.to_string(),
            TiValue::Number(TiNumber::I64(id)),
        );
        let mut entry = IndexMap::new();
        entry.insert(
            statics::TI_FIELD_KEY_CAP.to_string(),
            TiValue::Object(key_ref),
        );
        entry.insert(
            statics::TI_FIELD_VALUE_CAP.to_string(),
            TiValue::Object(props),
        );
        TiValue::Object(entry)
    }

    fn make_hab_save(
        module_id: i64,
        module_props: IndexMap<String, TiValue>,
        sector_id: i64,
        sector_props: IndexMap<String, TiValue>,
    ) -> LoadedSave {
        let mut gamestates = IndexMap::new();
        gamestates.insert(
            statics::TI_GROUP_HAB_MODULE_STATE.to_string(),
            TiValue::Array(vec![make_ti_entry(module_id, module_props)]),
        );
        gamestates.insert(
            statics::TI_GROUP_SECTOR_STATE.to_string(),
            TiValue::Array(vec![make_ti_entry(sector_id, sector_props)]),
        );
        let mut root = IndexMap::new();
        root.insert(
            statics::TI_GAMESTATES.to_string(),
            TiValue::Object(gamestates),
        );
        LoadedSave::from_root_for_test(TiValue::Object(root))
    }

    #[test]
    fn is_simple_list_accepts_primitives_only() {
        assert!(TiseApp::is_simple_list(&[
            TiValue::Null,
            TiValue::Bool(true),
            TiValue::Number(TiNumber::I64(1)),
            TiValue::String("x".to_string()),
        ]));
        assert!(!TiseApp::is_simple_list(&[TiValue::Array(vec![])]));
        assert!(!TiseApp::is_simple_list(&[
            TiValue::Object(IndexMap::new())
        ]));
    }

    #[test]
    fn is_simple_object_accepts_nonempty_primitives_only() {
        let mut map = IndexMap::new();
        map.insert("a".to_string(), TiValue::Null);
        map.insert("b".to_string(), TiValue::Bool(false));
        map.insert("c".to_string(), TiValue::Number(TiNumber::U64(2)));
        map.insert("d".to_string(), TiValue::String("y".to_string()));
        assert!(TiseApp::is_simple_object(&map));

        let mut map2 = IndexMap::new();
        map2.insert("a".to_string(), TiValue::Array(vec![]));
        assert!(!TiseApp::is_simple_object(&map2));

        let map3: IndexMap<String, TiValue> = IndexMap::new();
        assert!(!TiseApp::is_simple_object(&map3));
    }

    #[test]
    fn search_items_sorting_by_id_works() {
        let mut hits = vec![
            ItemSearchHit {
                group: "G2".to_string(),
                group_display: "G2".to_string(),
                object_id: 5,
                prop: "b".to_string(),
                value_preview: "2".to_string(),
            },
            ItemSearchHit {
                group: "G1".to_string(),
                group_display: "G1".to_string(),
                object_id: 2,
                prop: "a".to_string(),
                value_preview: "1".to_string(),
            },
        ];

        TiseApp::sort_item_search_hits(&mut hits, ItemSortKey::Id, true);
        assert_eq!(hits[0].object_id, 2);
        assert_eq!(hits[1].object_id, 5);
    }

    #[test]
    fn hab_module_faction_id_resolves_full_chain() {
        let mut module_props = IndexMap::new();
        module_props.insert(statics::TI_PROP_SECTOR.to_string(), make_ref(200));
        let mut sector_props = IndexMap::new();
        sector_props.insert(statics::TI_PROP_FACTION.to_string(), make_ref(300));
        let save = make_hab_save(100, module_props, 200, sector_props);
        assert_eq!(hab_module_faction_id(&save, 100), Some(300));
    }

    #[test]
    fn hab_module_faction_id_returns_none_for_unknown_module() {
        let mut module_props = IndexMap::new();
        module_props.insert(statics::TI_PROP_SECTOR.to_string(), make_ref(200));
        let mut sector_props = IndexMap::new();
        sector_props.insert(statics::TI_PROP_FACTION.to_string(), make_ref(300));
        let save = make_hab_save(100, module_props, 200, sector_props);
        assert_eq!(hab_module_faction_id(&save, 999), None);
    }

    #[test]
    fn hab_module_faction_id_returns_none_when_sector_prop_missing() {
        let module_props = IndexMap::new(); // no "sector" key
        let sector_props = IndexMap::new();
        let save = make_hab_save(100, module_props, 200, sector_props);
        assert_eq!(hab_module_faction_id(&save, 100), None);
    }

    #[test]
    fn hab_module_faction_id_returns_none_when_sector_not_found() {
        // Module points at sector 999, but only sector 200 exists.
        let mut module_props = IndexMap::new();
        module_props.insert(statics::TI_PROP_SECTOR.to_string(), make_ref(999));
        let mut sector_props = IndexMap::new();
        sector_props.insert(statics::TI_PROP_FACTION.to_string(), make_ref(300));
        let save = make_hab_save(100, module_props, 200, sector_props);
        assert_eq!(hab_module_faction_id(&save, 100), None);
    }

    #[test]
    fn hab_module_faction_id_returns_none_when_faction_prop_missing() {
        let mut module_props = IndexMap::new();
        module_props.insert(statics::TI_PROP_SECTOR.to_string(), make_ref(200));
        let sector_props = IndexMap::new(); // no "faction" key
        let save = make_hab_save(100, module_props, 200, sector_props);
        assert_eq!(hab_module_faction_id(&save, 100), None);
    }

    fn make_construction_save(
        module_id: i64,
        construction_completed: Option<TiValue>,
    ) -> crate::LoadedSave {
        let mut module_props = IndexMap::new();
        if let Some(val) = construction_completed {
            module_props.insert(statics::TI_PROP_CONSTRUCTION_COMPLETED.to_string(), val);
        }
        // sector is not needed for construction tests; use a dummy sector with no entries
        let mut gamestates = IndexMap::new();
        gamestates.insert(
            statics::TI_GROUP_HAB_MODULE_STATE.to_string(),
            TiValue::Array(vec![make_ti_entry(module_id, module_props)]),
        );
        let mut root = IndexMap::new();
        root.insert(
            statics::TI_GAMESTATES.to_string(),
            TiValue::Object(gamestates),
        );
        crate::LoadedSave::from_root_for_test(TiValue::Object(root))
    }

    #[test]
    fn hab_module_construction_completed_returns_true_when_complete() {
        let save = make_construction_save(10, Some(TiValue::Bool(true)));
        assert_eq!(hab_module_construction_completed(&save, 10), Some(true));
    }

    #[test]
    fn hab_module_construction_completed_returns_false_when_in_progress() {
        let save = make_construction_save(10, Some(TiValue::Bool(false)));
        assert_eq!(hab_module_construction_completed(&save, 10), Some(false));
    }

    #[test]
    fn hab_module_construction_completed_returns_none_when_prop_missing() {
        let save = make_construction_save(10, None);
        assert_eq!(hab_module_construction_completed(&save, 10), None);
    }

    #[test]
    fn hab_module_construction_completed_returns_none_for_unknown_module() {
        let save = make_construction_save(10, Some(TiValue::Bool(true)));
        assert_eq!(hab_module_construction_completed(&save, 999), None);
    }

    #[test]
    fn hab_module_construction_completed_returns_none_when_prop_wrong_type() {
        let save =
            make_construction_save(10, Some(TiValue::Number(crate::value::TiNumber::I64(1))));
        assert_eq!(hab_module_construction_completed(&save, 10), None);
    }

    #[test]
    fn primitive_type_label_maps_each_primitive() {
        assert_eq!(
            TiseApp::primitive_type_label(&TiValue::Null),
            statics::EN_TYPE_NULL
        );
        assert_eq!(
            TiseApp::primitive_type_label(&TiValue::Bool(true)),
            statics::EN_TYPE_BOOL
        );
        assert_eq!(
            TiseApp::primitive_type_label(&TiValue::Number(TiNumber::I64(-1))),
            statics::EN_TYPE_I64
        );
        assert_eq!(
            TiseApp::primitive_type_label(&TiValue::Number(TiNumber::U64(1))),
            statics::EN_TYPE_U64
        );
        assert_eq!(
            TiseApp::primitive_type_label(&TiValue::Number(TiNumber::F64(1.5))),
            statics::EN_TYPE_F64
        );
        assert_eq!(
            TiseApp::primitive_type_label(&TiValue::String("x".to_string())),
            statics::EN_TYPE_STRING
        );
    }

    #[test]
    fn majority_primitive_label_picks_most_common() {
        let arr = vec![
            TiValue::String("a".to_string()),
            TiValue::String("b".to_string()),
            TiValue::Number(TiNumber::I64(1)),
        ];
        assert_eq!(
            TiseApp::majority_primitive_label(&arr),
            Some(statics::EN_TYPE_STRING)
        );
    }

    #[test]
    fn majority_primitive_label_empty_is_none() {
        assert_eq!(TiseApp::majority_primitive_label(&[]), None);
    }

    #[test]
    fn majority_primitive_label_tie_prefers_earliest() {
        // One bool, one string: tie at count 1. Bool appears first, so it wins.
        let arr = vec![TiValue::Bool(true), TiValue::String("a".to_string())];
        assert_eq!(
            TiseApp::majority_primitive_label(&arr),
            Some(statics::EN_TYPE_BOOL)
        );
        // Reverse order: string now appears first and wins the tie.
        let arr = vec![TiValue::String("a".to_string()), TiValue::Bool(true)];
        assert_eq!(
            TiseApp::majority_primitive_label(&arr),
            Some(statics::EN_TYPE_STRING)
        );
    }

    #[test]
    fn default_new_list_item_uses_majority_type_zero_value() {
        // Majority I64 -> new item is I64(0).
        let arr = vec![
            TiValue::Number(TiNumber::I64(7)),
            TiValue::Number(TiNumber::I64(8)),
            TiValue::String("x".to_string()),
        ];
        assert_eq!(
            TiseApp::default_new_list_item(&arr),
            TiValue::Number(TiNumber::I64(0))
        );

        // Majority String -> new item is empty string.
        let arr = vec![
            TiValue::String("a".to_string()),
            TiValue::String("b".to_string()),
            TiValue::Bool(true),
        ];
        assert_eq!(
            TiseApp::default_new_list_item(&arr),
            TiValue::String(String::new())
        );

        // Empty list -> null fallback.
        assert_eq!(TiseApp::default_new_list_item(&[]), TiValue::Null);
    }
}
