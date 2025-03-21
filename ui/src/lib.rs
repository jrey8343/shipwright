pub mod components;
pub mod static_assets;
pub mod view_engine;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Json error
    ///
    /// Return `500 Internal Server Error` on a json error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Could not render template
    ///
    /// Return `500 Internal Server Error` on a template rendering error.
    #[error("could not render template")]
    Template(#[from] minijinja::Error),
    /// Could not render component with wasm
    ///
    /// Return `500 Internal Server Error` on a component rendering error.
    #[error("error rendering component with wasm")]
    Component(#[from] extism::Error),
    /// Could not render component due to mutex poisoning
    ///
    /// Return `500 Internal Server Error` on a component rendering error.
    #[error("error rendering component as mutex poisoned")]
    Mutex,
    /// File io error while reading a template, style or asset file
    ///
    /// Return a `500 Internal Server Error` on a file io error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// File path error while manipulating a template, style or asset file
    ///
    /// Return a `500 Internal Server Error` on a file path error.
    #[error(transparent)]
    Path(#[from] std::path::StripPrefixError),
}
