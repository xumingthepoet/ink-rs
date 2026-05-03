//! [`Story`] is the entry point to load and run an Ink story.
use crate::{
    container::Container,
    story::{
        errors::ErrorHandler, external_functions::ExternalFunctionDef,
        internal_functions::InternalFunctionDef,
    },
    story_state::StoryState,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// The current version of the Ink story file format.
pub const INK_VERSION_CURRENT: i32 = ink_story_json_format::INK_VERSION_CURRENT;

#[derive(PartialEq)]
pub(crate) enum OutputStateChange {
    NoChange,
    ExtendedBeyondNewline,
    NewlineRemoved,
}

/// A `Story` is the core struct representing a complete Ink narrative,
/// managing evaluation and state.
pub struct Story {
    main_content_container: Rc<Container>,
    state: StoryState,
    temporary_evaluation_container: Option<Rc<Container>>,
    recursive_continue_count: usize,
    async_continue_active: bool,
    async_saving: bool,
    prev_containers: Vec<Rc<Container>>,
    pub(crate) on_error: Option<Rc<RefCell<dyn ErrorHandler>>>,
    pub(crate) state_snapshot_at_last_new_line: Option<StoryState>,
    pub(crate) has_validated_externals: bool,
    pub(crate) allow_external_function_fallbacks: bool,
    pub(crate) saw_lookahead_unsafe_function_after_new_line: bool,
    pub(crate) externals: HashMap<String, ExternalFunctionDef>,
    pub(crate) internal_functions: HashMap<String, InternalFunctionDef>,
}

struct CSharpRandom {
    seed_array: [i32; 56],
    inext: usize,
    inextp: usize,
}

impl CSharpRandom {
    const MBIG: i32 = i32::MAX;
    const MSEED: i32 = 161_803_398;

    fn new(seed: i32) -> Self {
        // Matches legacy System.Random(seed), which ink JSON relies on for deterministic output.
        let subtraction = if seed == i32::MIN {
            i32::MAX
        } else {
            seed.abs()
        };
        let mut seed_array = [0_i32; 56];
        let mut mj = Self::MSEED.wrapping_sub(subtraction);
        seed_array[55] = mj;
        let mut mk = 1;

        for i in 1..55 {
            let ii = (21 * i) % 55;
            seed_array[ii] = mk;
            mk = mj.wrapping_sub(mk);
            if mk < 0 {
                mk = mk.wrapping_add(Self::MBIG);
            }
            mj = seed_array[ii];
        }

        for _ in 1..5 {
            for i in 1..56 {
                seed_array[i] = seed_array[i].wrapping_sub(seed_array[1 + (i + 30) % 55]);
                if seed_array[i] < 0 {
                    seed_array[i] = seed_array[i].wrapping_add(Self::MBIG);
                }
            }
        }

        Self {
            seed_array,
            inext: 0,
            inextp: 21,
        }
    }

    fn next(&mut self) -> i32 {
        self.inext += 1;
        if self.inext >= 56 {
            self.inext = 1;
        }
        self.inextp += 1;
        if self.inextp >= 56 {
            self.inextp = 1;
        }

        let mut result = self.seed_array[self.inext].wrapping_sub(self.seed_array[self.inextp]);
        if result == Self::MBIG {
            result -= 1;
        }
        if result < 0 {
            result = result.wrapping_add(Self::MBIG);
        }

        self.seed_array[self.inext] = result;
        result
    }
}

pub(crate) fn csharp_random_next(seed: i32) -> i32 {
    CSharpRandom::new(seed).next()
}

mod misc {
    use crate::{
        json::json_read,
        object::{Object, RTObject},
        path::Path,
        story::Story,
        story_error::StoryError,
        story_state::StoryState,
        value::Value,
    };
    use std::{collections::HashMap, rc::Rc};

    impl Story {
        /// Construct a `Story` out of a JSON string that was compiled with
        /// `inklecate`.
        pub fn new(json_string: &str) -> Result<Self, StoryError> {
            let loaded_program = json_read::load_program_from_string(json_string)?;
            let main_content_container = loaded_program.main_content_container;
            let internal_functions = super::internal_functions::load_internal_function_defs(
                loaded_program.internal_functions,
            )?;

            let mut story = Story {
                main_content_container: main_content_container.clone(),
                state: StoryState::new(main_content_container.clone()),
                temporary_evaluation_container: None,
                recursive_continue_count: 0,
                async_continue_active: false,
                async_saving: false,
                saw_lookahead_unsafe_function_after_new_line: false,
                state_snapshot_at_last_new_line: None,
                on_error: None,
                prev_containers: Vec::new(),
                has_validated_externals: false,
                allow_external_function_fallbacks: false,
                externals: HashMap::with_capacity(0),
                internal_functions,
            };

            story.reset_globals()?;

            Ok(story)
        }

        /// Creates a string representing the hierarchy of objects and
        /// containers in a story.
        pub fn build_string_of_hierarchy(&self) -> String {
            let mut sb = String::new();

            let cp = self.get_state().get_current_pointer().resolve();

            let cp = cp.as_ref().map(|cp| cp.as_ref());

            self.main_content_container
                .build_string_of_hierarchy(&mut sb, 0, cp);

            sb
        }

        pub(crate) fn is_truthy(&self, obj: Rc<dyn RTObject>) -> Result<bool, StoryError> {
            let truthy = false;

            if let Some(val) = obj.as_ref().as_any().downcast_ref::<Value>() {
                if let Some(target_path) = Value::get_value::<&Path>(obj.as_ref()) {
                    return Err(StoryError::InvalidStoryState(format!("Shouldn't use a divert target (to {}) as a conditional value. Did you intend a function call 'likeThis()' or a read count check 'likeThis'? (no arrows)", target_path)));
                }

                return val.is_truthy();
            }

            Ok(truthy)
        }

        pub(crate) fn next_sequence_shuffle_index(&mut self) -> Result<i32, StoryError> {
            let pop_evaluation_stack = self.get_state_mut().pop_evaluation_stack();
            let num_elements =
                if let Some(v) = Value::get_value::<i32>(pop_evaluation_stack.as_ref()) {
                    v
                } else {
                    return Err(StoryError::InvalidStoryState(
                        "Expected number of elements in sequence for shuffle index".to_owned(),
                    ));
                };

            let seq_container = self.get_state().get_current_pointer().container.unwrap();

            let seq_count = if let Some(v) =
                Value::get_value::<i32>(self.get_state_mut().pop_evaluation_stack().as_ref())
            {
                v
            } else {
                return Err(StoryError::InvalidStoryState(
                    "Expected sequence count value for shuffle index".to_owned(),
                ));
            };

            let loop_index = seq_count / num_elements;
            let iteration_index = seq_count % num_elements;

            // Generate the same shuffle based on:
            // - The hash of this container, to make sure it's consistent each time the
            //   runtime returns to the sequence
            // - How many times the runtime has looped around this full shuffle
            let seq_path_str = Object::get_path(seq_container.as_ref()).to_string();
            let sequence_hash: i32 = seq_path_str.chars().map(|c| c as i32).sum();
            let random_seed = sequence_hash
                .wrapping_add(loop_index)
                .wrapping_add(self.get_state().story_seed);

            let mut random = super::CSharpRandom::new(random_seed);
            let mut unpicked_indices: Vec<i32> = (0..num_elements).collect();

            for i in 0..=iteration_index {
                let next_random = random.next();
                let chosen = next_random % unpicked_indices.len() as i32;
                let chosen_index = unpicked_indices.remove(chosen as usize);

                if i == iteration_index {
                    return Ok(chosen_index);
                }
            }

            Err(StoryError::InvalidStoryState(
                "Should never reach here".to_owned(),
            ))
        }
    }
}

mod choices;
mod control_logic;
pub mod errors;
pub mod external_functions;
mod flow;
mod internal_functions;
mod navigation;
mod progress;
mod state;
mod tags;
