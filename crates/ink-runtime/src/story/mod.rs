//! [`Story`] is the entry point to load and run an ink-rs story.
use crate::{
    container::Container,
    dynamic_interface::DynamicInterfaceRegistry,
    story::{
        errors::ErrorHandler, external_functions::ExternalFunctionDef,
        internal_functions::InternalFunctionDef,
    },
    story_state::StoryState,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(PartialEq)]
pub(crate) enum OutputStateChange {
    NoChange,
    ExtendedBeyondNewline,
    NewlineRemoved,
}

/// A `Story` is the core struct representing a complete ink-rs narrative,
/// managing evaluation and state.
pub struct Story {
    main_content_container: Rc<Container>,
    state: StoryState,
    temporary_evaluation_container: Option<Rc<Container>>,
    recursive_continue_count: usize,
    async_continue_active: bool,
    async_saving: bool,
    pub(crate) on_error: Option<Rc<RefCell<dyn ErrorHandler>>>,
    pub(crate) state_snapshot_at_last_new_line: Option<StoryState>,
    pub(crate) choice_save_snapshot: Option<StoryState>,
    pub(crate) has_validated_externals: bool,
    pub(crate) allow_external_function_fallbacks: bool,
    pub(crate) saw_lookahead_unsafe_function_after_new_line: bool,
    pub(crate) externals: HashMap<String, ExternalFunctionDef>,
    pub(crate) internal_functions: HashMap<String, InternalFunctionDef>,
    pub(crate) dynamic_interfaces: DynamicInterfaceRegistry,
}

struct CompiledStoryRandom {
    seed_array: [i32; 56],
    inext: usize,
    inextp: usize,
}

impl CompiledStoryRandom {
    const MBIG: i32 = i32::MAX;
    const MSEED: i32 = 161_803_398;

    fn new(seed: i32) -> Self {
        // Matches the historical System.Random(seed) sequence used by compiled story JSON.
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

pub(crate) fn compiled_story_random_next(seed: i32) -> i32 {
    CompiledStoryRandom::new(seed).next()
}

mod misc {
    use crate::{
        json::json_read, object::RTObject, path::Path, story::Story, story_error::StoryError,
        story_state::StoryState, value::Value,
    };
    use std::{collections::HashMap, rc::Rc};

    impl Story {
        /// Construct a `Story` from ink-rs compiled story JSON.
        pub fn new(json_string: &str) -> Result<Self, StoryError> {
            let loaded_program = json_read::load_program_from_string(json_string)?;
            let main_content_container = loaded_program.main_content_container;
            let internal_functions = super::internal_functions::load_internal_function_defs(
                loaded_program.internal_functions,
            )?;
            let dynamic_interfaces = loaded_program.dynamic_interfaces;

            let mut story = Story {
                main_content_container: main_content_container.clone(),
                state: StoryState::new(main_content_container.clone()),
                temporary_evaluation_container: None,
                recursive_continue_count: 0,
                async_continue_active: false,
                async_saving: false,
                saw_lookahead_unsafe_function_after_new_line: false,
                state_snapshot_at_last_new_line: None,
                choice_save_snapshot: None,
                on_error: None,
                has_validated_externals: false,
                allow_external_function_fallbacks: false,
                externals: HashMap::with_capacity(0),
                internal_functions,
                dynamic_interfaces,
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
                    return Err(StoryError::InvalidStoryState(format!("Shouldn't use a divert target (to {}) as a conditional value. Did you intend a function call 'likeThis()'?", target_path)));
                }

                return val.is_truthy();
            }

            Ok(truthy)
        }
    }
}

mod callstack;
mod choices;
mod control_logic;
pub mod errors;
pub mod external_functions;
mod internal_functions;
mod navigation;
mod progress;
mod state;
mod tags;
