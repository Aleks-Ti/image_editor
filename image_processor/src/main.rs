//! Image processing application with plugin support

#![warn(missing_docs)]

mod error;
mod plugin_loader;

use clap::Parser;
use error::AppError;
use plugin_loader::Plugin;

use image::{ImageBuffer, Rgba};
use std::ffi::CString;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about="Image processing application with plugin support", long_about = None)]
struct Cli {
    input: PathBuf,
    output: PathBuf,
    plugin: String,
    params: PathBuf,

    #[arg(long, default_value = "target/debug")]
    plugin_path: PathBuf,
}

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    if !cli.input.exists() {
        return Err(AppError::InputImageNotFound(cli.input));
    }
    if !cli.params.exists() {
        return Err(AppError::ParamsFileNotFound(cli.params));
    }

    let img = image::open(&cli.input)?.to_rgba8();
    let (width, height) = img.dimensions();
    let mut buffer = img.into_raw();

    let params_text = fs::read_to_string(&cli.params)?;
    let params_c = CString::new(params_text).expect("CString conversion failed");

    let plugin = Plugin::load(&cli.plugin_path, &cli.plugin)?;

    unsafe {
        (plugin.process_image)(width, height, buffer.as_mut_ptr(), params_c.as_ptr());
    }

    let out_img: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width, height, buffer).expect("Invalid image buffer size");

    out_img.save(&cli.output)?;

    Ok(())
}
