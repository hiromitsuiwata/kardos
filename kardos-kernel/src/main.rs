#![no_std]
#![no_main]

mod console;
pub mod graphics;

#[no_mangle]
pub extern "sysv64" fn kernel_main(fb: &graphics::FrameBuffer) {
    graphics::print_example(fb);

    graphics::print_string(fb, 0, 100, graphics::WHITE, "Hello, world from graphics");
    console::Console::new(fb, graphics::BLACK).put_string("Hello, world from console");

    loop {
        unsafe {
            core::arch::asm!("hlt");
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
