pub mod error;
pub mod io;
pub mod parser;
pub mod transform;

pub use error::{AppError, AppResult};
pub use io::OutputFormat;
pub use parser::{Dataset, Row};
