#![no_std]
#![feature(const_fn)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]

extern crate volatile;
extern crate spin;
extern crate lazy_static;
extern crate uart_16550;
extern crate bit_field;
#[macro_use]
extern crate x86_64;
#[macro_use]
extern crate bitflags;
extern crate pic8259_simple;
extern crate pc_keyboard;

use core::panic::PanicInfo;


mod vga_buffer;
mod snake;
mod ring_buffer;
mod boundary;
mod score;
mod interrupts;
mod prng;
mod system_time;

use snake::{SNAKE};
use vga_buffer::{VGA_WRITER};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

#[no_mangle]
pub extern fn rust_main() { 
    VGA_WRITER.lock().clear_screen();

    let boundary = boundary::Boundary{};
    let score = score::Score::new(0);
    score.draw(&VGA_WRITER);
    boundary.draw(&VGA_WRITER);
    // Interrupts are not enabled until this point, therefore not need of disabling them to avoid deadlock
    SNAKE.lock().draw(&VGA_WRITER);
    SNAKE.lock().set_score_handler(score);

    interrupts::init();

    hlt_loop();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
