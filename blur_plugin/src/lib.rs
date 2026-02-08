use serde::Deserialize;
use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Deserialize)]
struct Params {
    radius: u32,
    iterations: u32,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    data: *mut u8,
    params: *const c_char,
) {
    unsafe {
        let len = (width * height * 4) as usize;
        let buf = std::slice::from_raw_parts_mut(data, len);

        let params_str = if params.is_null() {
            ""
        } else {
            CStr::from_ptr(params).to_str().unwrap_or("")
        };

        let params: Params = serde_json::from_str(params_str).unwrap_or(Params {
            radius: 1,
            iterations: 1,
        });

        let w = width as i32;
        let h = height as i32;

        let mut temp = buf.to_vec();

        for _ in 0..params.iterations {
            for y in 0..h {
                for x in 0..w {
                    let mut sum = [0u32; 4];
                    let mut count = 0;

                    for dy in -(params.radius as i32)..=(params.radius as i32) {
                        for dx in -(params.radius as i32)..=(params.radius as i32) {
                            let nx = x + dx;
                            let ny = y + dy;

                            if nx >= 0 && nx < w && ny >= 0 && ny < h {
                                let idx = ((ny * w + nx) * 4) as usize;
                                for c in 0..4 {
                                    sum[c] += temp[idx + c] as u32;
                                }
                                count += 1;
                            }
                        }
                    }

                    let dst = ((y * w + x) * 4) as usize;
                    for c in 0..4 {
                        buf[dst + c] = (sum[c] / count) as u8;
                    }
                }
            }
            temp.copy_from_slice(buf);
        }
    }
}
