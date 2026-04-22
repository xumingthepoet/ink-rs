use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn unique_temp_dir() -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for tests")
        .as_millis();
    env::temp_dir().join(format!("ink-compiler-cli-{}-{millis}", std::process::id()))
}

#[test]
fn cli_compiles_a_story_with_relative_includes() {
    let temp_dir = unique_temp_dir();
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let story_path = temp_dir.join("main.ink");
    let include_path = temp_dir.join("chapter.ink");
    fs::write(&include_path, "Included line.\n").expect("write include file");
    fs::write(&story_path, "Prelude.\nINCLUDE chapter.ink\nPostlude.\n").expect("write story file");

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");

    let output = Command::new("cargo")
        .current_dir(workspace_root)
        .args([
            "run",
            "--quiet",
            "-p",
            "ink-compiler",
            "--example",
            "ink_compile",
            "--",
        ])
        .arg(&story_path)
        .output()
        .expect("run ink_compile");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(stdout.contains("^Prelude."), "stdout: {stdout}");
    assert!(stdout.contains("^Included line."), "stdout: {stdout}");
    assert!(stdout.contains("^Postlude."), "stdout: {stdout}");
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap_or_else(|_| "<non-utf8 stderr>".to_string())
}
