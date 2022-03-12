use crate::graphics;

// コンソールの縦横文字数
const ROWS: usize = 30;
const COLS: usize = 50;

// フォント１文字の縦横ピクセル数
const FONT_DX: u32 = 10;
const FONT_DY: u32 = 18;

pub struct Console<'a> {
    fb: &'a graphics::FrameBuffer,
    fg_color: (u8, u8, u8),
    bg_color: (u8, u8, u8),
    buffer: [[char; COLS]; ROWS],
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
            buffer: [['\0'; COLS]; ROWS],
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
                    self.cursor_col as u32 * FONT_DX,
                    self.cursor_row as u32 * FONT_DY,
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
            COLS as u32 * FONT_DX,
            ROWS as u32 * FONT_DY,
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
            for row_index in 0..ROWS - 1 {
                // 一つ上の行へ移動
                self.buffer[row_index] = self.buffer[row_index + 1];

                // 行をプリント
                for col_index in 0..COLS {
                    if self.buffer[row_index][col_index] == '\0' {
                        break;
                    }
                    self.print_font_from_buffer(row_index, col_index);
                }
            }
            // バッファの最下行をnull文字で埋める
            self.buffer[ROWS - 1] = [' '; COLS];
        }
    }

    fn print_font_from_buffer(&mut self, row_index: usize, col_index: usize) {
        graphics::print_font(
            self.fb,
            col_index as u32 * FONT_DX,
            row_index as u32 * FONT_DY,
            self.fg_color,
            self.buffer[row_index][col_index],
        );
    }
}
