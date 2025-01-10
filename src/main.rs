#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]

mod graphics;
mod font;
mod console;

use core::arch::asm;
use core::panic::PanicInfo;
use kernel::{FrameBufferConfig, PixelFormat_kPixelBGRResv8BitPerColor, PixelFormat_kPixelRGBResv8BitPerColor};
use crate::console::Console;
use crate::graphics::{BGRResv8BitPerColorPixelWriter, PixelColor, PixelWriter, RGBResv8BitPerColorPixelWriter};

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
            pixel_writer.write(x as usize, y as usize, &PixelColor {
                r: 255,
                g: 255,
                b: 255,
            });
        }
    }

    let mut console = Console::new(
        pixel_writer,
        PixelColor {
            r: 0,
            g: 0,
            b: 0,
        },
        PixelColor {
            r: 255,
            g: 255,
            b: 255,
        }
    );
    console.put_string("Line 1\n");
    console.put_string("Line 2\n");
    console.put_string("Line 3\n");
    console.put_string("Line 4\n");
    console.put_string("Line 5\n");
    console.put_string("Line 6\n");
    console.put_string("Line 7\n");
    console.put_string("Line 8\n");
    console.put_string("Line 9\n");
    console.put_string("Line 10\n");
    console.put_string("Line 11\n");
    console.put_string("Line 12\n");
    console.put_string("Line 13\n");
    console.put_string("Line 14\n");
    console.put_string("Line 15\n");
    console.put_string("Line 16\n");
    console.put_string("Line 17\n");
    console.put_string("Line 18\n");
    console.put_string("Line 19\n");
    console.put_string("Line 20\n");
    console.put_string("Line 21\n");
    console.put_string("Line 22\n");
    console.put_string("Line 23\n");
    console.put_string("Line 24\n");
    console.put_string("Line 25\n");
    console.put_string("Line 26\n");

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
