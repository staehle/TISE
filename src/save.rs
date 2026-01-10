use crate::{TiValue, statics};
use anyhow::Context;
use flate2::{Compression, GzBuilder, read::GzDecoder};
use indexmap::IndexMap;
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

const COMMON_NAMESPACE: &str = "PavonisInteractive.TerraInvicta.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFormat {
    Json5,
    GzipJson5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
}

#[derive(Debug, Clone)]
pub struct ObjectSummary {
    pub id: i64,
    pub display_name: String,
    pub index_in_group: usize,
}

/// An index of the save file to allow O(1) lookups of objects by ID or group.
/// Built once upon loading or modifying the save structure.
#[derive(Debug, Clone)]
pub struct SaveIndex {
    pub groups: Vec<String>,
    pub objects_by_group: HashMap<String, Vec<ObjectSummary>>,
    pub id_lookup: HashMap<i64, (String, usize)>,
    pub id_to_display_name: HashMap<i64, String>,
}

impl SaveIndex {
    pub fn empty() -> Self {
        Self {
            groups: Vec::new(),
            objects_by_group: HashMap::new(),
            id_lookup: HashMap::new(),
            id_to_display_name: HashMap::new(),
        }
    }
}

/// Represents a loaded save file, preserving its original bytes to ensure
/// byte-for-byte roundtripping if unmodified.
#[derive(Debug, Clone)]
pub struct LoadedSave {
    pub source_path: Option<PathBuf>,
    pub format: SaveFormat,
    pub line_ending: LineEnding,
    pub original_bytes: Vec<u8>,
    pub root: TiValue,
    pub dirty: bool,
    pub index: SaveIndex,
}

impl LoadedSave {
    pub fn load_path(path: &Path) -> anyhow::Result<Self> {
        let bytes = fs::read(path).with_context(|| format!("reading {path:?}"))?;
        let format = detect_format(path, &bytes);
        let text_bytes = match format {
            SaveFormat::Json5 => bytes.clone(),
            SaveFormat::GzipJson5 => {
                let mut decoder = GzDecoder::new(&bytes[..]);
                let mut out = Vec::new();
                decoder.read_to_end(&mut out).context("gzip decompress")?;
                out
            }
        };

        let line_ending = detect_line_ending(&text_bytes);

        let text = std::str::from_utf8(&text_bytes).context("save file is not valid UTF-8")?;
        let root = TiValue::parse_json5(text).context("parsing JSON5")?;

        let mut save = Self {
            source_path: Some(path.to_path_buf()),
            format,
            line_ending,
            original_bytes: bytes,
            root,
            dirty: false,
            index: SaveIndex::empty(),
        };
        save.rebuild_index();
        Ok(save)
    }

    pub fn rebuild_index(&mut self) {
        self.index = build_index(&self.root);
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Recompute `dirty` by comparing the current serialized bytes to `original_bytes`.
    /// This is used by UI features like Undo/Redo so "dirty" can clear when changes are undone.
    pub fn refresh_dirty(&mut self) {
        // If the format changes, we consider the save dirty.
        let Ok(current) = self.generate_bytes_for_format(self.format) else {
            self.dirty = true;
            return;
        };
        self.dirty = current != self.original_bytes;
    }

    /// Generate bytes for a format regardless of current `dirty` state.
    pub fn generate_bytes_for_format(&self, format: SaveFormat) -> anyhow::Result<Vec<u8>> {
        let newline = match self.line_ending {
            LineEnding::Lf => statics::NL_LF,
            LineEnding::CrLf => statics::NL_CRLF,
        };
        let text = self.root.to_ti_save_pretty_with_newline(newline);
        let text_bytes = text.as_bytes();

        match format {
            SaveFormat::Json5 => Ok(text_bytes.to_vec()),
            SaveFormat::GzipJson5 => {
                let mut encoder = GzBuilder::new()
                    .mtime(0)
                    .write(Vec::new(), Compression::default());
                encoder.write_all(text_bytes).context("gzip compress")?;
                let bytes = encoder.finish().context("gzip finish")?;
                Ok(bytes)
            }
        }
    }

    pub fn group_display_name(group: &str) -> &str {
        group.strip_prefix(COMMON_NAMESPACE).unwrap_or(group)
    }

    pub fn game_id(&self) -> Option<i64> {
        self.root
            .get(statics::TI_PROP_CURRENT_ID)
            .and_then(|v| v.as_object())
            .and_then(|o| o.get(statics::TI_REF_FIELD_VALUE))
            .and_then(|v| match v {
                crate::TiValue::Number(n) => n.as_i64(),
                _ => None,
            })
    }

    pub fn get_object_value_mut(
        &mut self,
        group: &str,
        object_id: i64,
    ) -> Option<&mut IndexMap<String, TiValue>> {
        let (real_group, idx) = self.index.id_lookup.get(&object_id)?.clone();
        if real_group != group {
            return None;
        }
        let gamestates = self.root.get_mut(statics::TI_GAMESTATES)?.as_object_mut()?;
        let group_list = gamestates.get_mut(group)?.as_array_mut()?;
        let entry = group_list.get_mut(idx)?.as_object_mut()?;
        let value = entry
            .get_mut(statics::TI_FIELD_VALUE_CAP)?
            .as_object_mut()?;
        Some(value)
    }

    pub fn get_object_value(
        &self,
        group: &str,
        object_id: i64,
    ) -> Option<&IndexMap<String, TiValue>> {
        let (real_group, idx) = self.index.id_lookup.get(&object_id)?.clone();
        if real_group != group {
            return None;
        }
        let gamestates = self.root.get(statics::TI_GAMESTATES)?.as_object()?;
        let group_list = gamestates.get(group)?.as_array()?;
        let entry = group_list.get(idx)?.as_object()?;
        let value = entry.get(statics::TI_FIELD_VALUE_CAP)?.as_object()?;
        Some(value)
    }

    pub fn save_to_path(&mut self, path: &Path) -> anyhow::Result<()> {
        let target_format = if path.extension().and_then(|e| e.to_str()) == Some("gz") {
            SaveFormat::GzipJson5
        } else {
            SaveFormat::Json5
        };

        let bytes = self.save_bytes_for_format(target_format)?;
        fs::write(path, &bytes).with_context(|| format!("writing {path:?}"))?;

        self.source_path = Some(path.to_path_buf());
        self.format = target_format;
        self.original_bytes = bytes;
        self.dirty = false;
        Ok(())
    }

    pub fn save_bytes_for_format(&self, format: SaveFormat) -> anyhow::Result<Vec<u8>> {
        if !self.dirty && format == self.format {
            return Ok(self.original_bytes.clone());
        }

        self.generate_bytes_for_format(format)
    }
}

fn detect_line_ending(text_bytes: &[u8]) -> LineEnding {
    // Detect by counting actual newline terminators.
    // Using "any CRLF anywhere" can mis-detect if the file contains occasional CRLF
    // sequences for reasons other than line endings (or has a few mixed lines).
    let mut lf_count = 0usize;
    let mut crlf_count = 0usize;

    for (i, b) in text_bytes.iter().enumerate() {
        if *b != b'\n' {
            continue;
        }
        if i > 0 && text_bytes[i - 1] == b'\r' {
            crlf_count += 1;
        } else {
            lf_count += 1;
        }
    }

    if crlf_count > lf_count {
        LineEnding::CrLf
    } else {
        LineEnding::Lf
    }
}

fn detect_format(path: &Path, bytes: &[u8]) -> SaveFormat {
    if path.extension().and_then(|e| e.to_str()) == Some("gz") {
        return SaveFormat::GzipJson5;
    }
    // Gzip magic: 1F 8B
    if bytes.len() >= 2 && bytes[0] == 0x1F && bytes[1] == 0x8B {
        return SaveFormat::GzipJson5;
    }
    SaveFormat::Json5
}

fn build_index(root: &TiValue) -> SaveIndex {
    let mut index = SaveIndex::empty();

    let Some(gamestates) = root.get(statics::TI_GAMESTATES).and_then(|v| v.as_object()) else {
        return index;
    };

    for (group, listitems) in gamestates.iter() {
        index.groups.push(group.clone());
        let Some(items) = listitems.as_array() else {
            continue;
        };

        let mut summaries = Vec::new();
        for (idx, item) in items.iter().enumerate() {
            let Some(item_obj) = item.as_object() else {
                continue;
            };
            let id = item_obj
                .get(statics::TI_FIELD_KEY_CAP)
                .and_then(|v| v.as_object())
                .and_then(|o| o.get(statics::TI_REF_FIELD_VALUE))
                .and_then(|v| match v {
                    crate::TiValue::Number(n) => n.as_i64(),
                    _ => None,
                });
            let Some(id) = id else {
                continue;
            };

            let value_obj = item_obj
                .get(statics::TI_FIELD_VALUE_CAP)
                .and_then(|v| v.as_object());

            let display_name = value_obj
                .and_then(|o| {
                    [
                        statics::TI_PROP_DISPLAY_NAME,
                        statics::TI_PROP_NAME,
                        statics::TI_PROP_EVENT_NAME,
                    ]
                    .into_iter()
                    .find_map(|key| {
                        o.get(key)
                            .and_then(|v| v.as_str())
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                    })
                })
                .unwrap_or(statics::EN_EMPTY)
                .to_string();

            index.id_lookup.insert(id, (group.clone(), idx));
            index.id_to_display_name.insert(id, display_name.clone());
            summaries.push(ObjectSummary {
                id,
                display_name,
                index_in_group: idx,
            });
        }

        index.objects_by_group.insert(group.clone(), summaries);
    }

    index
}

#[cfg(test)]
mod tests {
    use super::{LineEnding, detect_line_ending};
    use super::{SaveFormat, build_index, detect_format};
    use crate::{TiValue, statics};
    use indexmap::IndexMap;
    use std::path::Path;

    #[test]
    fn detect_format_uses_extension_and_magic() {
        let gz_magic = [0x1F_u8, 0x8B_u8, 0x08_u8, 0x00_u8];
        let plain = b"{ a: 1 }\n";

        assert_eq!(
            detect_format(Path::new("save.json.gz"), plain),
            SaveFormat::GzipJson5
        );
        assert_eq!(
            detect_format(Path::new("save.json"), &gz_magic),
            SaveFormat::GzipJson5
        );
        assert_eq!(
            detect_format(Path::new("save.json5"), plain),
            SaveFormat::Json5
        );
    }

    #[test]
    fn build_index_extracts_ids_and_display_names() {
        // Build a minimal TI-shaped structure:
        // { gamestates: { Group: [ { Key: {value: 1}, Value: {displayName: "X"} } ... ] } }
        let make_entry = |id: i64, props: IndexMap<String, TiValue>| {
            let mut entry = IndexMap::new();

            let mut key_ref = IndexMap::new();
            key_ref.insert(
                statics::TI_REF_FIELD_VALUE.to_string(),
                TiValue::Number(crate::value::TiNumber::I64(id)),
            );
            entry.insert(
                statics::TI_FIELD_KEY_CAP.to_string(),
                TiValue::Object(key_ref),
            );

            entry.insert(
                statics::TI_FIELD_VALUE_CAP.to_string(),
                TiValue::Object(props),
            );

            TiValue::Object(entry)
        };

        let mut props1 = IndexMap::new();
        props1.insert(
            statics::TI_PROP_DISPLAY_NAME.to_string(),
            TiValue::String("X".to_string()),
        );

        let mut props2 = IndexMap::new();
        props2.insert(
            statics::TI_PROP_DISPLAY_NAME.to_string(),
            TiValue::String("".to_string()),
        );
        props2.insert(
            statics::TI_PROP_NAME.to_string(),
            TiValue::String("Name".to_string()),
        );

        let mut props3 = IndexMap::new();
        props3.insert(
            statics::TI_PROP_EVENT_NAME.to_string(),
            TiValue::String("Event".to_string()),
        );

        let props4 = IndexMap::new();

        let group = "PavonisInteractive.TerraInvicta.TITest";
        let mut gamestates = IndexMap::new();
        gamestates.insert(
            group.to_string(),
            TiValue::Array(vec![
                make_entry(1, props1),
                make_entry(2, props2),
                make_entry(3, props3),
                make_entry(4, props4),
            ]),
        );

        let mut root = IndexMap::new();
        root.insert(
            statics::TI_GAMESTATES.to_string(),
            TiValue::Object(gamestates),
        );

        let index = build_index(&TiValue::Object(root));
        assert_eq!(index.groups, vec![group.to_string()]);
        assert_eq!(index.id_lookup.get(&1).unwrap().0, group);
        assert_eq!(index.id_to_display_name.get(&1).unwrap(), "X");
        assert_eq!(index.id_to_display_name.get(&2).unwrap(), "Name");
        assert_eq!(index.id_to_display_name.get(&3).unwrap(), "Event");
        assert_eq!(index.id_to_display_name.get(&4).unwrap(), "");
    }

    #[test]
    fn detect_line_ending_uses_majority() {
        let mostly_lf = b"{\n  a: 1,\n  b: 2,\r\n  c: 3,\n}\n";
        assert_eq!(detect_line_ending(mostly_lf), LineEnding::Lf);

        let mostly_crlf = b"{\r\n  a: 1,\r\n  b: 2,\n  c: 3,\r\n}\r\n";
        assert_eq!(detect_line_ending(mostly_crlf), LineEnding::CrLf);
    }
}
