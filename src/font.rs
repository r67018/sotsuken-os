use crate::graphics::{PixelColor, PixelWriter};

const FONT_A: [u8; 16] = [
    0b00000000,
    0b00011000,
    0b00011000,
    0b00011000,
    0b00011000,
    0b00100100,
    0b00100100,
    0b00100100,
    0b00100100,
    0b01111110,
    0b01000010,
    0b01000010,
    0b01000010,
    0b11100111,
    0b00000000,
    0b00000000,
];

pub fn write_ascii<T>(writer: &T, x: u32, y: u32, c: char, color: &PixelColor)
    where T: PixelWriter + ?Sized,
{
    if c != 'A' {
        return;
    }
    for dy in 0..16u32 {
        for dx in 0..8u32 {
            let is_set = (FONT_A[dy as usize] << dx) & 0x80;
            if is_set == 0x80 {
                writer.write(x + dx, y + dy, color);
            }
        }
    }
}
