//! Test support for executable ink-rs language experiments.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use ink_compiler::{
    format_diagnostics, CompiledStory, Compiler, Diagnostic, DiagnosticSeverity, SourceInput,
};
use ink_runtime::{story::Story, story_error::StoryError};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Experiment {
    path: PathBuf,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperimentCompileError {
    path: PathBuf,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaythroughStep {
    pub output: String,
    pub choices: Vec<String>,
    pub choose: Option<usize>,
}

impl Experiment {
    pub fn from_path(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let source = fs::read_to_string(&path)?;
        Ok(Self { path, source })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> String {
        self.path
            .strip_prefix(experiments_root())
            .unwrap_or(&self.path)
            .display()
            .to_string()
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn stdout_path(&self) -> PathBuf {
        let mut path = self.path.as_os_str().to_os_string();
        path.push(".stdout");
        PathBuf::from(path)
    }

    pub fn playthrough_path(&self) -> PathBuf {
        let mut path = self.path.as_os_str().to_os_string();
        path.push(".playthrough.json");
        PathBuf::from(path)
    }

    pub fn expected_stdout(&self) -> io::Result<Option<String>> {
        let path = self.stdout_path();
        if path.exists() {
            fs::read_to_string(path).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn expected_playthrough(&self) -> io::Result<Option<Vec<PlaythroughStep>>> {
        let path = self.playthrough_path();
        if !path.exists() {
            return Ok(None);
        }

        parse_playthrough(&fs::read_to_string(path)?).map(Some)
    }

    pub fn compile(&self) -> Result<CompiledStory, ExperimentCompileError> {
        let output =
            Compiler::default().compile(SourceInput::named(self.source.clone(), self.name()));
        let diagnostics = output.diagnostics;

        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            || !diagnostics.is_empty()
        {
            return Err(ExperimentCompileError {
                path: self.path.clone(),
                diagnostics,
            });
        }

        output.artifact.ok_or_else(|| ExperimentCompileError {
            path: self.path.clone(),
            diagnostics: Vec::new(),
        })
    }

    pub fn story(&self) -> Result<Story, ExperimentStoryError> {
        let compiled = self
            .compile()
            .map_err(ExperimentStoryError::from_compile_error)?;
        Story::new(&compiled.json).map_err(|error| ExperimentStoryError {
            kind: ExperimentStoryErrorKind::Runtime {
                path: self.path.clone(),
                error,
            },
        })
    }
}

fn parse_playthrough(source: &str) -> io::Result<Vec<PlaythroughStep>> {
    let value = serde_json::from_str::<Value>(source).map_err(invalid_playthrough)?;
    let Value::Array(steps) = value else {
        return Err(invalid_playthrough("playthrough root must be an array"));
    };

    steps
        .into_iter()
        .enumerate()
        .map(|(index, step)| parse_playthrough_step(index, step))
        .collect()
}

fn parse_playthrough_step(index: usize, value: Value) -> io::Result<PlaythroughStep> {
    let Value::Object(mut fields) = value else {
        return Err(invalid_playthrough(format!(
            "playthrough step {index} must be an object"
        )));
    };

    let output = take_optional_string(&mut fields, "output")?.unwrap_or_default();
    let choices = take_optional_string_array(&mut fields, "choices")?.unwrap_or_default();
    let choose = take_optional_usize(&mut fields, "choose")?;

    if !fields.is_empty() {
        return Err(invalid_playthrough(format!(
            "playthrough step {index} has unknown fields: {:?}",
            fields.keys().collect::<Vec<_>>()
        )));
    }

    Ok(PlaythroughStep {
        output,
        choices,
        choose,
    })
}

fn take_optional_string(
    fields: &mut serde_json::Map<String, Value>,
    name: &str,
) -> io::Result<Option<String>> {
    match fields.remove(name) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(invalid_playthrough(format!("{name} must be a string"))),
    }
}

fn take_optional_string_array(
    fields: &mut serde_json::Map<String, Value>,
    name: &str,
) -> io::Result<Option<Vec<String>>> {
    match fields.remove(name) {
        None => Ok(None),
        Some(Value::Array(values)) => values
            .into_iter()
            .map(|value| match value {
                Value::String(value) => Ok(value),
                _ => Err(invalid_playthrough(format!(
                    "{name} must contain only strings"
                ))),
            })
            .collect::<io::Result<Vec<_>>>()
            .map(Some),
        Some(_) => Err(invalid_playthrough(format!("{name} must be an array"))),
    }
}

fn take_optional_usize(
    fields: &mut serde_json::Map<String, Value>,
    name: &str,
) -> io::Result<Option<usize>> {
    match fields.remove(name) {
        None => Ok(None),
        Some(Value::Number(value)) => {
            let Some(value) = value.as_u64() else {
                return Err(invalid_playthrough(format!(
                    "{name} must be a non-negative integer"
                )));
            };
            usize::try_from(value)
                .map(Some)
                .map_err(|_| invalid_playthrough(format!("{name} does not fit usize")))
        }
        Some(_) => Err(invalid_playthrough(format!("{name} must be an integer"))),
    }
}

fn invalid_playthrough(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

impl ExperimentCompileError {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn formatted_diagnostics(&self) -> String {
        if self.diagnostics.is_empty() {
            "compiler produced no artifact and no diagnostics".to_string()
        } else {
            format_diagnostics(&self.diagnostics)
        }
    }
}

#[derive(Debug)]
pub struct ExperimentStoryError {
    kind: ExperimentStoryErrorKind,
}

#[derive(Debug)]
enum ExperimentStoryErrorKind {
    Compile(ExperimentCompileError),
    Runtime { path: PathBuf, error: StoryError },
}

impl ExperimentStoryError {
    fn from_compile_error(error: ExperimentCompileError) -> Self {
        Self {
            kind: ExperimentStoryErrorKind::Compile(error),
        }
    }

    pub fn path(&self) -> &Path {
        match &self.kind {
            ExperimentStoryErrorKind::Compile(error) => error.path(),
            ExperimentStoryErrorKind::Runtime { path, .. } => path,
        }
    }

    pub fn compile_error(&self) -> Option<&ExperimentCompileError> {
        match &self.kind {
            ExperimentStoryErrorKind::Compile(error) => Some(error),
            ExperimentStoryErrorKind::Runtime { .. } => None,
        }
    }

    pub fn runtime_error(&self) -> Option<&StoryError> {
        match &self.kind {
            ExperimentStoryErrorKind::Compile(_) => None,
            ExperimentStoryErrorKind::Runtime { error, .. } => Some(error),
        }
    }
}

pub fn experiments_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments")
}

pub fn discover_experiments() -> io::Result<Vec<Experiment>> {
    let mut paths = Vec::new();
    collect_ink_paths(&experiments_root(), &mut paths)?;
    paths.sort();

    paths
        .into_iter()
        .map(Experiment::from_path)
        .collect::<io::Result<Vec<_>>>()
}

fn collect_ink_paths(directory: &Path, paths: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            collect_ink_paths(&path, paths)?;
        } else if path.extension().is_some_and(|extension| extension == "ink") {
            paths.push(path);
        }
    }

    Ok(())
}
