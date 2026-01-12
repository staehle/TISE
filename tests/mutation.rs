use tise::{LoadedSave, SaveFormat, TiValue, statics};

#[test]
fn modifying_a_value_marks_dirty_and_changes_bytes() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("PrunedGame.json");
    let mut save = LoadedSave::load_path(&path).expect("load example");
    assert_eq!(save.format, SaveFormat::Json5);
    assert!(!save.dirty);

    let group = save.index.groups.first().expect("has groups").to_string();
    let obj = save
        .index
        .objects_by_group
        .get(&group)
        .and_then(|v| v.first())
        .expect("has objects")
        .clone();

    let value = save
        .get_object_value_mut(&group, obj.id)
        .expect("get object value");
    value.insert(
        statics::TI_PROP_DISPLAY_NAME.to_string(),
        TiValue::String("TISE test mutation".to_string()),
    );

    save.mark_dirty();

    let bytes = save
        .save_bytes_for_format(SaveFormat::Json5)
        .expect("save bytes");
    assert_ne!(bytes, save.original_bytes);

    let text = std::str::from_utf8(&bytes).expect("utf8");
    TiValue::parse_json5(text).expect("saved json5 parses");
}
