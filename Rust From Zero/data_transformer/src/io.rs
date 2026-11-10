use std::fs;
use std::path::Path;

use crate::error::{AppError, AppResult};
use crate::parser::Dataset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Csv,
    Json,
}

/// Reads a CSV file.
///
/// The path is borrowed because the caller owns the path.
pub fn read_csv(path: &Path) -> AppResult<Dataset> {
    let content = fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    crate::parser::parse_csv(&content)
}

/// Serializes a dataset into CSV.
///
/// The dataset is borrowed because serialization does not
/// need to consume or modify it.
pub fn serialize_csv(dataset: &Dataset) -> AppResult<String> {
    let mut writer = csv::Writer::from_writer(Vec::new());

    writer.write_record(&dataset.headers)?;

    for row in &dataset.rows {
        writer.write_record(&row.values)?;
    }

    let bytes = writer.into_inner().map_err(|error| AppError::Io {
        path: "<memory>".into(),
        source: error.into_error(),
    })?;

    String::from_utf8(bytes).map_err(|error| AppError::Io {
        path: "<memory>".into(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, error),
    })
}

/// Serializes rows into JSON objects.
pub fn serialize_json(dataset: &Dataset) -> AppResult<String> {
    let rows = dataset
        .rows
        .iter()
        .map(|row| {
            dataset
                .headers
                .iter()
                .zip(row.values.iter())
                .map(|(header, value)| (header.clone(), value.clone()))
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .collect::<Vec<_>>();

    Ok(serde_json::to_string_pretty(&rows)?)
}

/// Writes serialized content to a file.
pub fn write_output(path: &Path, content: &str) -> AppResult<()> {
    fs::write(path, content).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Serializes a dataset according to its output format.
pub fn serialize(dataset: &Dataset, format: OutputFormat) -> AppResult<String> {
    match format {
        OutputFormat::Csv => serialize_csv(dataset),
        OutputFormat::Json => serialize_json(dataset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_csv;

    #[test]
    fn serializes_csv() {
        let dataset = parse_csv("id,name\n1,Alice\n").expect("valid test input");

        let output = serialize_csv(&dataset).expect("valid serialization");

        assert!(output.contains("Alice"));
    }

    #[test]
    fn serializes_json() {
        let dataset = parse_csv("id,name\n1,Alice\n").expect("valid test input");

        let output = serialize_json(&dataset).expect("valid serialization");

        assert!(output.contains("\"name\": \"Alice\""));
    }

    #[test]
    fn writes_output() {
        let path = std::env::temp_dir().join("rust-data-transformer-test.txt");

        write_output(&path, "test").expect("valid output path");

        assert_eq!(fs::read_to_string(&path).expect("written file"), "test");

        fs::remove_file(path).expect("remove test file");
    }

    #[test]
    fn rejects_invalid_output_path() {
        let path = std::env::temp_dir()
            .join("rust-data-transformer-missing")
            .join("output.txt");

        assert!(write_output(&path, "test").is_err());
    }
}
