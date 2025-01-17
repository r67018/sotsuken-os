use crate::graphics::{PixelColor, PixelWriter};

extern "C" {
    static _binary_hankaku_bin_start: u8;
    static _binary_hankaku_bin_end: u8;
    static _binary_hankaku_bin_size: u8;
}

pub fn get_font(c: char) -> Option<*const u8> {
    let index = 16 * c as isize;
    unsafe {
        let size = &_binary_hankaku_bin_size as *const u8;
        if index >= size as isize {
            None
        } else {
            Some((&_binary_hankaku_bin_start as *const u8).offset(index))
        }
    }
}

pub fn write_ascii<T>(writer: &T, x: usize, y: usize, c: char, color: &PixelColor)
    where T: PixelWriter + ?Sized,
{
    // フォントが存在するなら描画
    if let Some(font) = get_font(c) {
        for dy in 0..16 {
            for dx in 0..8 {
                unsafe {
                    let is_set = (*font.offset(dy as isize) << dx) & 0x80;
                    if is_set == 0x80 {
                        writer.write_pixel(x + dx, y + dy, color);
                    }
                }
            }
        }
    }
}

pub fn write_string<T, S>(writer: &T, x: usize, y: usize, s: &S, color: &PixelColor)
    where T: PixelWriter + ?Sized,
          for<'a> &'a S: IntoIterator<Item = &'a char>,
{
    for  (i, c) in s.into_iter().enumerate() {
        write_ascii(writer, x + 8 * i, y, *c, color);
    }
}
