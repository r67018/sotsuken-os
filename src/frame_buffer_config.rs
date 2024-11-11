// #[repr(C)]
// #[derive(Eq, PartialEq)]
// pub enum PixelFormat {
//     PixelRGBResv8BitPerColor,
//     PixelBGRResv8BitPerColor,
// }
//
// pub struct FrameBufferConfig {
//     pub frame_buffer: *mut u8,
//     pub pixels_per_scan_line: u32,
//     pub horizontal_resolution: u32,
//     pub vertical_resolution: u32,
//     pub pixel_format: PixelFormat,
// }

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));