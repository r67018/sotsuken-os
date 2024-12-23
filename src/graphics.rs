use kernel::FrameBufferConfig;

pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

fn pixel_at(x: u32, y: u32, config: &FrameBufferConfig) -> *mut u8 {
    unsafe {
        config.frame_buffer.offset(4 * (config.pixels_per_scan_line * y + x) as isize)
    }
}

pub trait PixelWriter {
    fn write(&self, x: u32, y: u32, c: &PixelColor);
}

pub struct RGBResv8BitPerColorPixelWriter<'a> {
    pub config: &'a FrameBufferConfig,
}

impl PixelWriter for RGBResv8BitPerColorPixelWriter<'_> {
    fn write(&self, x: u32, y: u32, c: &PixelColor) {
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
    fn write(&self, x: u32, y: u32, c: &PixelColor) {
        let p = pixel_at(x, y, self.config);
        unsafe {
            *p.offset(0) = c.b;
            *p.offset(1) = c.g;
            *p.offset(2) = c.r;
        }
    }
}