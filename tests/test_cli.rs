use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;

fn get_binary_path() -> PathBuf {
    env!("CARGO_BIN_EXE_simstring").into()
}

fn get_test_data_path(filename: &str) -> PathBuf {
    let root = env!("CARGO_MANIFEST_DIR");
    Path::new(root).join("tests").join("data").join(filename)
}

#[test]
fn test_cli_text_workflow() {
    let bin = get_binary_path();
    let source_path = get_test_data_path("test.txt");
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_owned();

    let status = Command::new(&bin)
        .arg("build")
        .arg("-d")
        .arg(&db_path)
        .arg("-n")
        .arg("2")
        .arg(&source_path)
        .status()
        .expect("Failed to execute simstring build");

    assert!(status.success(), "Build command failed");

    let output = Command::new(&bin)
        .arg("search")
        .arg("-d")
        .arg(&db_path)
        .arg("--source")
        .arg(&source_path)
        .arg("-t")
        .arg("0.8")
        .arg("-n")
        .arg("2")
        .arg("--ranked")
        .arg("foo")
        .output()
        .expect("Failed to execute simstring search");

    assert!(output.status.success(), "Search command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("foo\t1.0000"),
        "Should contain 'foo' with score 1.0000. Output: {}",
        stdout
    );
    assert!(
        stdout.contains("fooo\t0.8944"),
        "Should contain 'fooo' with score 0.8944. Output: {}",
        stdout
    );
    assert!(!stdout.contains("bar"), "Results should not contain 'bar'");
}

#[test]
fn test_cli_json_workflow() {
    let bin = get_binary_path();
    let source_path = get_test_data_path("test.json");
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_owned();

    let status = Command::new(&bin)
        .arg("build")
        .arg("-d")
        .arg(&db_path)
        .arg("--format")
        .arg("json")
        .arg(&source_path)
        .status()
        .expect("Failed to execute simstring build");

    assert!(status.success(), "Build command failed");

    let output = Command::new(&bin)
        .arg("search")
        .arg("-d")
        .arg(&db_path)
        .arg("--source")
        .arg(&source_path)
        .arg("--format")
        .arg("json")
        .arg("-t")
        .arg("1.0")
        .arg("bar")
        .output()
        .expect("Failed to execute simstring search");

    assert!(output.status.success(), "Search command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("bar"), "Results should contain 'bar'");
}

#[test]
fn test_cli_csv_workflow() {
    let bin = get_binary_path();
    let source_path = get_test_data_path("test.csv");
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_owned();

    let status = Command::new(&bin)
        .arg("build")
        .arg("-d")
        .arg(&db_path)
        .arg("--format")
        .arg("csv")
        .arg("--column")
        .arg("1")
        .arg(&source_path)
        .status()
        .expect("Failed to execute simstring build");

    assert!(status.success(), "Build command failed");

    let output = Command::new(&bin)
        .arg("search")
        .arg("-d")
        .arg(&db_path)
        .arg("--source")
        .arg(&source_path)
        .arg("--format")
        .arg("csv")
        .arg("--column")
        .arg("1")
        .arg("-t")
        .arg("0.5")
        .arg("alic")
        .output()
        .expect("Failed to execute simstring search");

    assert!(output.status.success(), "Search command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("alice"), "Results should contain 'alice'");
    assert!(!stdout.contains("bob"), "Results should not contain 'bob'");
}

#[test]
fn test_cli_stdin_input() {
    let bin = get_binary_path();

    let source_path = get_test_data_path("test.txt");
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_owned();

    Command::new(&bin)
        .arg("build")
        .arg("-d")
        .arg(&db_path)
        .arg("-n")
        .arg("2")
        .arg(&source_path)
        .status()
        .unwrap();

    let mut child = Command::new(&bin)
        .arg("search")
        .arg("-d")
        .arg(&db_path)
        .arg("--source")
        .arg(&source_path)
        .arg("-n")
        .arg("2")
        .arg("-t")
        .arg("0.5") // Lower threshold for partial matches
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn search process");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        // query for "fo" (should match foo, fooo) and "ba" (should match bar)
        stdin
            .write_all(b"fo\nba")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("foo"), "Should find foo");
    assert!(stdout.contains("bar"), "Should find bar");
}
