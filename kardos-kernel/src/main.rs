#![no_std]
#![no_main]

mod console;
pub mod graphics;

#[no_mangle]
pub extern "sysv64" fn kernel_main(fb: &graphics::FrameBuffer) {
    graphics::print_example(fb);

    graphics::print_string(fb, 0, 100, graphics::WHITE, "Hello, world from graphics");
    let mut console = console::Console::new(fb);
    console.put_string("Hello from console1\n");
    console.put_string("Goodbye from console2\n");

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
