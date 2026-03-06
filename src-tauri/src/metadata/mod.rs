pub mod fetcher;
pub mod format_processor;
pub mod models;
pub mod store;
pub mod tiktok;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("Sidecar error: {0}")]
    Sidecar(String),
    #[error("JSON parse error: {0}")]
    Parse(String),
    #[error("Execution error: {0}")]
    Execution(String),
}

#[cfg(test)]
mod tests;
