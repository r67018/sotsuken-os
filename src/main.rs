#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]

mod graphics;
mod font;

use core::arch::asm;
use core::panic::PanicInfo;
use kernel::{FrameBufferConfig, PixelFormat_kPixelBGRResv8BitPerColor, PixelFormat_kPixelRGBResv8BitPerColor};
use crate::font::write_ascii;
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

    for i in 0..26 {
        write_ascii(pixel_writer, 50 + 8 * i, 50, (b'A' + i as u8) as char, &PixelColor {
            r: 0,
            g: 0,
            b: 0,
        });
    }
    // write_ascii(pixel_writer, 50, 50, 'O', &PixelColor {
    //     r: 0,
    //     g: 0,
    //     b: 0,
    // });
    // write_ascii(pixel_writer, 58, 50, 'K', &PixelColor {
    //     r: 0,
    //     g: 0,
    //     b: 0,
    // });

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
