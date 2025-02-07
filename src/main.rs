#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]

extern crate alloc;

mod graphics;
mod font;
mod console;
mod pci;
mod usb;
mod mouse;

use core::arch::asm;
use core::panic::PanicInfo;
use kernel::{FrameBufferConfig, PixelFormat_kPixelBGRResv8BitPerColor, PixelFormat_kPixelRGBResv8BitPerColor};
use crate::console::{Console, CONSOLE};
use crate::graphics::{pixel_writer, BGRResv8BitPerColorPixelWriter, PixelColor, RGBResv8BitPerColorPixelWriter, Vector2D, PIXEL_WRITER};
use crate::mouse::{mouse_cursor, MouseCursor, MOUSE_CURSOR};
use crate::pci::Device;

/// ヒープ管理用のアロケータ
#[global_allocator]
static ALLOCATOR: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();

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
    // ヒープを初期化
    init_heap();

    // グローバル変数を初期化
    init_global_variables(frame_buffer_config);

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

    printk!("Welcome to GotOS!\n");

    // PCIデバイスをスキャン
    let mut pci = pci::PciBus::new();
    if let Err(err) = pci.scan_all_bus() {
        printk!("{}", err);
    }

    // Intel製を優先してxHC(USB3.0用のコントローラ)を探す
    let mut xhc_dev: Option<&Device> = None;
    for i in 0..pci.num_device {
        let dev = &pci.devices()[i];
        // ベースクラス 0x0c: シリアルバスのコントローラ全体
        // サブクラス 0x03： USBコントローラ
        // インターフェース 0x30: xHCI
        if dev.class_code.match_base_sub_interface(0x0c, 0x03, 0x30) {
            xhc_dev = Some(dev);

            // ベンダIDがインテル社のIDなら終了
            let vendor_id = pci::read_vendor_id(dev.bus, dev.device, dev.function);
            if vendor_id == 0x8086 {
                break;
            }
        }
    }
    if let Some(xhc_dev) = xhc_dev {
        printk!("xHC has been found: {}.{}.{}\n", xhc_dev.bus, xhc_dev.device, xhc_dev.function);

        // BAR0レジスタの読み取り
        let xhc_bar = pci::read_bar(xhc_dev, 0).expect("Failed to fetch BAR0 register.");
        let xhc_mmio_base = xhc_bar & !0xf; // 下位4ビットはBARのフラグなのでマスクする
        printk!("xHC mmio_base = {:08x}\n", xhc_mmio_base);
        
        let xhc = usb::xhci::Controller::new(xhc_mmio_base);
        if pci::read_vendor_id(xhc_dev.bus, xhc_dev.device, xhc_dev.function) == 0x8086 {
            usb::xhci::switch_ehci2xhci(&pci, &xhc_dev);
        }
        xhc.initialize();
        printk!("xHC starting...\n");
        xhc.run();
        
        xhc.configure_ports();
        printk!("xHC ports configured\n");
        xhc.process_event();
    } else {
        printk!("xHC has not found\n");
    }

    loop {
        unsafe { asm!("hlt"); }
    }
}

fn init_heap() {
    // 一旦適当に領域を割り当てる
    let heap_start = 0x0100_0000;
    let heap_end = 0x0200_0000;
    let heap_size = heap_end - heap_start;
    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
}

fn init_global_variables(frame_buffer_config: &'static FrameBufferConfig) {
    static mut RGB_PIXEL_WRITER: Option<RGBResv8BitPerColorPixelWriter> = None;
    static mut BGR_PIXEL_WRITER: Option<BGRResv8BitPerColorPixelWriter> = None;

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
        
        MOUSE_CURSOR.write(MouseCursor::new(
            pixel_writer(),
            DESKTOP_BG_COLOR,
            Vector2D::new(300, 200),
        ));
        usb::xhci::set_mouse_observer(mouse::mouse_observer);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { asm!("hlt"); }
    }
}
