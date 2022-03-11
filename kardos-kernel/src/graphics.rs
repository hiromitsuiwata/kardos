include!(concat!(env!("OUT_DIR"), "/latin1_font.rs"));

pub const BLACK: (u8, u8, u8) = (0, 0, 0);
pub const WHITE: (u8, u8, u8) = (255, 255, 255);
pub const RED: (u8, u8, u8) = (255, 0, 0);
pub const GREEN: (u8, u8, u8) = (0, 255, 0);
pub const BLUE: (u8, u8, u8) = (0, 0, 255);

// 文字列を描画する.1文字目の座標を指定する.colorはRGBの順番で指定する.
pub fn print_string(fb: &FrameBuffer, x: u32, y: u32, color: (u8, u8, u8), string: &str) {
    for (i, c) in string.chars().enumerate() {
        print_font(fb, x + i as u32 * 10, y, color, c);
    }
}

/// フォントを描画する. colorはRGBの順番で指定する.
pub fn print_font(fb: &FrameBuffer, x: u32, y: u32, color: (u8, u8, u8), c: char) {
    // 配列のインデックスはusizeである必要がある
    let num = c as usize - 1;

    match fb.format {
        MyPixelFormat::Rgb => {
            render_font::<RgbPixelWriter>(fb, x, y, color, &LATIN1_FONT[num]);
        }
        MyPixelFormat::Bgr => {
            render_font::<BgrPixelWriter>(fb, x, y, color, &LATIN1_FONT[num]);
        }
    }
}

pub fn print_example(fb: &FrameBuffer) {
    match fb.format {
        MyPixelFormat::Rgb => {
            render_example::<RgbPixelWriter>(fb);
        }
        MyPixelFormat::Bgr => {
            render_example::<BgrPixelWriter>(fb);
        }
    }
}

#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum MyPixelFormat {
    Rgb,
    Bgr,
}

#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct FrameBuffer {
    pub frame_buffer: *mut u8,
    pub stride: u32,
    pub resolution: (u32, u32), // (horizontal, vertical)
    pub format: MyPixelFormat,
}

trait PixelWriter {
    fn put_pixel(fb: &FrameBuffer, x: u32, y: u32, color: (u8, u8, u8));
}

enum RgbPixelWriter {}

impl PixelWriter for RgbPixelWriter {
    fn put_pixel(fb: &FrameBuffer, x: u32, y: u32, color: (u8, u8, u8)) {
        unsafe {
            let offset = (4 * (fb.stride * y + x)) as usize;
            *fb.frame_buffer.add(offset) = color.0;
            *fb.frame_buffer.add(offset + 1) = color.1;
            *fb.frame_buffer.add(offset + 2) = color.2;
        }
    }
}

enum BgrPixelWriter {}

impl PixelWriter for BgrPixelWriter {
    fn put_pixel(fb: &FrameBuffer, x: u32, y: u32, color: (u8, u8, u8)) {
        unsafe {
            let offset = (4 * (fb.stride * y + x)) as usize;
            *fb.frame_buffer.add(offset) = color.2;
            *fb.frame_buffer.add(offset + 1) = color.1;
            *fb.frame_buffer.add(offset + 2) = color.0;
        }
    }
}

fn render_font<W: PixelWriter>(
    fb: &FrameBuffer,
    x: u32,
    y: u32,
    color: (u8, u8, u8),
    font: &[u16; 18],
) {
    for dy in 0..18 {
        for dx in 0..9 {
            if (font[dy] << dx) & 0b1000000000 != 0 {
                let px = x + dx;
                let py = y + (dy as u32);
                W::put_pixel(fb, px, py, color);
            }
        }
    }
}

fn render_example<W: PixelWriter>(fb: &FrameBuffer) {
    for x in 0..fb.resolution.0 {
        for y in 0..fb.resolution.1 {
            W::put_pixel(fb, x, y, ((x % 256) as u8, (y % 256) as u8, 255));
        }
    }
}
