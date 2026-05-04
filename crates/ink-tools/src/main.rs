use std::{env, fs, path::PathBuf};

use ink_compiler::{format_diagnostics, Compiler, CompilerOptions, SourceInput};

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
    let source_filename = source_path.display().to_string();

    let compiler = Compiler::with_options(CompilerOptions {
        source_filename: Some(source_filename.clone()),
        ..CompilerOptions::default()
    });

    let result = compiler.compile_sources(vec![SourceInput::named(
        source_text,
        source_filename.clone(),
    )]);
    if !result.diagnostics.is_empty() {
        eprintln!("{}", format_diagnostics(&result.diagnostics));
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
fn usage() -> String {
    "Usage: ink_compile <story.ink> [output.json]".to_string()
}
