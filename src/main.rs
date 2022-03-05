#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;
use vga::writers::{Graphics640x480x16, GraphicsWriter};
use vga::colors::{Color16, TextModeColor};
use vga::writers::{ScreenCharacter, TextWriter, Text80x25};

mod vga_buffer;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // let mode = Graphics640x480x16::new();
    // mode.set_mode();
    // mode.clear_screen(Color16::Black);
    // mode.draw_line((80, 60), (80, 420), Color16::White);
    // mode.draw_line((80, 60), (540, 60), Color16::White);
    // mode.draw_line((80, 420), (540, 420), Color16::White);
    // mode.draw_line((540, 420), (540, 60), Color16::White);
    // mode.draw_line((80, 90), (540, 90), Color16::White);
    // for (offset, character) in "Hello World!".chars().enumerate() {
    //     mode.draw_character(270 + offset * 8, 72, character, Color16::White)
    // }

    println!("Hello World{}", "!");

    // let text_mode = Text80x25::new();
    // let color = TextModeColor::new(Color16::Yellow, Color16::Black);
    // let screen_character = ScreenCharacter::new(b'T', color);
    // text_mode.set_mode();
    // text_mode.clear_screen();
    // text_mode.write_character(0, 0, screen_character);

    loop {
      hlt()
    }
}

#[no_mangle]
fn hlt() {
    unsafe {
        // assembly で "HLT" したのと同じ効果がある。
        asm!("hlt");
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        hlt()
    }
}