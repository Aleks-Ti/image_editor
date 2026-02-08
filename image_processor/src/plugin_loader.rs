use libloading::Library;
use std::path::Path;

use crate::error::AppError;

pub type ProcessImageFn = unsafe extern "C" fn(
    width: u32,
    height: u32,
    rgba_data: *mut u8,
    params: *const std::os::raw::c_char,
);

pub struct Plugin {
    _lib: Library,
    pub process_image: ProcessImageFn,
}

fn platform_library_name(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{name}.dylib")
    } else {
        format!("lib{name}.so")
    }
}

impl Plugin {
    /// Loads a plugin from the specified directory and name
    pub fn load(plugin_dir: &Path, plugin_name: &str) -> Result<Self, AppError> {
        let lib_name = platform_library_name(plugin_name);
        let lib_path = plugin_dir.join(lib_name);

        if !lib_path.exists() {
            return Err(AppError::PluginNotFound(lib_path));
        }

        let lib = unsafe { Library::new(&lib_path)? };

        let process_image = unsafe {
            let symbol: libloading::Symbol<ProcessImageFn> = lib.get(b"process_image\0")?;
            *symbol
        };

        Ok(Self {
            _lib: lib,
            process_image,
        })
    }
}
