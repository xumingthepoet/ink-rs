use std::{
    env, fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use ink_compiler::{Compiler, CompilerOptions, FileHandler, SourceInput};

#[derive(Debug)]
struct CliFileHandler {
    base_dir: PathBuf,
}

impl CliFileHandler {
    fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl FileHandler for CliFileHandler {
    fn resolve_ink_filename(&self, include_name: &str) -> PathBuf {
        let include_path = Path::new(include_name);
        if include_path.is_absolute() {
            include_path.to_path_buf()
        } else {
            self.base_dir.join(include_path)
        }
    }

    fn load_ink_file_contents(&self, full_filename: &Path) -> io::Result<String> {
        fs::read_to_string(full_filename)
    }
}

fn main() {
    if let Err(message) = run() {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let source_path = args.next().ok_or_else(usage)?;
    let output_path = args.next().map(PathBuf::from);

    if args.next().is_some() {
        return Err(usage());
    }

    let source_path = PathBuf::from(source_path);
    let source_text = fs::read_to_string(&source_path)
        .map_err(|error| format!("Failed to read '{}': {error}", source_path.display()))?;
    let base_dir = source_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let file_handler: Arc<dyn FileHandler> = Arc::new(CliFileHandler::new(base_dir));
    let source_filename = source_path.display().to_string();

    let compiler = Compiler::with_options(CompilerOptions {
        source_filename: Some(source_filename.clone()),
        count_all_visits: false,
        file_handler: Some(file_handler),
    });

    let result = compiler.compile(SourceInput::named(source_text, source_filename.clone()));
    for diagnostic in &result.diagnostics {
        eprintln!("{}", format_diagnostic(diagnostic, &source_filename));
    }

    let Some(compiled) = result.artifact else {
        return Err("Compilation failed.".to_string());
    };
    let json = compiled.json;

    if let Some(output_path) = output_path {
        fs::write(&output_path, json)
            .map_err(|error| format!("Failed to write '{}': {error}", output_path.display()))?;
    } else {
        println!("{json}");
    }

    Ok(())
}

fn format_diagnostic(diagnostic: &ink_compiler::Diagnostic, fallback_source: &str) -> String {
    let source_filename = diagnostic
        .source_filename
        .as_deref()
        .unwrap_or(fallback_source);

    format!(
        "{source_filename}:{}:{}: {}",
        diagnostic.line, diagnostic.column, diagnostic.message
    )
}

fn usage() -> String {
    "Usage: ink_compile <story.ink> [output.json]".to_string()
}
