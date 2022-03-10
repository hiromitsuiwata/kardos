#![no_std]
#![no_main]

include!(concat!(env!("OUT_DIR"), "/latin1_font.rs"));

pub const BLACK: (u8, u8, u8) = (0, 0, 0);
pub const WHITE: (u8, u8, u8) = (255, 255, 255);
pub const RED: (u8, u8, u8) = (255, 0, 0);
pub const GREEN: (u8, u8, u8) = (0, 255, 0);
pub const BLUE: (u8, u8, u8) = (0, 0, 255);

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

#[no_mangle]
pub extern "sysv64" fn kernel_main(fb: &FrameBuffer) {
    match fb.format {
        MyPixelFormat::Rgb => {
            render_example::<RgbPixelWriter>(fb);
        }
        MyPixelFormat::Bgr => {
            render_example::<BgrPixelWriter>(fb);
        }
    }

    print_font(fb, 60, 20, BLACK, 'g');
    print_font(fb, 70, 20, RED, '[');
    print_font(fb, 80, 20, GREEN, '\\');
    print_font(fb, 90, 20, BLUE, 'y');
    print_font(fb, 100, 20, WHITE, 'm');
    print_string(fb, 150, 20, BLACK, "Hello, world!");

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
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

fn render_example<W: PixelWriter>(fb: &FrameBuffer) {
    for x in 0..fb.resolution.0 {
        for y in 0..fb.resolution.1 {
            W::put_pixel(fb, x, y, ((x % 256) as u8, (y % 256) as u8, 255));
        }
    }

    for x in 50..250 {
        for y in 50..150 {
            W::put_pixel(fb, x, y, (255, 0, 0));
        }
    }
}

fn print_string(fb: &FrameBuffer, x: u32, y: u32, color: (u8, u8, u8), string: &str) {
    for (i, c) in string.chars().enumerate() {
        print_font(fb, x + i as u32 * 8, y, color, c);
    }
}

/// フォントを描画する. colorはRGBの順番で指定する.
fn print_font(fb: &FrameBuffer, x: u32, y: u32, color: (u8, u8, u8), c: char) {
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

fn render_font<W: PixelWriter>(
    fb: &FrameBuffer,
    x: u32,
    y: u32,
    color: (u8, u8, u8),
    font: &[u8; 14],
) {
    // 0, 1, 2, ..., 13までのループであることに注意
    for dy in 0..14 {
        for dx in 0..8 {
            if (font[dy] << dx) & 0x80 != 0 {
                let px = x + dx;
                let py = y + (dy as u32);
                W::put_pixel(fb, px, py, color);
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
