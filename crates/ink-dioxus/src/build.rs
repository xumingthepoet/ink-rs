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

pub fn generate_ink_game_catalog(games_dir: impl AsRef<Path>) -> io::Result<()> {
    let games_dir = games_dir.as_ref();
    let games_dir = games_dir.canonicalize()?;
    let mut games = Vec::new();
    collect_game_dirs(&games_dir, &mut games)?;
    games.sort_by(|left, right| left.id.cmp(&right.id));

    println!("cargo:rerun-if-changed={}", games_dir.display());
    for game in &games {
        println!("cargo:rerun-if-changed={}", game.path.display());
        for file in &game.files {
            println!("cargo:rerun-if-changed={}", file.display());
        }
    }

    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "OUT_DIR is not set"))?;
    fs::write(
        out_dir.join("ink_games.rs"),
        render_game_catalog(&games_dir, &games),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GameSourceBundle {
    id: String,
    title: String,
    description: String,
    path: PathBuf,
    files: Vec<PathBuf>,
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

fn collect_game_dirs(games_dir: &Path, games: &mut Vec<GameSourceBundle>) -> io::Result<()> {
    for entry in fs::read_dir(games_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let path = entry.path();
        let mut files = Vec::new();
        collect_ink_files(&path, &mut files)?;
        files.sort();
        if files.is_empty() {
            continue;
        }

        let id = entry.file_name().to_string_lossy().to_string();
        games.push(GameSourceBundle {
            title: title_from_id(&id),
            description: "A playable ink-rs text game.".to_string(),
            id,
            path,
            files,
        });
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

fn render_game_catalog(games_dir: &Path, games: &[GameSourceBundle]) -> String {
    let mut output = String::from("use ink_dioxus::{InkGameSource, InkSource};\n\n");

    for game in games {
        output.push_str("pub static ");
        output.push_str(&game_sources_static_name(&game.id));
        output.push_str(": &[InkSource] = &[\n");
        for file in &game.files {
            output.push_str("    InkSource::new(");
            output.push_str(&rust_string_literal(&source_filename(games_dir, file)));
            output.push_str(", include_str!(");
            output.push_str(&rust_string_literal(
                file.to_str().expect("ink path should be UTF-8"),
            ));
            output.push_str(")),\n");
        }
        output.push_str("];\n\n");
    }

    output.push_str("pub static INK_GAMES: &[InkGameSource] = &[\n");
    for game in games {
        output.push_str("    InkGameSource::new(");
        output.push_str(&rust_string_literal(&game.id));
        output.push_str(", ");
        output.push_str(&rust_string_literal(&game.title));
        output.push_str(", ");
        output.push_str(&rust_string_literal(&game.description));
        output.push_str(", ");
        output.push_str(&game_sources_static_name(&game.id));
        output.push_str("),\n");
    }
    output.push_str("];\n");
    output
}

fn source_filename(source_dir: &Path, file: &Path) -> String {
    file.strip_prefix(source_dir)
        .ok()
        .and_then(|path| path.to_str())
        .unwrap_or_else(|| file.to_str().expect("ink path should be UTF-8"))
        .to_string()
}

fn game_sources_static_name(id: &str) -> String {
    let mut name = String::from("INK_GAME_");
    for character in id.chars() {
        if character.is_ascii_alphanumeric() {
            name.push(character.to_ascii_uppercase());
        } else {
            name.push('_');
        }
    }
    name.push_str("_SOURCES");
    name
}

fn title_from_id(id: &str) -> String {
    id.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut title = String::new();
            title.extend(first.to_uppercase());
            title.push_str(chars.as_str());
            title
        })
        .collect::<Vec<_>>()
        .join(" ")
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

    #[test]
    fn render_game_catalog_generates_one_source_list_per_game() {
        let games_dir = Path::new("/game/assets/ink");
        let games = vec![GameSourceBundle {
            id: "text-snake-10x10".to_string(),
            title: "Text Snake 10x10".to_string(),
            description: "A playable ink-rs text game.".to_string(),
            path: PathBuf::from("/game/assets/ink/text-snake-10x10"),
            files: vec![PathBuf::from("/game/assets/ink/text-snake-10x10/story.ink")],
        }];

        let rendered = render_game_catalog(games_dir, &games);

        assert!(rendered.contains("pub static INK_GAME_TEXT_SNAKE_10X10_SOURCES"));
        assert!(rendered.contains("InkSource::new(\"text-snake-10x10/story.ink\""));
        assert!(rendered.contains("pub static INK_GAMES: &[InkGameSource]"));
        assert!(rendered.contains("InkGameSource::new(\"text-snake-10x10\""));
    }
}
