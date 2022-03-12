use crate::graphics;

const ROWS: usize = 25;
const COLS: usize = 80;

pub struct Console<'a> {
    fb: &'a graphics::FrameBuffer,
    fg_color: (u8, u8, u8),
    bg_color: (u8, u8, u8),
    buffer: [[char; ROWS]; COLS + 1],
    cursor_row: usize,
    cursor_col: usize,
}

impl<'a> Console<'a> {
    /// コンストラクタ
    pub fn new(fb: &'a graphics::FrameBuffer) -> Self {
        let mut console = Console {
            fb,
            fg_color: graphics::WHITE,
            bg_color: graphics::BLACK,
            buffer: [['\0'; ROWS]; COLS + 1],
            cursor_row: 0,
            cursor_col: 0,
        };
        console.paint_background();
        console
    }

    /// 文字列を出力
    pub fn put_string(&mut self, string: &str) {
        for c in string.chars() {
            if c == '\n' {
                self.new_line();
            } else if self.cursor_col < COLS - 1 {
                graphics::print_font(
                    self.fb,
                    self.cursor_col as u32 * 10,
                    self.cursor_row as u32 * 18,
                    self.fg_color,
                    c,
                );
                self.buffer[self.cursor_row][self.cursor_col] = c;
                self.cursor_col += 1;
            }
        }
    }

    /// コンソール領域を背景色で塗りつぶす
    fn paint_background(&mut self) {
        graphics::paint_rectangle(
            self.fb,
            0,
            0,
            COLS as u32 * 10,
            ROWS as u32 * 18,
            self.bg_color,
        );
    }

    /// 改行
    fn new_line(&mut self) {
        self.cursor_col = 0;
        if self.cursor_row < ROWS - 1 {
            self.cursor_row += 1;
        } else {
            self.paint_background();
            for r in 0..ROWS {
                // 一つ上の行へ移動
                // 行をプリント
            }
            // 最下行をnull文字で埋める
        }
    }
}
