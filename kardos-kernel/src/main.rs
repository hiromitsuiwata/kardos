#![no_std]
#![no_main]

include!(concat!(env!("OUT_DIR"), "/latin1_font.rs"));

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
            render_font::<RgbPixelWriter>(fb, 10, 10, (0, 0, 0), &LATIN1_FONT[0x41]);
            render_font::<RgbPixelWriter>(fb, 20, 10, (0, 0, 0), &LATIN1_FONT[0x42]);
            render_font::<RgbPixelWriter>(fb, 30, 10, (0, 0, 0), &LATIN1_FONT[0x43]);
        }
        MyPixelFormat::Bgr => {
            render_example::<BgrPixelWriter>(fb);
            render_font::<BgrPixelWriter>(fb, 10, 20, (0, 0, 0), &LATIN1_FONT[0x50]);
            render_font::<BgrPixelWriter>(fb, 20, 20, (0, 0, 0), &LATIN1_FONT[0x51]);
            render_font::<BgrPixelWriter>(fb, 30, 20, (0, 0, 0), &LATIN1_FONT[0x52]);
            render_font::<BgrPixelWriter>(fb, 40, 20, (0, 0, 0), &LATIN1_FONT[0x53]);
        }
    }
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

fn render_font<W: PixelWriter>(
    fb: &FrameBuffer,
    x: u32,
    y: u32,
    color: (u8, u8, u8),
    font: &[u8; 14],
) {
    for dy in 0..13 {
        for dx in 0..7 {
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
