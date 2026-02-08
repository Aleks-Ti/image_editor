use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    /// Error when the input image file is not found
    #[error("Input image not found: {0}")]
    InputImageNotFound(PathBuf),

    /// Error when the parameters file is not found
    #[error("Params file not found: {0}")]
    ParamsFileNotFound(PathBuf),

    /// Error when the plugin library cannot be found or loaded
    #[error("Plugin not found: {0}")]
    PluginNotFound(PathBuf),

    /// Error when the plugin does not contain the required `process_image` function
    #[error("Failed to load image: {0}")]
    ImageLoadError(#[from] image::ImageError),

    /// Error when the plugin library cannot be loaded
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error when the plugin library cannot be loaded
    #[error("Library load error: {0}")]
    LibraryLoad(#[from] libloading::Error),

    /// Error when the plugin library cannot be loaded
    #[error("Invalid UTF-8 in params")]
    InvalidUtf8(#[from] std::str::Utf8Error),
}
