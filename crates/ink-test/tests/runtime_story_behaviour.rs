use ink_runtime::story::Story as RuntimeStory;
use serde_json::json;

fn load_story(value: serde_json::Value) -> RuntimeStory {
    RuntimeStory::new(&value.to_string()).expect("expected runtime story to load")
}

#[test]
fn choice_fixture_generates_a_choice_and_can_be_selected() {
    let story_json = r##"{"inkVersion":21,"root":[["^Hello world!","\n","ev","str","^Hello back!","/str","/ev",{"*":"0.c-0","flg":20},{"c-0":["\n","done",{"->":"0.g-0"},{"#f":5}],"g-0":["done",null]}],"done",null],"listDefs":{}}"##;
    let mut story = RuntimeStory::new(story_json).expect("choice fixture should load");

    let text = story
        .cont()
        .expect("expected story to continue to a choice");
    assert!(text.contains("Hello world!"));
    assert_eq!(story.get_current_choices().len(), 1);

    story
        .choose_choice_index(0)
        .expect("expected the first choice to be selectable");
    story
        .continue_maximally()
        .expect("expected chosen branch to continue without errors");
}

#[test]
fn named_container_paths_can_enter_knot_stitch_and_gather_targets() {
    let story_json = json!({
        "inkVersion": 21,
        "root": [
            [
                "^Knot body",
                "\n",
                [
                    "^Stitch body",
                    "\n",
                    "done",
                    {"#n": "inner"}
                ],
                "done",
                {"#n": "start"}
            ],
            "done",
            null
        ],
        "listDefs": {}
    });

    let mut story = load_story(story_json);
    story
        .choose_path_string("start.inner", true, None)
        .expect("expected nested stitch path to resolve");
    let output = story
        .continue_maximally()
        .expect("expected nested stitch story to run");
    assert!(output.contains("Stitch body"));

    let gather_json = json!({
        "inkVersion": 21,
        "root": [
            [
                "^Gather body",
                "\n",
                "done",
                {"#n": "g-0"}
            ],
            "done",
            null
        ],
        "listDefs": {}
    });

    let mut gather_story = load_story(gather_json);
    gather_story
        .choose_path_string("g-0", true, None)
        .expect("expected gather-like path to resolve");
    let output = gather_story
        .continue_maximally()
        .expect("expected gather story to run");
    assert!(output.contains("Gather body"));
}

#[test]
fn explicit_divert_reaches_named_target_container() {
    let story_json = json!({
        "inkVersion": 21,
        "root": [
            [
                "^Before divert",
                "\n",
                {"->": "0.g-0"},
                [
                    "^After divert",
                    "\n",
                    "done",
                    {"#n": "g-0"}
                ],
                null
            ],
            "done",
            null
        ],
        "listDefs": {}
    });

    let mut story = load_story(story_json);
    let output = story
        .continue_maximally()
        .expect("expected divert story to run");
    assert!(output.contains("Before divert"));
    assert!(output.contains("After divert"));
}
