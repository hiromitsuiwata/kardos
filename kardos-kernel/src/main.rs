#![no_std]
#![no_main]

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
        MyPixelFormat::Rgb => render_example::<RgbPixelWriter>(fb),
        MyPixelFormat::Bgr => render_example::<BgrPixelWriter>(fb),
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

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
