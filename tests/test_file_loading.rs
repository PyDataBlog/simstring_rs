use simstring_rust::database::file_loader::{
    load_csv_file, load_json_file, load_text_file, FileLoadError,
};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_load_text_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "hello").unwrap();
    writeln!(file, "world").unwrap();
    writeln!(file, "").unwrap(); // Empty line should be skipped
    writeln!(file, "  test  ").unwrap(); // Whitespace should NOT be trimmed

    let strings = load_text_file(file.path()).unwrap();
    assert_eq!(strings, vec!["hello", "world", "  test  "]);
}

#[test]
fn test_load_json_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"["foo", "bar", "baz"]"#).unwrap();

    let strings = load_json_file(file.path()).unwrap();
    assert_eq!(strings, vec!["foo", "bar", "baz"]);
}

#[test]
fn test_load_csv_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "id,name,value").unwrap();
    writeln!(file, "1,hello,100").unwrap();
    writeln!(file, "2,world,200").unwrap();

    // Load column 1 (name)
    let strings = load_csv_file(file.path(), 1).unwrap();
    assert_eq!(strings, vec!["name", "hello", "world"]);
}

#[test]
fn test_load_csv_file_column_out_of_bounds() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "a,b").unwrap();

    let result = load_csv_file(file.path(), 5);
    assert!(matches!(result, Err(FileLoadError::Csv { .. })));
}
