use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn example_paths() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(repository_root().join("docs/llm/lesson-examples"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "ribalta"))
        .collect();
    paths.sort();
    paths
}

fn check_lesson(path: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ribalta"))
        .args(["check", "--json", "--profile", "lesson"])
        .arg(path)
        .env_remove("RIBALTA_LANG")
        .output()
        .unwrap()
}

#[test]
fn lesson_examples_pass_the_lesson_profile() {
    let paths = example_paths();
    assert!(paths.len() >= 2);
    for path in paths {
        let output = check_lesson(&path);
        assert_eq!(output.status.code(), Some(0), "{}: {}", path.display(), String::from_utf8_lossy(&output.stdout));
    }
}

#[test]
fn lesson_guide_script_embeds_every_example() {
    let output_path = std::env::temp_dir().join("ribalta-lesson-guide-it").join("lesson-guide.md");
    let status = Command::new("node")
        .arg(repository_root().join("scripts/lesson-guide.mjs"))
        .arg(&output_path)
        .current_dir(repository_root())
        .status()
        .unwrap();
    assert!(status.success());
    let guide = fs::read_to_string(&output_path).unwrap();
    for path in example_paths() {
        let source = fs::read_to_string(&path).unwrap();
        assert!(guide.contains(source.trim_end()), "{}", path.display());
    }
    assert!(guide.contains("lesson"));
    assert!(guide.contains("{#"));
    assert!(guide.contains("anchor:"));
}
