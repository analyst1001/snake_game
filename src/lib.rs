#![no_std]
#![no_main]
#![feature(const_fn)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]

extern crate volatile;
extern crate spin;
#[macro_use]
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


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

#[no_mangle]
pub extern fn rust_main() {
    use vga_buffer::{VGA_WRITER, ScreenChar, ColorCode, Color};
    VGA_WRITER.lock().write_character_at(&ScreenChar{ascii_character: 30, color_code: ColorCode::new(Color::White, Color::Black)}, 0, 0);
    let hello = b"Hello World!";
    let color_byte = 0x1f;

    let mut hello_colored = [color_byte; 24];
    for (i, char_byte) in hello.into_iter().enumerate() {
        hello_colored[i*2] = *char_byte;
    }

    let buffer_ptr = (0xb8000 + 1988) as *mut _;
    unsafe { *buffer_ptr = hello_colored };

    interrupts::init();

    fn stack_overflow() {
        print!("F");
        stack_overflow();
    }
//    stack_overflow();

//    unsafe { software_interrupt!(3) };
//     unsafe {*(0xdeadbeef as *mut u64) = 42};
//    unsafe { asm!("ud2") };
//    divide_by_zero();
    println!("Did not crash!");
    hlt_loop();

//    use snake::{SNAKE};

//    let boundary = boundary::Boundary{};
//    let score = score::Score::new(65535);

//    SNAKE.lock().draw(&VGA_WRITER);
//    boundary.draw(&VGA_WRITER);
//    score.draw(&VGA_WRITER);
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx":::"ax", "dx": "volatile", "intel")
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
