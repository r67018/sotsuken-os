#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;
use kernel::{FrameBufferConfig, PixelFormat_kPixelBGRResv8BitPerColor, PixelFormat_kPixelRGBResv8BitPerColor};

#[no_mangle]
pub extern "C" fn KernelMain(frame_buffer_config: &mut FrameBufferConfig) -> ! {
    for x in 0..frame_buffer_config.horizontal_resolution {
        for y in 0..frame_buffer_config.vertical_resolution {
            write_pixel(frame_buffer_config, x, y, &PixelColor {
                r: 255,
                g: 255,
                b: 255,
            });
        }
    }
    for x in 0..200 {
        for y in 0..100 {
            write_pixel(frame_buffer_config, 100 + x, 100 + y, &PixelColor {
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

fn write_pixel(config: &mut FrameBufferConfig, x: u32, y: u32, c: &PixelColor) -> bool {
    let pixel_position = config.pixels_per_scan_line * y + x;
    if config.pixel_format == PixelFormat_kPixelRGBResv8BitPerColor {
        unsafe {
            let p = config.frame_buffer.offset(4 * pixel_position as isize);
            *p.offset(0) = c.r;
            *p.offset(1) = c.g;
            *p.offset(2) = c.b;
        }
    } else if config.pixel_format == PixelFormat_kPixelBGRResv8BitPerColor {
        unsafe {
            let p = config.frame_buffer.offset(4 * pixel_position as isize);
            *p.offset(0) = c.b;
            *p.offset(1) = c.g;
            *p.offset(2) = c.r;
        }
    } else {
        return false;
    }
    true
}

