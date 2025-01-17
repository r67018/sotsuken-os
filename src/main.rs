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
use crate::graphics::{pixel_writer, BGRResv8BitPerColorPixelWriter, PixelColor, RGBResv8BitPerColorPixelWriter, Vector2D, PIXEL_WRITER};

const MOUSE_CURSOR_WIDTH: usize = 15;
const MOUSE_CURSOR_HEIGHT: usize = 24;
const MOUSE_CURSOR_SHAPE: [&str; MOUSE_CURSOR_HEIGHT] = [
    "@              ",
    "@@             ",
    "@.@            ",
    "@..@           ",
    "@...@          ",
    "@....@         ",
    "@.....@        ",
    "@......@       ",
    "@.......@      ",
    "@........@     ",
    "@.........@    ",
    "@..........@   ",
    "@...........@  ",
    "@............@ ",
    "@......@@@@@@@@",
    "@......@       ",
    "@....@@.@      ",
    "@...@ @.@      ",
    "@..@   @.@     ",
    "@.@    @.@     ",
    "@@      @.@    ",
    "@       @.@    ",
    "         @.@   ",
    "         @@@   ",
];
const DESKTOP_BG_COLOR: PixelColor = PixelColor {
    r: 0,
    g: 0,
    b: 255,
};

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
            pixel_writer(),
            PixelColor::new(255, 255, 255),
            DESKTOP_BG_COLOR,
        ));
    }

    let frame_width = frame_buffer_config.horizontal_resolution as usize;
    let frame_height = frame_buffer_config.vertical_resolution as usize;
    pixel_writer().fill_rectangle(Vector2D::new(0, 0), Vector2D::new(frame_width, frame_height), DESKTOP_BG_COLOR);
    pixel_writer().fill_rectangle(Vector2D::new(0, frame_height - 50), Vector2D::new(frame_width, 50), PixelColor::new(1, 8, 17));
    pixel_writer().fill_rectangle(Vector2D::new(0, frame_height - 50), Vector2D::new(frame_width / 5, 50), PixelColor::new(80, 80, 80));
    pixel_writer().draw_rectangle(Vector2D::new(10, frame_height - 40), Vector2D::new(30, 30), PixelColor::new(160, 160, 160));

    // マウスカーソルの描画
    for dy in 0..MOUSE_CURSOR_HEIGHT {
        let s = MOUSE_CURSOR_SHAPE[dy];
        for (dx, c) in s.chars().enumerate() {
            if c == '@' {
                pixel_writer().write_pixel(200 + dx, 100 + dy, PixelColor::new(0, 0, 0));
            } else if c == '.' {
                pixel_writer().write_pixel(200 + dx, 100 + dy, PixelColor::new(255, 255, 255));
            }
        }
    }

    printk!("Welcome to GotOS!");

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
