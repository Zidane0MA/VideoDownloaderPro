pub mod parser;
pub mod worker;

#[cfg(test)]
mod cancel_test;

pub use worker::{DownloadError, DownloadResult, DownloadWorker};
