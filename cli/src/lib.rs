//! The my-app-cli crate implements the project's CLI tools `db` and `generate` as well as contains functionality for displaying information in a console UI.

/// Utilities for CLIs
pub mod util;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to load configuration: {0}")]
    Config(#[from] shipwright_config::Error),
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Filesystem io error")]
    Io(#[from] std::io::Error),
    #[error("Other error")]
    Other(#[from] color_eyre::Report),
}
