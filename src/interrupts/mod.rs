/* Root for interrupts module.
/
/  Sets handlers for interrupts, and load IDT to enable interrupt
/  handling
*/

mod gdt;
mod idt;

use crate::{print, println};
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use spin;
use vga_buffer::VGA_WRITER;

use crate::snake::{Direction, SNAKE};

// New offset for interrupts from PIC1 of chained PICs
pub const PIC_1_OFFSET: u8 = 32;
// New offset for interrupts from PIC2 of chained PICs
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// Chained PIC structure to handle Intel 8259 PIC
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Save caller saved registers
// Although "x86-interrupt" would be more efficient, going with naive approach for now
macro_rules! save_scratch_registers {
    () => {
        asm!("push rax
              push rcx
              push rdx
              push rsi
              push rdi
              push r8
              push r9
              push r10
              push r11"
              :::: "intel", "volatile");
    }
}

// Restore caller saved registers
macro_rules! restore_scratch_registers {
    () => {
        asm!("pop r11
              pop r10
              pop r9
              pop r8
              pop rdi
              pop rsi
              pop rdx
              pop rcx
              pop rax"
              :::: "intel", "volatile");
    }
}

/// Macro expansion for exception/interrupt handlers without error code parameter
macro_rules! handler {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                save_scratch_registers!();
                asm!("mov rdi, rsp  // first arg is stack pointer
                      add rdi, 9*8    // No alignment fix required: 9 * 8 byte registers + 5 * 8 byte registers => (14 * 8) % 16 == 0
                      call $0"
                      :: "i"($name as extern "C" fn(&ExceptionStackFrame))
                      : "rdi" : "intel");
                restore_scratch_registers!();
                asm!("iretq"
                      :::: "intel", "volatile");
                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }}
}

/// Macro expansion for exception/interrup handlers with error code parameter
macro_rules! handler_with_error_code {
    ($name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                save_scratch_registers!();
                asm!("mov rsi, [rsp + 9*8] // Load value for error code as second arg
                      mov rdi, rsp  // load stack pointer as first arg
                      add rdi, 10*8 // Fix first arg to point to exception's stack frame
                      sub rsp, 8    // fix 16 byte alignment
                      call $0
                      add rsp, 8"   // Restore rsp to allow restoring other registers
                      :: "i"($name as extern "C" fn(&ExceptionStackFrame, u64))
                      : "rdi", "rsi" : "intel");
                restore_scratch_registers!();
                asm!("add rsp, 8    // Skip error code
                      iretq"
                     :::: "intel", "volatile");
                ::core::intrinsics::unreachable();
            }
        }
        wrapper
    }}
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    /// Interrupt Descriptor Table with entries for exceptions/interrupts we handle
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(0, handler!(divide_by_zero_handler));
        idt.set_handler(3, handler!(breakpoint_handler));
        idt.set_handler(6, handler!(invalid_opcode_handler));
        idt.set_handler(8, handler_with_error_code!(double_fault_handler))
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX + 1);
        idt.set_handler(14, handler_with_error_code!(page_fault_handler));
        idt.set_handler(
            InterruptIndex::Timer.as_u8(),
            handler!(timer_interrupt_handler),
        );
        idt.set_handler(
            InterruptIndex::Keyboard.as_u8(),
            handler!(keyboard_interrupt_handler),
        );
        idt
    };
}

/// Exception stack frame provided as input to exception/interrupt handler
#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

bitflags! {
    struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1 << 0;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
    }
}

/// Handler for division by zero anytime during execution
extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) {
    panic!("EXCEPTION: DIVIDE_BY_ZERO {:#?}", stack_frame);
}

/// Handler when execution reaches an invalid opcode during execution
extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) {
    println!(
        "EXCEPTION: INVALID OPCODE at {:#x}\n{:#?}",
        stack_frame.instruction_pointer, stack_frame
    );
}

/// Handle page faults during execution
extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    use x86_64::registers::control;
    panic!(
        "\nEXCEPTION: PAGE FAULT while accessing {:#x}\
              \nerror code {:?}\n{:#?}",
        control::Cr2::read().as_u64(),
        PageFaultErrorCode::from_bits(error_code).unwrap(),
        stack_frame
    );
}

/// Handle breakpoints during execution
extern "C" fn breakpoint_handler(stack_frame: &ExceptionStackFrame) {
    println!(
        "EXCEPTION: BREAKPOINT at {:#x}\n{:#?}",
        stack_frame.instruction_pointer, stack_frame
    );
}

/// Handle all double fault exceptions during execution
extern "C" fn double_fault_handler(stack_frame: &ExceptionStackFrame, error_code: u64) {
    panic!(
        "EXCEPTION: DOUBLE FAULT\n{:#?}. Error code: {:#x}",
        stack_frame, error_code
    );
}

/// Handle Timer interrupts from Intel 8259 PIC
extern "C" fn timer_interrupt_handler(_stack_frame: &ExceptionStackFrame) {
    {
        SNAKE.lock().tick(&VGA_WRITER);
    }
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// Handle Keyboard interrupts from Intel 8259 PIC when user presses a key
extern "C" fn keyboard_interrupt_handler(_stack_frame: &ExceptionStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;
    // Okay to define here, since it will be initialized only once
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => (),
                DecodedKey::RawKey(key) => match key {
                    KeyCode::ArrowUp => SNAKE.lock().set_turn_direction(Direction::Up),
                    KeyCode::ArrowDown => SNAKE.lock().set_turn_direction(Direction::Down),
                    KeyCode::ArrowLeft => SNAKE.lock().set_turn_direction(Direction::Left),
                    KeyCode::ArrowRight => SNAKE.lock().set_turn_direction(Direction::Right),
                    _ => (),
                },
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

/// Initialize GDT, and load IDT. Also enable interrupt handling
pub fn init() {
    gdt::init();
    IDT.load();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}
