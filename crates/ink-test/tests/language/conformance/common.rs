use std::fs;

use crate::conformance::api::Story;

pub fn next_all(story: &mut Story, text: &mut Vec<String>) {
    while story.can_continue() {
        let line = story.cont();
        print!("{line}");

        if !line.trim().is_empty() {
            text.push(line.trim().to_string());
        }
    }

    if !story.get_current_errors().is_empty() {
        panic!("{}", join_text(&story.get_current_errors()));
    }
}

pub fn join_text(text: &[String]) -> String {
    let mut sb = String::new();
    for s in text {
        sb.push_str(s);
    }
    sb
}

pub fn run_story(
    filename: &str,
    choice_list: Option<Vec<usize>>,
    errors: &mut Vec<String>,
) -> Vec<String> {
    let json = get_json_string(filename);
    let mut story = Story::new(&json);
    let mut text = Vec::new();
    let mut choice_list_index = 0;

    while story.can_continue() || !story.get_current_choices().is_empty() {
        println!("{}", story.build_string_of_hierarchy());

        while story.can_continue() {
            let line = story.cont();
            print!("{}", line);
            text.push(line);
        }

        if !story.get_current_errors().is_empty() {
            for error_msg in story.get_current_errors() {
                println!("{}", error_msg);
                errors.push(error_msg.to_string());
            }
        }

        let current_choices = story.get_current_choices();
        if !current_choices.is_empty() {
            for choice in &current_choices {
                println!("{}", choice.text);
                text.push(format!("{}\n", choice.text));
            }

            if let Some(choice_list) = &choice_list {
                if choice_list_index < choice_list.len() {
                    story.choose_choice_index(choice_list[choice_list_index]);
                    choice_list_index += 1;
                } else {
                    story.choose_choice_index(0);
                }
            } else {
                story.choose_choice_index(0);
            }
        }
    }

    text
}

pub fn get_json_string(filename: &str) -> String {
    let path = ink_test::fixture_root()
        .join("language")
        .join("conformance")
        .join(filename);
    fs::read_to_string(path).expect("fixture json must exist")
}
