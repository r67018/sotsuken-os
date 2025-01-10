use crate::font::{write_ascii, write_string};
use crate::graphics::{PixelColor, PixelWriter};

// コンソールの行数
const ROWS: usize = 25;
// コンソールの列数
const COLUMNS: usize = 80;

pub struct Console<'a, T>
    where T: PixelWriter + ?Sized
{
    writer: &'a T,
    buffer: [[char; COLUMNS + 1]; ROWS], // ヌル文字用に1バイト多く割り当てる
    cursor_row: usize,
    cursor_column: usize,
    fg_color: PixelColor,
    bg_color: PixelColor,
}

impl<'a, T> Console<'a, T>
    where T: PixelWriter + ?Sized
{
    pub fn new(writer: &'a T, fg_color: PixelColor, bg_color: PixelColor) -> Self {
        Self {
            writer,
            buffer: [['\0'; COLUMNS + 1]; ROWS],
            cursor_row: 0,
            cursor_column: 0,
            fg_color,
            bg_color,
        }
    }

    // コンソールに文字列を出力する関数
    pub fn put_string(&mut self, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                self.new_line();
            } else if self.cursor_column < (COLUMNS - 1) {
                write_ascii(
                    self.writer,
                    8 * self.cursor_column,
                    16 * self.cursor_row,
                    c,
                    &self.fg_color
                );
                self.buffer[self.cursor_row][self.cursor_column] = c;
                self.cursor_column += 1;
            }
        }
    }

    // 改行用
    fn new_line(&mut self) {
        // 列のカーソルを0にセット
        self.cursor_column = 0;

        // 最終行じゃない場合は行のカーソルを1増やして終了
        if self.cursor_row < (ROWS - 1) {
            self.cursor_row += 1;
            return;
        }

        // 最終行まで到達した場合
        // 画面を背景色で塗り潰す
        for y in 0..(16 * ROWS) {
            for x in 0..(8 * COLUMNS) {
                self.writer.write(x, y, &self.bg_color);
            }
        }
        // 各行を1つずらしながら描画する
        for row in 0..(ROWS - 1) {
            self.buffer.copy_within((row+1)..=(row+1), row);
            write_string(self.writer, 0, 16 * row, &self.buffer[row], &self.fg_color)
        }
        // 最終行をヌル文字で埋める
        self.buffer[ROWS - 1].fill('\0');
    }
}
