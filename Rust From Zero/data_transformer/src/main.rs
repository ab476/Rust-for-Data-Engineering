use std::path::PathBuf;

use data_transformer::{AppError, AppResult, OutputFormat, io, transform};

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> AppResult<()> {
    let mut args = std::env::args().skip(1);

    let input = args.next().ok_or_else(|| {
        AppError::InvalidFilter(
            "missing input path. Usage: rust-data-transformer <input.csv>".to_owned(),
        )
    })?;

    let mut filter = None;
    let mut map = None;
    let mut format = OutputFormat::Csv;
    let mut output = None;

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--filter" => {
                filter =
                    Some(args.next().ok_or_else(|| {
                        AppError::InvalidFilter("missing column=value".to_owned())
                    })?);
            }
            "--map" => {
                map = Some(
                    args.next()
                        .ok_or_else(|| AppError::InvalidMap("missing column=factor".to_owned()))?,
                );
            }
            "--format" => {
                let value = args
                    .next()
                    .ok_or_else(|| AppError::UnsupportedFormat("missing format".to_owned()))?;

                format = match value.as_str() {
                    "csv" => OutputFormat::Csv,
                    "json" => OutputFormat::Json,
                    _ => return Err(AppError::UnsupportedFormat(value)),
                };
            }
            "--output" => {
                output = Some(PathBuf::from(args.next().ok_or_else(|| AppError::Io {
                    path: PathBuf::from("<output>"),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "missing output path",
                    ),
                })?));
            }
            _ => {
                return Err(AppError::InvalidFilter(format!(
                    "unknown argument: {argument}"
                )));
            }
        }
    }

    let mut dataset = io::read_csv(&PathBuf::from(input))?;

    if let Some(expression) = filter {
        let (column, value) = expression
            .split_once('=')
            .ok_or_else(|| AppError::InvalidFilter(expression.clone()))?;

        dataset = transform::filter_eq(&dataset, column, value)?;
    }

    if let Some(expression) = map {
        let (column, factor) = expression
            .split_once('=')
            .ok_or_else(|| AppError::InvalidMap(expression.clone()))?;

        let factor = factor.parse::<f64>().map_err(|_| AppError::InvalidNumber {
            column: column.to_owned(),
            value: factor.to_owned(),
        })?;

        transform::multiply_column(&mut dataset, column, factor)?;
    }

    let content = io::serialize(&dataset, format)?;

    if let Some(path) = output {
        io::write_output(&path, &content)?;
    } else {
        print!("{content}");
    }

    Ok(())
}
