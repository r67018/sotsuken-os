use crate::graphics::{PixelColor, PixelWriter, Vector2D};
use crate::{printk, MOUSE_CURSOR_HEIGHT, MOUSE_CURSOR_SHAPE, MOUSE_CURSOR_WIDTH};
use core::mem::MaybeUninit;

pub static mut MOUSE_CURSOR: MaybeUninit<MouseCursor> = MaybeUninit::uninit();

pub fn mouse_cursor() -> &'static mut MouseCursor<'static> {
    unsafe { MOUSE_CURSOR.assume_init_mut() }
}

pub struct MouseCursor<'a> {
    pixel_writer: &'a dyn PixelWriter,
    erase_color: PixelColor,
    position: Vector2D<i32>
}

impl<'a> MouseCursor<'a> {
    pub fn new(
        writer: &'a dyn PixelWriter,
        erase_color: PixelColor,
        initial_position: Vector2D<i32>
    ) -> Self {
        MouseCursor {
            pixel_writer: writer,
            erase_color,
            position: initial_position
        }
    }
    
    pub fn move_relative(&mut self, displacement: Vector2D<i32>) {
        self.erase();
        self.position += displacement;
        self.draw();
    }
    
    fn draw(&self) {
        for dy in 0..MOUSE_CURSOR_HEIGHT {
            let s = MOUSE_CURSOR_SHAPE[dy];
            for (dx, c) in s.chars().enumerate() {
                let x = self.position.x as usize + dx;
                let y = self.position.y as usize + dy;
                if c == '@' {
                    self.pixel_writer.write_pixel(x, y, PixelColor::new(0, 0, 0));
                } else if c == '.' {
                    self.pixel_writer.write_pixel(x, y, PixelColor::new(255, 255, 255));
                }
            }
        }
    }
    
    fn erase(&self) {
        self.pixel_writer.fill_rectangle(
            // self.position,
            Vector2D::new(self.position.x as usize, self.position.y as usize),
            Vector2D::new(MOUSE_CURSOR_WIDTH, MOUSE_CURSOR_HEIGHT),
            self.erase_color
        )
    }
}

pub extern "C" fn mouse_observer(_buttons: u8, displacement_x: i8, displacement_y: i8) {
    mouse_cursor().move_relative(Vector2D::new(displacement_x as i32, displacement_y as i32));
}
