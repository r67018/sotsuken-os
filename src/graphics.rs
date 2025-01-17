use core::mem::MaybeUninit;
use core::ops::AddAssign;
use kernel::FrameBufferConfig;

#[derive(Clone, Copy, Debug)]
pub struct Vector2D<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> AddAssign<Vector2D<T>> for Vector2D<T>
    where T: AddAssign<T>
{
    fn add_assign(&mut self, rhs: Vector2D<T>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PixelColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

fn pixel_at(x: usize, y: usize, config: &FrameBufferConfig) -> *mut u8 {
    unsafe {
        config.frame_buffer.offset(4 * (config.pixels_per_scan_line * (y as u32) + (x as u32)) as isize)
    }
}

pub trait PixelWriter {
    fn write_pixel(&self, x: usize, y: usize, c: PixelColor);

    fn fill_rectangle(&self, pos: Vector2D<usize>, size: Vector2D<usize>, c: PixelColor) {
        for dx in 0..size.x {
            for dy in 0..size.y {
                self.write_pixel(pos.x + dx, pos.y + dy, c);
            }
        }
    }

    fn draw_rectangle(&self, pos: Vector2D<usize>, size: Vector2D<usize>, c: PixelColor) {
        for dx in 0..size.x {
            self.write_pixel(pos.x + dx, pos.y, c);
            self.write_pixel(pos.x + dx, pos.y + size.y - 1, c);
        }
        for dy in 0..size.y {
            self.write_pixel(pos.x, pos.y + dy, c);
            self.write_pixel(pos.x + size.x - 1, pos.y + dy, c);
        }
    }
}

pub struct RGBResv8BitPerColorPixelWriter<'a> {
    pub config: &'a FrameBufferConfig,
}

impl PixelWriter for RGBResv8BitPerColorPixelWriter<'_> {
    fn write_pixel(&self, x: usize, y: usize, c: PixelColor) {
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
    fn write_pixel(&self, x: usize, y: usize, c: PixelColor) {
        let p = pixel_at(x, y, self.config);
        unsafe {
            *p.offset(0) = c.b;
            *p.offset(1) = c.g;
            *p.offset(2) = c.r;
        }
    }
}

pub static mut PIXEL_WRITER: MaybeUninit<&dyn PixelWriter> = MaybeUninit::uninit();

pub fn pixel_writer() -> &'static dyn PixelWriter {
    unsafe {
        PIXEL_WRITER.assume_init()
    }
}
