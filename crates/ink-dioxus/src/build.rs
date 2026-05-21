use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

pub fn generate_ink_source_list(source_dir: impl AsRef<Path>) -> io::Result<()> {
    let source_dir = source_dir.as_ref();
    let source_dir = source_dir.canonicalize()?;
    let mut files = Vec::new();
    collect_ink_files(&source_dir, &mut files)?;
    files.sort();

    println!("cargo:rerun-if-changed={}", source_dir.display());
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
    }

    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "OUT_DIR is not set"))?;
    fs::write(
        out_dir.join("ink_sources.rs"),
        render_source_list(&source_dir, &files),
    )
}

fn collect_ink_files(path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_ink_files(&file_path, files)?;
            continue;
        }

        if file_path.extension().and_then(|ext| ext.to_str()) == Some("ink") {
            files.push(file_path);
        }
    }
    Ok(())
}

fn render_source_list(source_dir: &Path, files: &[PathBuf]) -> String {
    let mut output =
        String::from("use ink_dioxus::InkSource;\n\npub static INK_SOURCES: &[InkSource] = &[\n");

    for file in files {
        let filename = file
            .strip_prefix(source_dir)
            .ok()
            .and_then(|path| path.to_str())
            .unwrap_or_else(|| file.to_str().expect("ink path should be UTF-8"));
        let source_path = file.to_str().expect("ink path should be UTF-8");
        output.push_str("    InkSource::new(");
        output.push_str(&rust_string_literal(filename));
        output.push_str(", include_str!(");
        output.push_str(&rust_string_literal(source_path));
        output.push_str(")),\n");
    }

    output.push_str("];\n");
    output
}

fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_source_list_generates_stable_include_entries() {
        let source_dir = Path::new("/game/assets/ink/src");
        let files = vec![
            PathBuf::from("/game/assets/ink/src/main.ink"),
            PathBuf::from("/game/assets/ink/src/world/start.ink"),
        ];

        let rendered = render_source_list(source_dir, &files);

        assert!(rendered.contains("pub static INK_SOURCES: &[InkSource]"));
        assert!(rendered.contains("InkSource::new(\"main.ink\""));
        assert!(rendered.contains("InkSource::new(\"world/start.ink\""));
        assert!(rendered.contains("include_str!(\"/game/assets/ink/src/main.ink\")"));
    }
}
