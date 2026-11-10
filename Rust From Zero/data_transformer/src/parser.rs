use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Dataset {
    pub headers: Vec<String>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub values: Vec<String>,
}

/// Parses CSV text into an owned dataset.
///
/// Ownership is intentional: the returned dataset must remain usable
/// after the original input string has been dropped.
pub fn parse_csv(input: &str) -> AppResult<Dataset> {
    if input.is_empty() {
        return Err(AppError::EmptyInput);
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input.as_bytes());

    let headers = reader
        .headers()?
        .iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if headers.is_empty() {
        return Err(AppError::EmptyHeaders);
    }

    let mut rows = Vec::new();

    for record in reader.records() {
        let record = record?;

        rows.push(Row {
            values: record.iter().map(str::to_owned).collect(),
        });
    }

    Ok(Dataset { headers, rows })
}

/// Returns a column index using shared borrowing.
///
/// The dataset is only inspected, not modified or consumed.
pub fn column_index(dataset: &Dataset, column: &str) -> AppResult<usize> {
    dataset
        .headers
        .iter()
        .position(|header| header == column)
        .ok_or_else(|| AppError::MissingColumn(column.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_csv() {
        let input = "id,name\n1,Alice\n2,Bob\n";

        let dataset = parse_csv(input).expect("valid test input");

        assert_eq!(dataset.headers, ["id", "name"]);
        assert_eq!(dataset.rows.len(), 2);
    }

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(parse_csv(""), Err(AppError::EmptyInput)));
    }

    #[test]
    fn rejects_missing_column() {
        let dataset = parse_csv("id,name\n1,Alice\n").expect("valid test input");

        assert!(matches!(
            column_index(&dataset, "salary"),
            Err(AppError::MissingColumn(_))
        ));
    }

    #[test]
    fn finds_column_index() {
        let dataset = parse_csv("id,name\n1,Alice\n").expect("valid test input");

        assert_eq!(column_index(&dataset, "name").unwrap(), 1);
    }
}
