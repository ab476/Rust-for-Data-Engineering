use crate::error::{AppError, AppResult};
use crate::parser::{Dataset, column_index};

/// Filters rows using shared borrowing.
///
/// The input remains unchanged. The returned dataset owns its selected rows.
pub fn filter_eq(dataset: &Dataset, column: &str, value: &str) -> AppResult<Dataset> {
    let index = column_index(dataset, column)?;

    let rows = dataset
        .rows
        .iter()
        .filter(|row| row.values[index] == value)
        .cloned()
        .collect();

    Ok(Dataset {
        headers: dataset.headers.clone(),
        rows,
    })
}

/// Mutates a numeric column in place.
///
/// Mutable borrowing avoids copying the entire dataset.
pub fn multiply_column(dataset: &mut Dataset, column: &str, factor: f64) -> AppResult<()> {
    if !factor.is_finite() {
        return Err(AppError::InvalidNumber {
            column: column.to_owned(),
            value: factor.to_string(),
        });
    }

    let index = column_index(dataset, column)?;

    for row in &mut dataset.rows {
        let value = &row.values[index];

        let number = value.parse::<f64>().map_err(|_| AppError::InvalidNumber {
            column: column.to_owned(),
            value: value.clone(),
        })?;

        let result = number * factor;

        if !result.is_finite() {
            return Err(AppError::InvalidNumber {
                column: column.to_owned(),
                value: result.to_string(),
            });
        }

        row.values[index] = result.to_string();
    }

    Ok(())
}

/// Consumes the dataset and returns its rows.
///
/// Taking ownership is useful when the caller no longer needs
/// the original dataset.
pub fn into_rows(dataset: Dataset) -> Vec<Vec<String>> {
    dataset.rows.into_iter().map(|row| row.values).collect()
}

/// Computes the average of numeric values.
///
/// Copy is sufficient because the numeric values are small,
/// copyable scalar values. Into<f64> provides conversion.
///
/// Shared borrowing means the input slice is not consumed.
pub fn average<T>(values: &[T]) -> AppResult<f64>
where
    T: Copy + Into<f64>,
{
    if values.is_empty() {
        return Err(AppError::EmptyInput);
    }

    let sum = values.iter().map(|value| (*value).into()).sum::<f64>();

    let average = sum / values.len() as f64;

    if !average.is_finite() {
        return Err(AppError::InvalidNumber {
            column: "average".to_owned(),
            value: average.to_string(),
        });
    }

    Ok(average)
}

/// Extracts numeric values from a dataset column and calculates its average.
pub fn average_column(dataset: &Dataset, column: &str) -> AppResult<f64> {
    let index = column_index(dataset, column)?;

    let values = dataset
        .rows
        .iter()
        .map(|row| {
            row.values[index]
                .parse::<f64>()
                .map_err(|_| AppError::InvalidNumber {
                    column: column.to_owned(),
                    value: row.values[index].clone(),
                })
        })
        .collect::<AppResult<Vec<_>>>()?;

    average(&values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_csv;

    #[test]
    fn filters_rows() {
        let dataset = parse_csv("id,department\n1,Engineering\n2,Sales\n3,Engineering\n")
            .expect("valid test input");

        let result = filter_eq(&dataset, "department", "Engineering").expect("valid filter");

        assert_eq!(result.rows.len(), 2);
        assert_eq!(dataset.rows.len(), 3);
    }

    #[test]
    fn filter_rejects_missing_column() {
        let dataset = parse_csv("id,name\n1,Alice\n").expect("valid test input");

        assert!(filter_eq(&dataset, "salary", "100").is_err());
    }

    #[test]
    fn mutates_numeric_column() {
        let mut dataset = parse_csv("id,salary\n1,100\n2,200\n").expect("valid test input");

        multiply_column(&mut dataset, "salary", 2.0).expect("valid transformation");

        assert_eq!(dataset.rows[0].values[1], "200");
        assert_eq!(dataset.rows[1].values[1], "400");
    }

    #[test]
    fn mutation_rejects_invalid_number() {
        let mut dataset = parse_csv("id,salary\n1,abc\n").expect("valid test input");

        assert!(multiply_column(&mut dataset, "salary", 2.0).is_err());
    }

    #[test]
    fn mutation_rejects_missing_column() {
        let mut dataset = parse_csv("id,salary\n1,100\n").expect("valid test input");

        assert!(multiply_column(&mut dataset, "bonus", 2.0).is_err());
    }

    #[test]
    fn calculates_average() {
        assert_eq!(average(&[10.0, 20.0, 30.0]).unwrap(), 20.0);
    }

    #[test]
    fn average_rejects_empty_values() {
        assert!(average(&[] as &[f64]).is_err());
    }

    #[test]
    fn consumes_dataset() {
        let dataset = parse_csv("id,name\n1,Alice\n").expect("valid test input");

        let rows = into_rows(dataset);

        assert_eq!(rows[0], ["1", "Alice"]);
    }
}
