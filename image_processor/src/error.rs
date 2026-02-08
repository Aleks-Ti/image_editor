use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Input image not found: {0}")]
    InputImageNotFound(PathBuf),

    #[error("Params file not found: {0}")]
    ParamsFileNotFound(PathBuf),

    #[error("Plugin not found: {0}")]
    PluginNotFound(PathBuf),

    #[error("Failed to load image: {0}")]
    ImageLoadError(#[from] image::ImageError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Library load error: {0}")]
    LibraryLoad(#[from] libloading::Error),

    #[error("Invalid UTF-8 in params")]
    InvalidUtf8(#[from] std::str::Utf8Error),
}
