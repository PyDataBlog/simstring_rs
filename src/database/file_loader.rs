use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileLoadError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV error at line {line}: {message}")]
    Csv { line: usize, message: String },
}

/// Load strings from a plain text file, one string per line.
/// Empty lines are skipped.
pub fn load_text_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut strings = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if !line.is_empty() {
            strings.push(line);
        }
    }

    Ok(strings)
}

/// Load strings from a JSON file containing an array of strings.
pub fn load_json_file<P: AsRef<Path>>(path: P) -> Result<Vec<String>, FileLoadError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let strings: Vec<String> = serde_json::from_reader(reader)?;
    Ok(strings)
}

/// Load strings from a CSV file, extracting values from the specified column.
/// Uses simple comma splitting.
/// Empty values are skipped.
pub fn load_csv_file<P: AsRef<Path>>(path: P, column: usize) -> Result<Vec<String>, FileLoadError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut strings = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let columns: Vec<&str> = line.split(',').collect();

        if column >= columns.len() {
            return Err(FileLoadError::Csv {
                line: line_num + 1,
                message: format!(
                    "Column index {} out of bounds, line has {} columns",
                    column,
                    columns.len()
                ),
            });
        }

        let value = columns[column];
        if !value.is_empty() {
            strings.push(value.to_string());
        }
    }

    Ok(strings)
}
