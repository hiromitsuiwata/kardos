#![no_std]
#![no_main]

mod graphics;

#[no_mangle]
pub extern "sysv64" fn kernel_main(fb: &graphics::FrameBuffer) {
    graphics::print_example(fb);

    graphics::print_string(fb, 0, 0, graphics::BLACK, "Hello, world!");
    graphics::print_string(fb, 0, 20, graphics::RED, "Hello, world!");

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
