use pretty_assertions::assert_eq;

use std::path::Path;

use tise::statics;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn game_id_and_indexing_work() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("sample.json5");

    // Minimal, but representative, TI-shaped save.
    let input = r#"{
  currentID: { value: 4242 },
  gamestates: {
    "PavonisInteractive.TerraInvicta.TITest": [
      {
        Key: { value: 123 },
        Value: {
          displayName: "Hello",
        },
      },
      {
        Key: { value: 456 },
        Value: {
          displayName: "World",
        },
      },
    ],
  },
}
"#;

    std::fs::write(&path, input.as_bytes())?;

    let mut save = tise::LoadedSave::load_path(&path)?;
    assert_eq!(save.game_id(), Some(4242));

    // Index contains our group.
    assert!(
        save.index
            .groups
            .iter()
            .any(|g| g == "PavonisInteractive.TerraInvicta.TITest")
    );

    // Can fetch object value by ID.
    let obj_123 = save
        .get_object_value("PavonisInteractive.TerraInvicta.TITest", 123)
        .unwrap();
    assert_eq!(
        obj_123
            .get(statics::TI_PROP_DISPLAY_NAME)
            .and_then(|v| v.as_str()),
        Some("Hello")
    );

    // Mutate a field using the mutable accessor.
    {
        let obj_456 = save
            .get_object_value_mut("PavonisInteractive.TerraInvicta.TITest", 456)
            .unwrap();
        obj_456.insert(
            statics::TI_PROP_DISPLAY_NAME.to_string(),
            tise::TiValue::String("Changed".to_string()),
        );
    }
    save.mark_dirty();
    save.rebuild_index();

    let obj_456 = save
        .get_object_value("PavonisInteractive.TerraInvicta.TITest", 456)
        .unwrap();
    assert_eq!(
        obj_456
            .get(statics::TI_PROP_DISPLAY_NAME)
            .and_then(|v| v.as_str()),
        Some("Changed")
    );

    Ok(())
}

#[test]
fn load_example_resistance_builds_index() -> Result<()> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("Resistance.json");

    let save = tise::LoadedSave::load_path(&path)?;
    // Not asserting exact counts (those change with pruning/examples); just basic sanity.
    assert!(!save.index.groups.is_empty());
    assert!(!save.index.id_lookup.is_empty());
    Ok(())
}
