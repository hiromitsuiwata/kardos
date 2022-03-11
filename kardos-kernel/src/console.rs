use crate::graphics;

const ROWS: usize = 25;
const COLS: usize = 80;

pub struct Console<'a> {
    fb: &'a graphics::FrameBuffer,
    color: (u8, u8, u8),
    buffer: [[char; ROWS]; COLS + 1],
    cursor_row: usize,
    cursor_col: usize,
}

impl<'a> Console<'a> {
    /// コンストラクタ
    pub fn new(fb: &'a graphics::FrameBuffer, color: (u8, u8, u8)) -> Self {
        let console = Console {
            fb,
            color: graphics::BLACK,
            buffer: [[' '; ROWS]; COLS + 1],
            cursor_row: 0,
            cursor_col: 0,
        };
        console
    }

    /// 文字列を出力
    pub fn put_string(&mut self, string: &str) {
        graphics::print_string(
            self.fb,
            self.cursor_col as u32 * 10,
            self.cursor_row as u32 * 10,
            self.color,
            string,
        );
    }

    /// 改行
    fn new_line(&mut self) {}
}
