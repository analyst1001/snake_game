/* Entry module for Rust code */

#![no_std]
#![feature(const_fn)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]

extern crate bit_field;
extern crate lazy_static;
extern crate spin;
extern crate uart_16550;
extern crate volatile;
extern crate x86_64;
#[macro_use]
extern crate bitflags;
extern crate pc_keyboard;
extern crate pic8259_simple;

use core::panic::PanicInfo;

mod boundary;
mod interrupts;
mod prng;
mod ring_buffer;
mod score;
mod snake;
mod system_time;
mod vga_buffer;

use snake::SNAKE;
use vga_buffer::VGA_WRITER;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    hlt_loop();
}

#[no_mangle]
pub extern "C" fn rust_main() {
    VGA_WRITER.lock().clear_screen();

    let boundary = boundary::Boundary {};
    let score = score::Score::new(0);
    score.draw(&VGA_WRITER);
    boundary.draw(&VGA_WRITER);
    // Interrupts are not enabled until this point, therefore no need of disabling them while using `VGA_WRITER` to avoid deadlock
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
