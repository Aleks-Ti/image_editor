//! Blur plugin for image processing application

#![warn(missing_docs)]

use serde::Deserialize;
use std::ffi::CStr;
use std::os::raw::c_char;

#[derive(Deserialize)]
struct Params {
    radius: u32,
    iterations: u32,
}

/// Применяет эффект размытия к изображению на месте.
///
/// Алгоритм выполняет `iterations` проходов размытия с радиусом `radius`.
/// Каждый пиксель заменяется средним значением пикселей в квадратной области
/// размером `(2 * radius + 1) × (2 * radius + 1)`.
///
/// # Параметры
///
/// - `width`, `height`: размеры изображения в пикселях
/// - `data`: указатель на буфер в формате RGBA8 (4 байта на пиксель)
/// - `params`: JSON-строка с параметрами `{"radius": u32, "iterations": u32}`
///   или null (используются значения по умолчанию: radius=1, iterations=1)
///
/// # Безопасность
///
/// Эта функция является FFI-границей. Вызывающая сторона ОБЯЗАНА гарантировать:
///
/// - `data` указывает на валидный, изменяемый буфер размером не менее
///   `width × height × 4` байт в формате RGBA8;
/// - буфер правильно выровнен и остаётся валидным на время выполнения;
/// - `params` — либо null, либо корректная нуль-терминированная C-строка;
/// - функция не вызывается конкурентно для одного и того же буфера.
///
/// # Возврат
///
/// - `0` — успешно обработано;
/// - `-1` — ошибка (переполнение арифметики, невалидные параметры, слишком большой размер изображения, повреждённая память).
///
/// # Предупреждения безопасности
///
/// При переполнении при вычислении размера буфера (`width × height × 4`)
/// в релизной сборке произойдёт wrapping, что приведёт к созданию среза
/// с некорректной длиной и неопределённому поведению (UB) при работе
/// с `unsafe`-кодом. Поэтому используется проверенная арифметика через
/// `.checked_mul()`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    data: *mut u8,
    params: *const c_char,
) -> i32 {
    let total_pixels = width.checked_mul(height).unwrap_or(u32::MAX);
    let len = total_pixels.checked_mul(4).unwrap_or(u32::MAX);

    if len > 0 && data.is_null() {
        return -1;
    }
    let len_usize: usize = match len.try_into() {
        Ok(v) => v,
        Err(_) => return -1,
    };
    let w: isize = match width.try_into() {
        Ok(v) => v,
        Err(_) => return -1, // width > i32::MAX
    };
    let h: isize = match height.try_into() {
        Ok(v) => v,
        Err(_) => return -1, // height > i32::MAX
    };

    // Создание среза из сырых данных
    let buf = unsafe { std::slice::from_raw_parts_mut(data, len_usize) };
    let params_str = if params.is_null() {
        ""
    } else {
        match unsafe { CStr::from_ptr(params) }.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let params: Params = match serde_json::from_str(params_str) {
        Ok(p) => p,
        Err(_) => Params {
            radius: 1,
            iterations: 1,
        },
    };

    let mut temp = buf.to_vec();
    let radius = params.radius as isize;

    for _ in 0..params.iterations {
        for y in 0..h {
            for x in 0..w {
                let mut sum = [0u32; 4];
                let mut count = 0;

                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let nx = x + dx;
                        let ny = y + dy;

                        if nx >= 0 && nx < w && ny >= 0 && ny < h {
                            // Все значения неотрицательны → безопасное преобразование в usize
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
    0
}
