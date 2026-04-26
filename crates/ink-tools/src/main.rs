use std::{env, fs, path::PathBuf};

use ink_compiler::{Compiler, CompilerOptions, SourceInput};

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
        count_all_visits: false,
    });

    let result = compiler.compile_sources(vec![SourceInput::named(
        source_text,
        source_filename.clone(),
    )]);
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

#[cfg(test)]
mod tests {
    use ink_compiler::{Diagnostic, SourceSpan};

    use super::*;

    #[test]
    fn formats_diagnostic_with_source_filename() {
        let diagnostic = Diagnostic::error(
            SourceSpan::new(Some("story.ink".to_string()), 3, 5),
            "bad syntax",
        );

        assert_eq!(
            format_diagnostic(&diagnostic, "fallback.ink"),
            "story.ink:3:5: bad syntax"
        );
    }

    #[test]
    fn formats_diagnostic_with_fallback_filename() {
        let diagnostic = Diagnostic::error(SourceSpan::new(None, 7, 2), "bad syntax");

        assert_eq!(
            format_diagnostic(&diagnostic, "fallback.ink"),
            "fallback.ink:7:2: bad syntax"
        );
    }
}
