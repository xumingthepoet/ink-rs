use ink_runtime::story::Story as RuntimeStory;

#[test]
fn minimal_runtime_json_story_loads() {
    let json = r#"{"inkVersion":21,"root":["done",null],"listDefs":{}}"#;

    RuntimeStory::new(json).expect("expected minimal runtime JSON to load");
}

#[test]
fn runtime_json_requires_list_defs() {
    let json = r#"{"inkVersion":21,"root":["done",null]}"#;

    assert!(RuntimeStory::new(json).is_err());
}
