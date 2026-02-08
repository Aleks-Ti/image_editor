//! Mirror plugin for image processing application

#![warn(missing_docs)]
use serde::Deserialize;
use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Deserialize)]
struct Params {
    horizontal: bool,
    vertical: bool,
}

/// Processes an image in-place by applying a mirror effect.
///
/// # Safety
///
/// - `data` must be a valid, writable pointer to a buffer of at least
///   `width * height * 4` bytes (RGBA8 format).
/// - The memory pointed to by `data` must be properly aligned and remain
///   valid for the duration of the call.
/// - `params` must be either a null pointer or a valid null-terminated
///   C string.
/// - This function must not be called concurrently on the same buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    data: *mut u8,
    params: *const c_char,
) {
    unsafe {
        let len = (width * height * 4) as usize;
        let slice = std::slice::from_raw_parts_mut(data, len);

        let params_str = if params.is_null() {
            ""
        } else {
            CStr::from_ptr(params).to_str().unwrap_or("")
        };

        let params: Params = serde_json::from_str(params_str).unwrap_or(Params {
            horizontal: false,
            vertical: false,
        });

        let w = width as usize;
        let h = height as usize;

        let copy = slice.to_vec();

        for y in 0..h {
            for x in 0..w {
                let src_x = if params.horizontal { w - 1 - x } else { x };
                let src_y = if params.vertical { h - 1 - y } else { y };

                let dst = (y * w + x) * 4;
                let src = (src_y * w + src_x) * 4;

                slice[dst..dst + 4].copy_from_slice(&copy[src..src + 4]);
            }
        }
    }
}
