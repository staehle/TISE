use pretty_assertions::assert_eq;
use std::path::Path;
use tise::statics;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn roundtrip_unmodified_json5_bytes_identical() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("sample.json");

    let input = r#"{
  currentID: { value: 42 },
  gamestates: {
    "PavonisInteractive.TerraInvicta.TITest": [
      {
        Key: { value: 123 },
        Value: {
          ID: { value: 123 },
          $type: "PavonisInteractive.TerraInvicta.TITest",
          displayName: "Hello",
          maxStrength: Infinity,
          distance: -Infinity,
          empty: {},
          ref: { value: 999 },
        },
      },
    ],
  },
}
"#;

    std::fs::write(&path, input.as_bytes())?;

    let save = tise::LoadedSave::load_path(&path)?;
    let out_bytes = save.save_bytes_for_format(tise::SaveFormat::Json5)?;
    assert_eq!(out_bytes, input.as_bytes());
    Ok(())
}

#[test]
fn roundtrip_unmodified_gz_bytes_identical() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("sample.json.gz");

    let input = b"{ value: Infinity }\n";

    // Write a gzip with deterministic header.
    let gz = {
        use flate2::{Compression, GzBuilder};
        use std::io::Write;
        let mut encoder = GzBuilder::new()
            .mtime(0)
            .write(Vec::new(), Compression::default());
        encoder.write_all(input)?;
        encoder.finish()?
    };

    std::fs::write(&path, &gz)?;

    let save = tise::LoadedSave::load_path(&path)?;
    let out_bytes = save.save_bytes_for_format(tise::SaveFormat::GzipJson5)?;
    assert_eq!(out_bytes, gz);
    Ok(())
}

#[test]
fn roundtrip_example_pruned_game_more_identical() -> Result<()> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("PrunedGameMore.json");

    let input = std::fs::read(&path)?;
    let save = tise::LoadedSave::load_path(&path)?;
    let out = save.save_bytes_for_format(tise::SaveFormat::Json5)?;
    assert_eq!(out, input);
    Ok(())
}

#[test]
fn dirty_save_is_valid_json5() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("sample.json");

    std::fs::write(&path, b"{ a: 1 }\n")?;
    let mut save = tise::LoadedSave::load_path(&path)?;

    // Mutate then ensure we can serialize and re-parse.
    save.root = tise::TiValue::parse_json5("{ a: 2, b: Infinity }")?;
    save.mark_dirty();

    let out = save.save_bytes_for_format(tise::SaveFormat::Json5)?;
    let _parsed = tise::TiValue::parse_json5(std::str::from_utf8(&out)?)?;
    Ok(())
}

#[test]
// #[ignore = "Slow: reads/writes examples/LargeGame.json"]
fn integration_large_game_edit_councilor_3896_minimal_diff() -> Result<()> {
    let input_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("LargeGame.json");

    if !input_path.exists() {
        return Err(format!("Missing test fixture: {input_path:?}").into());
    }

    let input_bytes = std::fs::read(&input_path)?;
    let mut save = tise::LoadedSave::load_path(&input_path)?;

    let Some(props) = save.get_object_value_mut(statics::TI_GROUP_COUNCILOR_STATE, 3896) else {
        return Err("Could not locate TICouncilorState ID 3896".into());
    };

    props.insert(
        statics::TI_PROP_DISPLAY_NAME.to_string(),
        tise::TiValue::String("Bob Ross".to_string()),
    );
    props.insert(
        statics::TI_PROP_FAMILY_NAME.to_string(),
        tise::TiValue::String("Ross".to_string()),
    );
    props.insert(
        statics::TI_PROP_PERSONAL_NAME.to_string(),
        tise::TiValue::String("Bob".to_string()),
    );

    save.mark_dirty();

    let out_bytes = save.save_bytes_for_format(tise::SaveFormat::Json5)?;
    assert!(out_bytes != input_bytes, "expected modified output bytes");

    let normalize = |bytes: &[u8]| -> Result<String> {
        let text = std::str::from_utf8(bytes)?;
        // Compare line-by-line, ignoring CRLF vs LF.
        Ok(text.replace("\r\n", "\n").replace('\r', "\n"))
    };

    let in_text = normalize(&input_bytes)?;
    let out_text = normalize(&out_bytes)?;

    let in_line_count = in_text.split('\n').count();
    let out_line_count = out_text.split('\n').count();
    assert_eq!(
        in_line_count, out_line_count,
        "expected same number of lines after edit"
    );

    let mut changed_out_lines = Vec::new();
    for (a, b) in in_text.split('\n').zip(out_text.split('\n')) {
        if a != b {
            changed_out_lines.push(b.to_string());
        }
    }

    assert_eq!(
        changed_out_lines.len(),
        3,
        "expected only 3 changed lines, got {}\nFirst changes: {:?}",
        changed_out_lines.len(),
        changed_out_lines.iter().take(10).collect::<Vec<_>>()
    );

    assert!(
        changed_out_lines
            .iter()
            .any(|l| l.contains("\"displayName\": \"Bob Ross\"")),
        "missing updated displayName line"
    );
    assert!(
        changed_out_lines
            .iter()
            .any(|l| l.contains("\"familyName\": \"Ross\"")),
        "missing updated familyName line"
    );
    assert!(
        changed_out_lines
            .iter()
            .any(|l| l.contains("\"personalName\": \"Bob\"")),
        "missing updated personalName line"
    );

    Ok(())
}
