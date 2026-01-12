// Central place for UI strings and other non-localized constants.
// Keep these out of gui.rs to reduce duplication and make tweaks safer.

// External links
pub const GITHUB_URL: &str = "https://github.com/staehle/tise";

// English UI strings (EN_ prefix to make future localization easier)
pub const EN_APP_TITLE: &str = "TISE: Terra Invicta Save Editor";

pub const EN_BTN_OPEN: &str = "Open...";
pub const EN_BTN_SAVE_AS: &str = "Save As...";
pub const EN_BTN_ABOUT: &str = "About";
pub const EN_BTN_TOGGLE_THEME: &str = "Theme";

pub const EN_NAV_BACK: &str = "<- Back";
pub const EN_NAV_FORWARD: &str = "Forward ->";
pub const EN_NAV_GO_TO_ID: &str = "Go to ID";

pub const EN_BTN_UNDO: &str = "Undo";
pub const EN_BTN_REDO: &str = "Redo";
pub const EN_BTN_CHANGES: &str = "Changes";
pub const EN_BTN_SEARCH_REF_BROWSER: &str = "Search References";
pub const EN_BTN_SEARCH_ITEMS: &str = "Search Items";

pub const EN_WINDOW_ABOUT: &str = "About";
pub const EN_WINDOW_GO_TO_ID: &str = "Go to ID";
pub const EN_WINDOW_CHANGES: &str = "Changes";
pub const EN_WINDOW_SEARCH_REF_BROWSER: &str = "Search References";
pub const EN_WINDOW_SEARCH_ITEMS: &str = "Search Items";

pub const EN_ABOUT_HEADING: &str = "TISE: Terra Invicta Save Editor";
pub const EN_ABOUT_VERSION: &str = "Version:";
pub const EN_ABOUT_SHORTCUTS: &str = "Shortcuts:";
pub const EN_ABOUT_SHORTCUT_ALT: &str = "- Alt+Left / Alt+Right: Back/Forward";
pub const EN_ABOUT_SHORTCUT_MOUSE: &str = "- Mouse back/forward buttons also work";
pub const EN_PROJECT_REPO: &str = "GitHub Repo";

pub const EN_HOME_HEADING: &str = "TISE: Terra Invicta Save Editor";
pub const EN_HOME_INSTRUCTIONS: &str = "Open a Terra Invicta save (.json/.gz) to begin.";

pub const EN_HEADING_GROUPS: &str = "Groups";
pub const EN_HEADING_OBJECTS: &str = "Objects";
pub const EN_HEADING_PROPERTIES: &str = "Properties";
pub const EN_HEADING_EDIT: &str = "Edit";

pub const EN_LABEL_SEARCH: &str = "Search:";
pub const EN_HINT_SEARCH: &str = "ID or name";
pub const EN_HINT_SEARCH_ITEMS: &str = "key or value";
pub const EN_SEARCH_ENTER_QUERY: &str = "Enter a search query.";
pub const EN_SEARCH_NO_MATCHES: &str = "No matches.";

// Small glyphs used in tables/headers.
pub const EN_GLYPH_SORT_ASC: &str = "^";
pub const EN_GLYPH_SORT_DESC: &str = "v";

pub const EN_COL_KEY: &str = "Key";
pub const EN_COL_VALUE: &str = "Value";
pub const EN_COL_PROPERTY: &str = "Property";
pub const EN_COL_VALUE_REF: &str = "Value / Ref";
pub const EN_COL_TYPE: &str = "Type";
pub const EN_COL_REF: &str = "Ref";
pub const EN_COL_ID: &str = "ID";
pub const EN_COL_NAME: &str = "Name";
pub const EN_COL_GROUP: &str = "Group";

pub const EN_LABEL_SORT: &str = "Sort:";
pub const EN_SORT_NAME: &str = "Name";
pub const EN_SORT_ID: &str = "ID";

pub const EN_SELECT_GROUP: &str = "Select a group.";
pub const EN_SELECT_GROUP_LEFT: &str = "Select a group from the left.";
pub const EN_SELECT_OBJECT: &str = "Select an object.";
pub const EN_SELECT_PROPERTY: &str = "Select a property to edit.";

pub const EN_BTN_GO: &str = "Go";
pub const EN_BTN_CANCEL: &str = "Cancel";

pub const EN_BTN_APPLY_PROPERTY: &str = "Apply Property";
pub const EN_BTN_SET_NULL: &str = "Set null";
pub const EN_BTN_GO_TO_REF: &str = "Go to Ref";
pub const EN_BTN_CHANGE_TYPE: &str = "Change Type...";

pub const EN_WINDOW_CHANGE_TYPE: &str = "Change Type";

pub const EN_LABEL_REFERENCE_ID: &str = "Reference ID:";
pub const EN_LABEL_VALUE: &str = "Value";
pub const EN_PREFIX_VALUE: &str = "Value: ";
pub const EN_HINT_VALUE: &str = "Value";
pub const EN_CHECKBOX_RAW_JSON5: &str = "Raw JSON5";

pub const EN_GO_TO_ID_PROMPT: &str = "Enter an object ID number:";
pub const EN_GO_TO_ID_HINT: &str = "e.g. 4020";

pub const EN_PUBLIC_OPINION_HELPER: &str = "Public Opinion helper (auto-calculates Undecided)";
pub const EN_PUBLIC_OPINION_CHART: &str = "Pie chart";
pub const EN_PUBLIC_OPINION_CHART_HINT: &str =
    "Drag dividers to re-balance two slices, or drag a slice in/out to trade with Undecided.";
pub const EN_PUBLIC_OPINION_ERR_TOTAL_EXCEEDS: &str =
    "Total exceeds 1.0 (Undecided would be negative)";
pub const EN_BTN_APPLY_PUBLIC_OPINION: &str = "Apply Public Opinion";
pub const EN_SIMPLE_OBJECT_EDITOR: &str = "Simple object editor";
pub const EN_SIMPLE_LIST_EDITOR: &str = "Simple list editor";
pub const EN_MIXED_OBJECT_EDITOR: &str = "Mixed object editor";

pub const EN_COL_INDEX: &str = "Index";

pub const EN_BTN_ADD_ITEM: &str = "Add item";
pub const EN_BTN_DELETE: &str = "Delete";
pub const EN_BTN_INSERT: &str = "Insert";
pub const EN_BTN_UP: &str = "Up";
pub const EN_BTN_DOWN: &str = "Down";
pub const EN_BTN_APPLY: &str = "Apply";
pub const EN_BTN_RESET: &str = "Reset";
pub const EN_BTN_CLEAR: &str = "Clear";

pub const EN_LABEL_JSON5: &str = "JSON5";
pub const EN_LABEL_PREVIEW: &str = "Preview";
pub const EN_LABEL_PICK_TYPE: &str = "Pick a type:";

pub const EN_HISTORY_LABEL: &str = "history:";
pub const EN_HISTORY_BACK: &str = "<-";
pub const EN_HISTORY_FORWARD: &str = "->";

pub const EN_CHANGES_NONE: &str = "No changes.";
pub const EN_CHANGES_TIP: &str = "Tip: Undo/Redo also works with Ctrl+Z / Ctrl+Y";

pub const EN_PREFIX_UNDO: &str = "Undo:";
pub const EN_PREFIX_REDO: &str = "Redo:";
pub const EN_LABEL_CHANGES_COUNT: &str = "changes:";

pub const EN_LITERAL_MISSING: &str = "<missing>";
pub const EN_EMPTY: &str = "";

// Newline constants (used for save formatting; keep out of save/value code).
pub const NL_LF: &str = "\n";
pub const NL_CRLF: &str = "\r\n";

pub const EN_TYPE_NULL: &str = "null";
pub const EN_TYPE_BOOL: &str = "bool";
pub const EN_TYPE_I64: &str = "number (i64)";
pub const EN_TYPE_U64: &str = "number (u64)";
pub const EN_TYPE_F64: &str = "number (f64)";
pub const EN_TYPE_STRING: &str = "string";
pub const EN_TYPE_ARRAY: &str = "array";
pub const EN_TYPE_OBJECT: &str = "object";
pub const EN_TYPE_REFERENCE: &str = "reference";

pub const EN_LITERAL_NULL: &str = "null";

pub const EN_ERR_LOCATE_SELECTED_OBJECT: &str = "Could not locate selected object";
pub const EN_ERR_INVALID_ID_INTEGER: &str = "Invalid ID (must be an integer)";

pub const EN_ERR_OBJECT_VALUE_MISSING: &str = "Could not locate object value";
pub const EN_ERR_PUBLIC_OPINION_NOT_FOUND: &str = "publicOpinion not found";
pub const EN_ERR_PUBLIC_OPINION_NOT_OBJECT: &str = "publicOpinion is not an object";
pub const EN_BADGE_MODIFIED: &str = "Modified";
pub const EN_BADGE_DIRTY: &str = "dirty";
pub const EN_PLACEHOLDER_UNSAVED: &str = "<unsaved>";

// Terra Invicta save structure keys (TI_ prefix)
pub const TI_GAMESTATES: &str = "gamestates";

// Common object entry fields inside gamestates arrays.
pub const TI_FIELD_KEY_CAP: &str = "Key";
pub const TI_FIELD_VALUE_CAP: &str = "Value";

// Common relational ref object fields.
pub const TI_REF_FIELD_VALUE: &str = "value";
pub const TI_REF_FIELD_TYPE: &str = "$type";

// Common object properties.
pub const TI_PROP_PUBLIC_OPINION: &str = "publicOpinion";
pub const TI_PUBLIC_OPINION_UNDECIDED: &str = "Undecided";

// Common public-opinion outcomes (color-coded in the editor).
pub const TI_PUBLIC_OPINION_SUBMIT: &str = "Submit";
pub const TI_PUBLIC_OPINION_COOPERATE: &str = "Cooperate";
pub const TI_PUBLIC_OPINION_EXPLOIT: &str = "Exploit";
pub const TI_PUBLIC_OPINION_ESCAPE: &str = "Escape";
pub const TI_PUBLIC_OPINION_RESIST: &str = "Resist";
pub const TI_PUBLIC_OPINION_DESTROY: &str = "Destroy";

// Other known properties.
pub const TI_PROP_DISPLAY_NAME: &str = "displayName";
pub const TI_PROP_NAME: &str = "name";
pub const TI_PROP_EVENT_NAME: &str = "eventName";
pub const TI_PROP_CURRENT_ID: &str = "currentID";

// Common name-related fields (seen on councilors and other entities).
pub const TI_PROP_FAMILY_NAME: &str = "familyName";
pub const TI_PROP_PERSONAL_NAME: &str = "personalName";

// Common group names.
pub const TI_GROUP_COUNCILOR_STATE: &str = "PavonisInteractive.TerraInvicta.TICouncilorState";
