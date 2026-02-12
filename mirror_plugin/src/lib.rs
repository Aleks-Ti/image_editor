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

/// Применяет горизонтальное и/или вертикальное зеркалирование к изображению на месте.
///
/// Изображение должно быть в формате RGBA8 (4 байта на пиксель). Функция создаёт
/// временную копию исходных данных и записывает результат обратно в тот же буфер.
///
/// # Параметры
///
/// - `width`, `height`: размеры изображения в пикселях (максимум 2 147 483 647)
/// - `data`: указатель на буфер в формате RGBA8 (4 байта на пиксель)
/// - `params`: JSON-строка с параметрами `{"horizontal": bool, "vertical": bool}`
///   или null (используются значения по умолчанию: оба флага = false)
///
/// # Безопасность
///
/// Эта функция является FFI-границей. Вызывающая сторона ОБЯЗАНА гарантировать:
///
/// - `data` указывает на валидный, изменяемый буфер размером не менее
///   `width × height × 4` байт в формате RGBA8;
/// - буфер правильно выровнен и остаётся валидным на время выполнения;
/// - `params` — либо null, либо корректная нуль-терминированная C-строка в UTF-8;
/// - функция не вызывается конкурентно для одного и того же буфера.
///
/// # Возврат
///
/// - `0` — успешно обработано;
/// - `-1` — ошибка (переполнение арифметики, слишком большой размер изображения,
///   невалидная кодировка параметров, null-указатель при ненулевом буфере).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    data: *mut u8,
    params: *const c_char,
) -> i32 {
    let total_pixels = width.checked_mul(height).unwrap_or(u32::MAX);
    let buffer_size = total_pixels.checked_mul(4).unwrap_or(u32::MAX);

    let len: usize = match buffer_size.try_into() {
        Ok(v) => v,
        Err(_) => return -1, // буфер слишком велик для текущей архитектуры
    };
    if len > 0 && data.is_null() {
        return -1;
    }
    let w: usize = match width.try_into() {
        Ok(v) => v,
        Err(_) => return -1,
    };
    let h: usize = match height.try_into() {
        Ok(v) => v,
        Err(_) => return -1,
    };

    // Создание среза из сырых данных
    let slice = unsafe { std::slice::from_raw_parts_mut(data, len) };
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
            horizontal: false,
            vertical: false,
        },
    };

    let copy = slice.to_vec();
    for y in 0..h {
        for x in 0..w {
            let src_x = if params.horizontal {
                w.saturating_sub(1).saturating_sub(x)
            } else {
                x
            };
            let src_y = if params.vertical {
                h.saturating_sub(1).saturating_sub(y)
            } else {
                y
            };
            let dst = (y * w + x) * 4;
            let src = (src_y * w + src_x) * 4;
            slice[dst..dst + 4].copy_from_slice(&copy[src..src + 4]);
        }
    }
    0
}
