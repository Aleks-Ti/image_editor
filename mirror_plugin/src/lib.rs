//! Mirror plugin for image processing application

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
/// # Safety
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
    let total_pixels = match width.checked_mul(height) {
        Some(v) => v,
        None => return -1,
    };

    let buffer_size = match total_pixels.checked_mul(4) {
        Some(v) => v,
        None => return -1, // переполнение при умножении на 4 канала
    };

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    /// Вспомогательная функция для безопасного вызова FFI-функции из тестов
    unsafe fn call_process_image(
        width: u32,
        height: u32,
        data: &mut [u8],
        params_json: Option<&str>,
    ) -> i32 {
        let params_ptr = match params_json {
            Some(json) => CString::new(json).unwrap().into_raw() as *const c_char,
            None => std::ptr::null(),
        };

        let result = unsafe { process_image(width, height, data.as_mut_ptr(), params_ptr) };

        // Освобождаем память CString, если она была создана
        if !params_ptr.is_null() {
            let _ = unsafe { CString::from_raw(params_ptr as *mut c_char) };
        }

        result
    }

    #[test]
    fn test_mirror_horizontal_2x1() {
        // 2 пикселя в строке: [красный, синий] → после зеркалирования → [синий, красный]
        let mut data = vec![
            255, 0, 0, 255, // красный пиксель (0,0)
            0, 0, 255, 255, // синий пиксель (1,0)
        ];

        let result = unsafe {
            call_process_image(
                2,
                1,
                &mut data,
                Some(r#"{"horizontal": true, "vertical": false}"#),
            )
        };

        assert_eq!(result, 0, "Функция должна завершиться успешно");
        assert_eq!(
            data,
            vec![
                0, 0, 255,
                255, // синий пиксель (теперь на месте 0,0)
                255, 0, 0,
                255, // красный пиксель (теперь на месте 1,0)
            ],
            "Горизонтальное зеркалирование должно поменять пиксели местами"
        );
    }
    #[test]
    fn test_mirror_vertical_1x2() {
        let mut data = vec![
            0, 255, 0, 255, // зелёный пиксель (0,0)
            255, 255, 0, 255, // жёлтый пиксель (0,1)
        ];

        let result = unsafe {
            call_process_image(
                1,
                2,
                &mut data,
                Some(r#"{"horizontal": false, "vertical": true}"#),
            )
        };

        assert_eq!(result, 0, "Функция должна завершиться успешно");
        assert_eq!(
            data,
            vec![
                255, 255, 0,
                255, // жёлтый пиксель (теперь на месте 0,0)
                0, 255, 0,
                255, // зелёный пиксель (теперь на месте 0,1)
            ],
            "Вертикальное зеркалирование должно поменять пиксели местами"
        );
    }

    #[test]
    fn test_mirror_both_2x2() {
        let mut data = vec![
            255, 0, 0, 255, // красный (0,0)
            0, 255, 0, 255, // зелёный (1,0)
            0, 0, 255, 255, // синий (0,1)
            255, 255, 0, 255, // жёлтый (1,1)
        ];

        let result = unsafe {
            call_process_image(
                2,
                2,
                &mut data,
                Some(r#"{"horizontal": true, "vertical": true}"#),
            )
        };

        assert_eq!(result, 0, "Функция должна завершиться успешно");
        assert_eq!(
            data,
            vec![
                255, 255, 0,
                255, // жёлтый (теперь в левом верхнем углу)
                0, 0, 255, 255, // синий
                0, 255, 0, 255, // зелёный
                255, 0, 0,
                255, // красный (теперь в правом нижнем углу)
            ],
            "Двойное зеркалирование должно инвертировать изображение по обоим осям"
        );
    }

    #[test]
    fn test_overflow_prevention() {
        // Передаём максимальные значения — функция должна вернуть ошибку, а не паниковать
        let mut dummy = [0u8; 4];

        let result =
            unsafe { process_image(u32::MAX, u32::MAX, dummy.as_mut_ptr(), std::ptr::null()) };

        assert_eq!(
            result, -1,
            "При переполнении должна возвращаться ошибка (-1), а не происходить паника"
        );
    }

    #[test]
    fn test_zero_size_image() {
        // Пустое изображение (0×0) — корректный случай, не должен вызывать ошибок
        let mut dummy = Vec::<u8>::new();

        let result = unsafe { process_image(0, 0, dummy.as_mut_ptr(), std::ptr::null()) };

        assert_eq!(
            result, 0,
            "Пустое изображение (0×0) должно обрабатываться без ошибок"
        );
    }
}
