use core::mem::MaybeUninit;
use kernel::FrameBufferConfig;

pub static mut PIXEL_WRITER: MaybeUninit<&dyn PixelWriter> = MaybeUninit::uninit();

pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

fn pixel_at(x: usize, y: usize, config: &FrameBufferConfig) -> *mut u8 {
    unsafe {
        config.frame_buffer.offset(4 * (config.pixels_per_scan_line * (y as u32) + (x as u32)) as isize)
    }
}

pub trait PixelWriter {
    fn write(&self, x: usize, y: usize, c: &PixelColor);
    fn write_pixel(&self, x: usize, y: usize, c: &PixelColor);
}

pub struct RGBResv8BitPerColorPixelWriter<'a> {
    pub config: &'a FrameBufferConfig,
}

impl PixelWriter for RGBResv8BitPerColorPixelWriter<'_> {
    fn write_pixel(&self, x: usize, y: usize, c: &PixelColor) {
        let p = pixel_at(x, y, self.config);
        unsafe {
            *p.offset(0) = c.r;
            *p.offset(1) = c.g;
            *p.offset(2) = c.b;
        }
    }
}

pub struct BGRResv8BitPerColorPixelWriter<'a> {
    pub config: &'a FrameBufferConfig,
}

impl PixelWriter for BGRResv8BitPerColorPixelWriter<'_> {
    fn write_pixel(&self, x: usize, y: usize, c: &PixelColor) {
        let p = pixel_at(x, y, self.config);
        unsafe {
            *p.offset(0) = c.b;
            *p.offset(1) = c.g;
            *p.offset(2) = c.r;
        }
    }
}