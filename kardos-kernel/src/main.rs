#![no_std]
#![no_main]

mod console;
pub mod graphics;

#[no_mangle]
pub extern "sysv64" fn kernel_main(fb: &graphics::FrameBuffer) {
    graphics::print_example(fb);

    graphics::print_string(fb, 0, 100, graphics::WHITE, "Hello, world from graphics");
    let mut console = console::Console::new(fb);

    let num_array = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    for s1 in num_array {
        for s2 in num_array {
            console.put_string(s1);
            console.put_string(s2);
            console.put_string("\n");
        }
    }
    console.put_string("finish!");

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
