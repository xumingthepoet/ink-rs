use ink_experiments::discover_experiments;

#[test]
fn all_experiments_compile_without_diagnostics() {
    for experiment in discover_experiments().expect("experiment discovery should succeed") {
        experiment.compile().unwrap_or_else(|error| {
            panic!(
                "{} should compile without diagnostics:\n{}",
                error.path().display(),
                error.formatted_diagnostics()
            )
        });
    }
}

#[test]
fn stdout_snapshots_match_when_present() {
    for experiment in discover_experiments().expect("experiment discovery should succeed") {
        let expected = experiment
            .expected_stdout()
            .unwrap_or_else(|error| panic!("failed to read stdout snapshot: {error}"));
        let Some(expected) = expected else {
            continue;
        };

        let mut story = experiment.story().unwrap_or_else(|error| {
            panic!(
                "{} should load as a runtime story: {:?}",
                error.path().display(),
                error
            )
        });
        let actual = story
            .continue_maximally()
            .expect("experiment story should continue");

        assert_eq!(
            actual,
            expected,
            "{} stdout snapshot should match",
            experiment.path().display()
        );
        assert!(
            story.get_current_errors().is_empty(),
            "{} should not emit runtime errors: {:#?}",
            experiment.path().display(),
            story.get_current_errors()
        );
    }
}

#[test]
fn playthrough_snapshots_match_when_present() {
    for experiment in discover_experiments().expect("experiment discovery should succeed") {
        let steps = experiment
            .expected_playthrough()
            .unwrap_or_else(|error| panic!("failed to read playthrough snapshot: {error}"));
        let Some(steps) = steps else {
            continue;
        };

        let mut story = experiment.story().unwrap_or_else(|error| {
            panic!(
                "{} should load as a runtime story: {:?}",
                error.path().display(),
                error
            )
        });

        for (index, step) in steps.iter().enumerate() {
            let actual_output = story
                .continue_maximally()
                .expect("experiment story should continue");
            assert_eq!(
                actual_output,
                step.output,
                "{} playthrough step {index} output should match",
                experiment.path().display()
            );

            let actual_choices = story
                .get_current_choices()
                .iter()
                .map(|choice| choice.text.clone())
                .collect::<Vec<_>>();
            assert_eq!(
                actual_choices,
                step.choices,
                "{} playthrough step {index} choices should match",
                experiment.path().display()
            );

            if let Some(choice_index) = step.choose {
                story
                    .choose_choice_index(choice_index)
                    .expect("playthrough choice should be valid");
            }
        }

        assert!(
            story.get_current_errors().is_empty(),
            "{} should not emit runtime errors: {:#?}",
            experiment.path().display(),
            story.get_current_errors()
        );
    }
}
