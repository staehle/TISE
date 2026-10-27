//! Base-game data lookup: remembered game folder + template name catalogue.
//!
//! The user points TISE at their Terra Invicta install once. The path is cached
//! in a small JSON config file, and the game's `Templates/*.json` files (plus
//! any enabled mods' overrides) are parsed on a background thread to build a
//! catalogue of valid `dataName` values (e.g. every councillor trait). The GUI
//! uses that catalogue to offer pickers instead of free-text template names.
//!
//! Everything here fails soft: a missing or corrupt config, an unreadable
//! template, or a moved game folder just means "no catalogue", never an error
//! the user has to deal with.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError};

/// Template files (without `.json`) loaded into the catalogue.
pub const TEMPLATE_TRAITS: &str = "TITraitTemplate";
pub const TEMPLATE_PROJECTS: &str = "TIProjectTemplate";
pub const TEMPLATE_TECHS: &str = "TITechTemplate";
/// Catalogue key for projects with `oneTimeGlobally: true` (same file as
/// [`TEMPLATE_PROJECTS`]).
pub const TEMPLATE_ONE_TIME_PROJECTS: &str = "TIProjectTemplate#oneTimeGlobally";

/// Catalogues to build: (key, template file without `.json`, required flag).
/// With a flag, only entries where that boolean field is `true` are kept (an
/// entry that later sets it false, e.g. a mod override, is dropped).
const CATALOGUES: &[(&str, &str, Option<&str>)] = &[
    (TEMPLATE_TRAITS, TEMPLATE_TRAITS, None),
    (TEMPLATE_PROJECTS, TEMPLATE_PROJECTS, None),
    (TEMPLATE_TECHS, TEMPLATE_TECHS, None),
    (
        TEMPLATE_ONE_TIME_PROJECTS,
        TEMPLATE_PROJECTS,
        Some("oneTimeGlobally"),
    ),
];

/// File whose presence marks a directory as the game's `Templates` folder.
const MARKER_FILE: &str = "TITraitTemplate.json";

/// Persisted user settings.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    /// Game install folder chosen by the user (as entered; re-resolved on load).
    pub game_dir: Option<PathBuf>,
    /// The user chose "Skip" on the first-run prompt; don't ask again.
    pub prompt_dismissed: bool,
}

impl Config {
    /// Location of the config file, or `None` if no home/config dir is known.
    pub fn path() -> Option<PathBuf> {
        let base = if cfg!(windows) {
            std::env::var_os("APPDATA").map(PathBuf::from)
        } else if cfg!(target_os = "macos") {
            home_dir().map(|h| h.join("Library/Application Support"))
        } else {
            std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .or_else(|| home_dir().map(|h| h.join(".config")))
        };
        base.map(|b| b.join("TISE").join("config.json"))
    }

    /// Load from the default location; any problem yields defaults.
    pub fn load() -> Config {
        Self::path()
            .map(|p| Self::load_from(&p))
            .unwrap_or_default()
    }

    pub fn load_from(path: &Path) -> Config {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to the default location.
    pub fn save(&self) -> std::io::Result<()> {
        match Self::path() {
            Some(p) => self.save_to(&p),
            None => Err(std::io::Error::other("no config directory available")),
        }
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from)
}

/// Resolved locations inside a game install.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GamePaths {
    pub templates: PathBuf,
    /// `<game>/Mods/Enabled`, if present.
    pub mods_enabled: Option<PathBuf>,
}

/// Accepts the game root, the `Templates` folder itself, or an intermediate
/// folder (`TerraInvicta_Data`, `StreamingAssets`, a macOS `.app` bundle) and
/// finds the `Templates` folder under it.
pub fn resolve_game_paths(chosen: &Path) -> Option<GamePaths> {
    let candidates = [
        chosen.to_path_buf(),
        chosen.join("Templates"),
        chosen.join("StreamingAssets/Templates"),
        chosen.join("TerraInvicta_Data/StreamingAssets/Templates"),
        chosen.join("Contents/Resources/Data/StreamingAssets/Templates"),
        chosen.join("TerraInvicta.app/Contents/Resources/Data/StreamingAssets/Templates"),
    ];
    let templates = candidates
        .into_iter()
        .find(|c| c.join(MARKER_FILE).is_file())?;
    let mods_enabled = templates
        .ancestors()
        .take(6)
        .map(|a| a.join("Mods").join("Enabled"))
        .find(|m| m.is_dir());
    Some(GamePaths {
        templates,
        mods_enabled,
    })
}

/// Look for a Steam install of Terra Invicta in the usual places, including
/// extra Steam library folders listed in `libraryfolders.vdf`.
pub fn detect_game_dir() -> Option<PathBuf> {
    let mut steam_roots: Vec<PathBuf> = Vec::new();
    if cfg!(windows) {
        for var in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Some(p) = std::env::var_os(var) {
                steam_roots.push(PathBuf::from(p).join("Steam"));
            }
        }
    } else if let Some(h) = home_dir() {
        steam_roots.push(h.join(".local/share/Steam"));
        steam_roots.push(h.join(".steam/steam"));
        steam_roots.push(h.join("Library/Application Support/Steam"));
    }

    let mut libraries: Vec<PathBuf> = Vec::new();
    for root in &steam_roots {
        libraries.push(root.clone());
        if let Ok(vdf) = std::fs::read_to_string(root.join("steamapps/libraryfolders.vdf")) {
            libraries.extend(parse_library_paths(&vdf));
        }
    }

    libraries
        .into_iter()
        .map(|lib| lib.join("steamapps/common/Terra Invicta"))
        .find(|p| resolve_game_paths(p).is_some())
}

/// Extract `"path"  "<dir>"` entries from a Steam `libraryfolders.vdf`.
fn parse_library_paths(vdf: &str) -> Vec<PathBuf> {
    vdf.lines()
        .filter_map(|line| {
            let mut quoted = line.split('"').skip(1).step_by(2);
            if quoted.next()? != "path" {
                return None;
            }
            Some(PathBuf::from(quoted.next()?.replace("\\\\", "\\")))
        })
        .collect()
}

/// Catalogue of valid `dataName` values per template file.
#[derive(Debug, Default)]
pub struct GameData {
    names: BTreeMap<String, Vec<String>>,
}

impl GameData {
    /// Sorted `dataName`s for a template (e.g. [`TEMPLATE_TRAITS`]).
    pub fn names(&self, template: &str) -> &[String] {
        self.names.get(template).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Parse the base templates, then merge in enabled mods' overrides/additions.
    pub fn load(paths: &GamePaths) -> GameData {
        let mut mod_dirs: Vec<PathBuf> = Vec::new();
        if let Some(mods) = &paths.mods_enabled
            && let Ok(rd) = std::fs::read_dir(mods)
        {
            mod_dirs = rd
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            mod_dirs.sort();
        }

        let mut names = BTreeMap::new();
        for (key, template, flag) in CATALOGUES {
            let file = format!("{template}.json");
            let flag = *flag;
            let mut set = BTreeSet::new();
            collect_data_names(&paths.templates.join(&file), flag, &mut set);
            for dir in &mod_dirs {
                collect_data_names(&dir.join(&file), flag, &mut set);
                collect_data_names(&dir.join("Templates").join(&file), flag, &mut set);
            }
            names.insert((*key).to_string(), set.into_iter().collect());
        }
        GameData { names }
    }
}

/// Add every `dataName` in a template file to `out`. Missing or malformed
/// files are skipped. Template files may be JSON5-ish (comments, BOM).
/// With `flag`, only entries where that field is `true` are kept; entries where
/// it is not `true` are removed from `out` (later files override earlier ones).
fn collect_data_names(path: &Path, flag: Option<&str>, out: &mut BTreeSet<String>) {
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let text = text.trim_start_matches('\u{feff}');
    let Ok(serde_json::Value::Array(entries)) = json5::from_str::<serde_json::Value>(text) else {
        return;
    };
    for entry in entries {
        if let Some(name) = entry.get("dataName").and_then(|v| v.as_str())
            && !name.is_empty()
            && name != "dummy"
        {
            let keep = flag.is_none_or(|f| entry.get(f).and_then(|v| v.as_bool()) == Some(true));
            if keep {
                out.insert(name.to_string());
            } else {
                out.remove(name);
            }
        }
    }
}

/// Background load of [`GameData`], polled each frame by the GUI.
pub struct GameDataLoad {
    rx: Option<Receiver<GameData>>,
    result: Option<Arc<GameData>>,
}

impl GameDataLoad {
    pub fn spawn(paths: GamePaths) -> GameDataLoad {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            // Receiver may be gone if the folder was changed or app closed.
            let _ = tx.send(GameData::load(&paths));
        });
        GameDataLoad {
            rx: Some(rx),
            result: None,
        }
    }

    /// Non-blocking; `None` until the load finishes.
    pub fn poll(&mut self) -> Option<Arc<GameData>> {
        if self.result.is_none()
            && let Some(rx) = &self.rx
        {
            match rx.try_recv() {
                Ok(data) => {
                    self.result = Some(Arc::new(data));
                    self.rx = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => self.rx = None,
            }
        }
        self.result.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_game(root: &Path) -> PathBuf {
        let templates = root.join("TerraInvicta_Data/StreamingAssets/Templates");
        std::fs::create_dir_all(&templates).unwrap();
        std::fs::write(
            templates.join(MARKER_FILE),
            r#"[{"dataName":"dummy"},{"dataName":"Physicist"},{"dataName":"Affluent"}]"#,
        )
        .unwrap();
        templates
    }

    #[test]
    fn config_roundtrip_and_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/config.json");
        assert_eq!(Config::load_from(&path), Config::default());

        let cfg = Config {
            game_dir: Some(PathBuf::from("X:/Games/TI")),
            prompt_dismissed: true,
        };
        cfg.save_to(&path).unwrap();
        assert_eq!(Config::load_from(&path), cfg);

        std::fs::write(&path, "not json").unwrap();
        assert_eq!(Config::load_from(&path), Config::default());
    }

    #[test]
    fn resolves_root_or_templates_folder() {
        let dir = tempfile::tempdir().unwrap();
        let templates = fake_game(dir.path());
        assert_eq!(resolve_game_paths(dir.path()).unwrap().templates, templates);
        assert_eq!(resolve_game_paths(&templates).unwrap().templates, templates);
        assert!(resolve_game_paths(&dir.path().join("nope")).is_none());
    }

    #[test]
    fn loads_names_and_merges_mods() {
        let dir = tempfile::tempdir().unwrap();
        fake_game(dir.path());
        let mod_dir = dir.path().join("Mods/Enabled/SomeMod");
        std::fs::create_dir_all(&mod_dir).unwrap();
        std::fs::write(
            mod_dir.join(MARKER_FILE),
            "\u{feff}[ // comment\n {\"dataName\":\"Affluent\"}, {\"dataName\":\"ModTrait\"}, ]",
        )
        .unwrap();

        let paths = resolve_game_paths(dir.path()).unwrap();
        assert!(paths.mods_enabled.is_some());
        let data = GameData::load(&paths);
        assert_eq!(
            data.names(TEMPLATE_TRAITS),
            ["Affluent", "ModTrait", "Physicist"]
        );
        assert!(data.names("Nonexistent").is_empty());
    }

    #[test]
    fn parses_steam_library_paths() {
        let vdf = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t\t\"label\"\t\t\"\"\n\t}\n}";
        assert_eq!(
            parse_library_paths(vdf),
            vec![PathBuf::from("D:\\SteamLibrary")]
        );
    }
}
