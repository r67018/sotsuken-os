#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]

mod graphics;
mod font;
mod console;

use core::arch::asm;
use core::panic::PanicInfo;
use kernel::{FrameBufferConfig, PixelFormat_kPixelBGRResv8BitPerColor, PixelFormat_kPixelRGBResv8BitPerColor};
use crate::console::{Console, CONSOLE};
use crate::graphics::{BGRResv8BitPerColorPixelWriter, PixelColor, RGBResv8BitPerColorPixelWriter, PIXEL_WRITER};

#[no_mangle]
pub extern "C" fn KernelMain(frame_buffer_config: &'static mut FrameBufferConfig) -> ! {
    static mut RGB_PIXEL_WRITER: Option<RGBResv8BitPerColorPixelWriter> = None;
    static mut BGR_PIXEL_WRITER: Option<BGRResv8BitPerColorPixelWriter> = None;

    // グローバル変数を初期化
    // PixelFormatによってPixelWriterを切り替える
    // 配置newのやり方がわからないので、両方インスタンス化してそれを参照する
    unsafe {
        RGB_PIXEL_WRITER = Some(RGBResv8BitPerColorPixelWriter {
            config: frame_buffer_config,
        });
        BGR_PIXEL_WRITER = Some(BGRResv8BitPerColorPixelWriter {
            config: frame_buffer_config,
        });

        PIXEL_WRITER.write(match frame_buffer_config.pixel_format {
            PixelFormat_kPixelRGBResv8BitPerColor => RGB_PIXEL_WRITER.as_ref().unwrap(),
            PixelFormat_kPixelBGRResv8BitPerColor => BGR_PIXEL_WRITER.as_ref().unwrap(),
            _ => panic!("unsupported pixel format: {}", frame_buffer_config.pixel_format),
        });

        CONSOLE.write(Console::new(
            PIXEL_WRITER.assume_init(),
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
        ));
    }

    // 画面を白で塗りつぶす
    for x in 0..frame_buffer_config.horizontal_resolution {
        for y in 0..frame_buffer_config.vertical_resolution {
            unsafe {
                PIXEL_WRITER.assume_init().write_pixel(x as usize, y as usize, &PixelColor {
                    r: 255,
                    g: 255,
                    b: 255,
                });
            }
        }
    }

    for i in 0..27 {
        printk!("Line: {}\n", i);
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
