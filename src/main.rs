#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]

use core::arch::asm;
use core::panic::PanicInfo;
use kernel::{FrameBufferConfig, PixelFormat_kPixelBGRResv8BitPerColor, PixelFormat_kPixelRGBResv8BitPerColor};

#[no_mangle]
pub extern "C" fn KernelMain(frame_buffer_config: &mut FrameBufferConfig) -> ! {
    // PixelFormatによってPixelWriterを切り替える
    // 配置newのやり方がわからないので、両方インスタンス化してそれを参照する
    let mut rgb_pixel_writer = RGBResv8BitPerColorPixelWriter {
        config: frame_buffer_config,
    };
    let mut bgr_pixel_writer = BGRResv8BitPerColorPixelWriter {
        config: frame_buffer_config,
    };
    let pixel_writer: &mut dyn PixelWriter = match frame_buffer_config.pixel_format {
        PixelFormat_kPixelRGBResv8BitPerColor => &mut rgb_pixel_writer,
        PixelFormat_kPixelBGRResv8BitPerColor => &mut bgr_pixel_writer,
        _ => panic!("unsupported pixel format: {}", frame_buffer_config.pixel_format),
    };

    for x in 0..frame_buffer_config.horizontal_resolution {
        for y in 0..frame_buffer_config.vertical_resolution {
            pixel_writer.write(x, y, &PixelColor {
                r: 255,
                g: 255,
                b: 255,
            });
        }
    }
    for x in 0..200 {
        for y in 0..100 {
            pixel_writer    .write(x, y, &PixelColor {
                r: 0,
                g: 255,
                b: 0,
            });
        }
    }
    loop {
        unsafe { asm!("hlt"); }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { asm!("hlt"); }
    }
}

struct PixelColor {
    r: u8,
    g: u8,
    b: u8,
}

fn pixel_at(x: u32, y: u32, config: &FrameBufferConfig) -> *mut u8 {
    unsafe {
        config.frame_buffer.offset(4 * (config.pixels_per_scan_line * y + x) as isize)
    }
}

trait PixelWriter {
    fn write(&self, x: u32, y: u32, c: &PixelColor);
}

struct RGBResv8BitPerColorPixelWriter<'a> {
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

struct BGRResv8BitPerColorPixelWriter<'a> {
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
