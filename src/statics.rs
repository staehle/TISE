// Central place for UI strings and other non-localized constants.
// Keep these out of gui.rs to reduce duplication and make tweaks safer.

// External links
pub const GITHUB_URL: &str = "https://github.com/staehle/tise";

// English UI strings (EN_ prefix to make future localization easier)
pub const EN_APP_TITLE: &str = "TISE: Terra Invicta Save Editor";

pub const EN_BTN_OPEN: &str = "Open...";
pub const EN_BTN_SAVE_AS: &str = "Save As...";
pub const EN_BTN_ABOUT: &str = "About";
pub const EN_BTN_GAME_FOLDER: &str = "Game Folder...";
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

// Update-available banner (shown when a newer GitHub release exists).
pub const EN_UPDATE_PREFIX: &str = "Update available:";
pub const EN_UPDATE_DOWNLOAD: &str = "Download";
pub const EN_UPDATE_DISMISS: &str = "x";

pub const EN_HOME_HEADING: &str = "TISE: Terra Invicta Save Editor";
pub const EN_HOME_INSTRUCTIONS: &str = "Open a Terra Invicta save (.json/.gz) to begin.";

pub const EN_HEADING_GROUPS: &str = "Groups";
pub const EN_GROUP_CAT_FACTIONS_PEOPLE: &str = "Factions & People";
pub const EN_GROUP_CAT_EARTH: &str = "Earth";
pub const EN_GROUP_CAT_SPACE: &str = "Space";
pub const EN_GROUP_CAT_FACILITIES: &str = "Facilities";
pub const EN_GROUP_CAT_MISSIONS_EVENTS: &str = "Missions & Events";
pub const EN_GROUP_CAT_AI_GOALS: &str = "AI Goals";
pub const EN_GROUP_CAT_GLOBAL_SYSTEM: &str = "Global / System";
pub const EN_GROUP_CAT_OTHER: &str = "Other";
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

// Save comparison (diff) feature.
pub const EN_BTN_COMPARE: &str = "Compare...";
pub const EN_WINDOW_DIFF: &str = "Compare Saves";
pub const EN_DIFF_NO_OTHER: &str = "Pick another save to compare against the loaded one.";
pub const EN_DIFF_IDENTICAL: &str = "No differences (after ignore rules).";
pub const EN_DIFF_PREFIX_BEFORE: &str = "Before:";
pub const EN_DIFF_PREFIX_AFTER: &str = "After:";
pub const EN_DIFF_LABEL_ADDED: &str = "added";
pub const EN_DIFF_LABEL_REMOVED: &str = "removed";
pub const EN_DIFF_LABEL_CHANGED: &str = "changed";
pub const EN_DIFF_GROUPS_ADDED: &str = "Groups added:";
pub const EN_DIFF_GROUPS_REMOVED: &str = "Groups removed:";
pub const EN_DIFF_KIND_ADDED: &str = "+";
pub const EN_DIFF_KIND_REMOVED: &str = "-";
pub const EN_DIFF_KIND_CHANGED: &str = "~";
pub const EN_DIFF_IGNORE_HEADER: &str = "Filters";
pub const EN_DIFF_IGNORE_NOISE_PRESET: &str = "Hide per-turn noise (history/event logs)";
pub const EN_DIFF_SEARCH_LABEL: &str = "Search:";
pub const EN_DIFF_SEARCH_HINT: &str = "object, id, field, value...";
pub const EN_DIFF_SHOW_ALL_GROUPS: &str = "Show all changed groups";
pub const EN_DIFF_VISIBLE_GROUPS_LABEL: &str = "Visible groups:";
pub const EN_DIFF_BTN_IGNORE_ALL_GROUPS: &str = "Ignore all";
pub const EN_DIFF_BTN_SHOW_ALL_GROUPS: &str = "Show all";
pub const EN_DIFF_NO_MATCHES: &str = "No differences match the current filters.";
pub const EN_DIFF_ERR_NO_SAVE: &str = "Load a save first, then Compare against another.";

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
pub const TI_PROP_TRAIT_TEMPLATE_NAMES: &str = "traitTemplateNames";
pub const TI_PROP_FINISHED_ONE_TIME_PROJECTS: &str = "finishedOneTimeOnlyProjectNames";
pub const TI_PROP_AVAILABLE_PROJECTS: &str = "availableProjectNames";
pub const TI_PROP_FINISHED_TECHS: &str = "finishedTechsNames";
pub const TI_PROP_FINISHED_PROJECTS: &str = "finishedProjectNames";
pub const TI_PROP_ORG_SLOT_UNLOCKED: &str = "orgProjectSlotUnlocked";
pub const TI_PROP_HAB_SLOT_UNLOCKED: &str = "habProjectSlotUnlocked";
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
pub const TI_GROUP_HAB_STATE: &str = "PavonisInteractive.TerraInvicta.TIHabState";
pub const TI_GROUP_HAB_MODULE_STATE: &str = "PavonisInteractive.TerraInvicta.TIHabModuleState";
pub const TI_GROUP_ORG_STATE: &str = "PavonisInteractive.TerraInvicta.TIOrgState";
pub const TI_GROUP_SECTOR_STATE: &str = "PavonisInteractive.TerraInvicta.TISectorState";
pub const TI_GROUP_FACTION_STATE: &str = "PavonisInteractive.TerraInvicta.TIFactionState";

// Hab module / sector properties used for faction filtering.
pub const TI_PROP_SECTOR: &str = "sector";
pub const TI_PROP_FACTION: &str = "faction";
pub const TI_PROP_FACTION_ORBIT: &str = "factionOrbit";

// Org properties.
pub const TI_PROP_TIER: &str = "tier";

// Faction property listing orgs currently available/owned by the faction.
pub const TI_PROP_AVAILABLE_ORGS: &str = "availableOrgs";

// Hab module property for construction state.
pub const TI_PROP_CONSTRUCTION_COMPLETED: &str = "constructionCompleted";

// Tech progress entries (TIGlobalResearchState.techProgress).
pub const TI_FIELD_TECH_TEMPLATE_NAME: &str = "techTemplateName";
pub const TI_FIELD_ACCUMULATED_RESEARCH: &str = "accumulatedResearch";
pub const TI_FIELD_FACTION_CONTRIBUTIONS: &str = "factionContributions";

// Tech progress editor UI strings.
pub const EN_TECH_PROGRESS_EDITOR: &str = "Tech progress editor";
pub const EN_LABEL_ACCUMULATED_RESEARCH: &str = "Accumulated research:";
pub const EN_LABEL_FACTION_CONTRIBUTIONS: &str = "Faction contributions";

// Faction project progress entries (TIFactionState.currentProjectProgress).
pub const TI_PROP_CURRENT_PROJECT_PROGRESS: &str = "currentProjectProgress";
pub const TI_FIELD_PROJECT_TEMPLATE_NAME: &str = "projectTemplateName";
pub const TI_FIELD_SLOT: &str = "slot";
pub const TI_FIELD_COMPLETED: &str = "completed";

// Project progress editor UI strings.
pub const EN_PROJECT_PROGRESS_EDITOR: &str = "Project progress editor";
pub const EN_LABEL_SLOT: &str = "Slot:";
pub const EN_LABEL_RESEARCH_COST: &str = "Research cost:";
pub const EN_PROJECT_COST_NOT_RESEARCHABLE: &str = "Research cost: not researchable";
pub const EN_PROJECT_COST_UNKNOWN: &str = "Research cost: unknown";
pub const EN_BTN_REMOVE: &str = "Remove";
pub const EN_PROJECT_PROGRESS_EMPTY: &str = "No projects in progress.";
pub const EN_PROJECT_PROGRESS_ADD: &str = "Add project in progress";
pub const EN_PROJECT_PROGRESS_SEARCH_HINT: &str = "Search projects (or type a name)...";
pub const EN_PROJECT_PROGRESS_ADD_BTN: &str = "Add";
pub const EN_PROJECT_WARN_BAD_SLOT: &str =
    "Slot is below 3: the game ignores this entry and never shows it.";
pub const EN_PROJECT_WARN_FINISHED: &str =
    "Project is already finished: the game ignores this entry.";
pub const EN_PROJECT_WARN_FINISHED_TYPED: &str =
    "This project is already finished; the game would ignore a new entry.";
pub const EN_PROJECT_WARN_NO_SLOT: &str = "No free project slot is known for this faction (slots 3, 4 if org slot unlocked, 5 if hab slot unlocked, then 6+). Remove a project first.";

// Faction filter UI strings.
pub const EN_LABEL_FILTER_FACTION: &str = "Faction:";
pub const EN_FILTER_ALL_FACTIONS: &str = "All Factions";
pub const EN_FILTER_NO_FACTION: &str = "None";

// Tier filter UI strings.
pub const EN_LABEL_FILTER_TIER: &str = "Tier:";
pub const EN_FILTER_ALL_TIERS: &str = "All";

// Construction filter UI strings.
pub const EN_LABEL_FILTER_CONSTRUCTION: &str = "Construction:";
pub const EN_FILTER_CONSTRUCTION_ALL: &str = "All";
pub const EN_FILTER_CONSTRUCTION_COMPLETED: &str = "Completed";
pub const EN_FILTER_CONSTRUCTION_IN_PROGRESS: &str = "In Progress";

// Bulk "add selected orgs to faction" UI strings.
pub const EN_LABEL_ADD_TO_FACTION: &str = "Add to Faction:";
pub const EN_FILTER_SELECT_FACTION: &str = "Select Faction...";
pub const EN_BTN_ADD_TO_FACTION: &str = "Add Selected to Faction";
pub const EN_LABEL_ORGS_SELECTED: &str = "selected";

// Game folder (base-game data lookup).
pub const EN_WINDOW_GAME_FOLDER: &str = "Terra Invicta Game Folder";
pub const EN_GAME_FOLDER_INTRO: &str = "Select your Terra Invicta install folder so TISE can look up valid template names (traits, etc.) from the base game and enabled mods. This is remembered for next time and is optional.";
pub const EN_GAME_FOLDER_HINT: &str =
    r"e.g. C:\Program Files (x86)\Steam\steamapps\common\Terra Invicta";
pub const EN_GAME_FOLDER_BROWSE: &str = "Browse...";
pub const EN_GAME_FOLDER_DETECT: &str = "Auto-detect";
pub const EN_GAME_FOLDER_USE: &str = "Use this folder";
pub const EN_GAME_FOLDER_SKIP: &str = "Skip";
pub const EN_GAME_FOLDER_CLEAR: &str = "Clear";
pub const EN_GAME_FOLDER_CANCEL: &str = "Cancel";
pub const EN_GAME_FOLDER_DETECT_FAILED: &str =
    "Could not find a Terra Invicta install automatically.";
pub const EN_GAME_FOLDER_INVALID: &str = "That folder doesn't look like a Terra Invicta install (TITraitTemplate.json not found under it).";
pub const EN_GAME_FOLDER_LOADED: &str = "Game data loaded from:";
pub const EN_GAME_FOLDER_LOADING: &str = "Loading game data...";

// Councilor traits helper.
pub const EN_TRAIT_EDITOR: &str = "Councilor traits";
pub const EN_TRAIT_EDITOR_REMOVE_HINT: &str = "Click a trait to remove it.";
pub const EN_TRAIT_CURRENT: &str = "Current traits";
pub const EN_TRAIT_ADD: &str = "Add trait";
pub const EN_TRAIT_SEARCH_HINT: &str = "Search traits to add...";
pub const EN_TRAIT_NO_GAME_DATA: &str =
    "Set the game folder (\"Game Folder...\" in the top bar) to pick from the full trait list.";
pub const EN_TRAIT_UNKNOWN: &str = "Not found in game data";
pub const EN_TRAIT_DUPLICATE: &str = "Duplicate";
pub const EN_TRAIT_ADD_TYPED: &str = "Add as typed:";

// Finished one-time-only projects helper.
pub const EN_PROJECT_EDITOR: &str = "Finished one-time projects";
pub const EN_AVAILABLE_PROJECT_EDITOR: &str = "Available projects";
pub const EN_FINISHED_PROJECT_EDITOR: &str = "Finished projects";
/// Virtual group (not in the save): one object per faction, showing its finished projects.
pub const GROUP_FINISHED_PROJECTS: &str = "Finished Projects";
/// Virtual group (not in the save): the `factionEffectsNames` property of TIEffectsState.
pub const GROUP_FACTION_EFFECTS: &str = "Faction Effects";
pub const TI_GROUP_EFFECTS_STATE: &str = "TIEffectsState";
pub const TI_PROP_FACTION_EFFECTS_NAMES: &str = "factionEffectsNames";
pub const EN_FACTION_EFFECTS_MISSING: &str = "This faction has no entry in factionEffectsNames.";
pub const EN_FACTION_EFFECTS_NOT_LIST: &str =
    "Not a list of effect names; edit it via TIEffectsState.";
pub const EN_FACTION_EFFECTS_HINT: &str =
    "Grouped by context (right-click a context header to remove it). Click an effect to remove it.";
pub const EN_FACTION_EFFECTS_NO_CATALOG: &str =
    "Set the game folder to validate against TIEffectTemplate.";
pub const EN_FACTION_EFFECTS_UNKNOWN: &str = "Not found in TIEffectTemplate.";
pub const EN_FACTION_EFFECTS_WRONG_CONTEXT: &str =
    "TIEffectTemplate does not list this context for this effect.";
pub const EN_FACTION_EFFECTS_ADD: &str = "Add\u{2026}";
pub const EN_FACTION_EFFECTS_REMOVE_CONTEXT: &str = "Remove context";
pub const EN_FACTION_EFFECTS_NEW_CONTEXT: &str = "New context:";
pub const EN_FINISHED_PROJECTS_MISSING: &str =
    "This faction has no editable finishedProjectNames list.";
pub const EN_PROJECT_CURRENT: &str = "Current projects";
pub const EN_PROJECT_EDITOR_REMOVE_HINT: &str = "Click a project to remove it.";
pub const EN_PROJECT_ADD: &str = "Add project";
pub const EN_PROJECT_SEARCH_HINT: &str = "Search projects to add...";
pub const EN_BTN_REVERT: &str = "Revert";
pub const EN_PROJECT_UNAPPLIED: &str = "Unapplied changes";
pub const EN_PROJECT_REPEATABLE: &str = "Repeatable projects completed";
pub const EN_PROJECT_REPEATABLE_HINT: &str =
    "How many times each repeatable project has been completed.";
pub const EN_PROJECT_REPEATABLE_EMPTY: &str = "No repeatable projects completed yet.";
pub const EN_PROJECT_NO_GAME_DATA: &str =
    "Set the game folder (\"Game Folder...\" in the top bar) to pick from the full project list.";

// Finished techs helper.
pub const EN_TECH_EDITOR: &str = "Finished techs";
pub const EN_TECH_CURRENT: &str = "Current techs";
pub const EN_TECH_EDITOR_REMOVE_HINT: &str = "Click a tech to remove it.";
pub const EN_TECH_ADD: &str = "Add tech";
pub const EN_TECH_SEARCH_HINT: &str = "Search techs to add...";
pub const EN_TECH_NO_GAME_DATA: &str =
    "Set the game folder (\"Game Folder...\" in the top bar) to pick from the full tech list.";
