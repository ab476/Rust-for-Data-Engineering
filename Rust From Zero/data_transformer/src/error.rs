use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error while accessing {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Input is empty")]
    EmptyInput,

    #[error("CSV header is empty")]
    EmptyHeaders,

    #[error("Column '{0}' does not exist")]
    MissingColumn(String),

    #[error("Invalid numeric value '{value}' in column '{column}'")]
    InvalidNumber { column: String, value: String },

    #[error("Invalid filter '{0}'. Expected column=value")]
    InvalidFilter(String),

    #[error("Invalid map expression '{0}'. Expected column=factor")]
    InvalidMap(String),

    #[error("Unsupported output format '{0}'")]
    UnsupportedFormat(String),
}

pub type AppResult<T> = Result<T, AppError>;
