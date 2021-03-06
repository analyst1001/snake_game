/* Module for reading/writing character to screen
//
// Significant portion of code has been used from Philipp Oppermann's blog
// @ https://os.phil-opp.com/
// Copyright (c) 2019 Philipp Oppermann
*/

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    /// Static Writer instance to ues for reading/writing from VGA buffer
    pub static ref VGA_WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// Representation for allowed colors to display on screen
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// Color byte for each character in VGA buffer
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode(((background as u8) << 4) | (foreground as u8))
    }
}

/// Character representation for VGA buffer
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

// Use only ascii character for comparison of ScreenChars
impl PartialEq for ScreenChar {
    fn eq(&self, other: &Self) -> bool {
        self.ascii_character == other.ascii_character
    }
}

// VGA Buffer size constants
pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

/// VGA buffer
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Reader/Writer for underlying VGA buffer
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Write a byte in the last line of buffer, changing the line if necessary
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1;
            }
        }
    }

    /// Write a new line character, effectively scrolling up if required
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Write a given string in the last line of VGA buffer
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Clear the complete VGA buffer
    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
    }

    /// Clear particular row of VGA buffer
    fn clear_row(&mut self, row: usize) {
        let blank_char = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank_char);
        }
    }

    /// Write a character at given row and column in VGA buffer
    pub fn write_character_at(&mut self, character: ScreenChar, row: usize, col: usize) {
        assert!(row < BUFFER_HEIGHT);
        assert!(col < BUFFER_WIDTH);
        self.buffer.chars[row][col].write(character);
    }

    /// Read a characte from given row and column in VGA buffer
    pub fn read_character_at(&self, row: usize, col: usize) -> ScreenChar {
        assert!(row < BUFFER_HEIGHT);
        assert!(col < BUFFER_WIDTH);
        self.buffer.chars[row][col].read()
    }

    /// Write a string starting at given row and column in VGA buffer
    pub fn write_string_at(&mut self, string: &str, row: usize, col: usize) {
        let mut current_row = row;
        let mut current_col = col;
        for c in string.chars() {
            match c {
                '\n' => {
                    current_row += 1;
                    current_col = 0;
                }
                c => {
                    if current_col >= BUFFER_WIDTH {
                        current_col = 0;
                        current_row += 1;
                    }
                    self.buffer.chars[current_row][current_col].write(ScreenChar {
                        ascii_character: c as u8,
                        color_code: self.color_code,
                    });
                    current_col += 1;
                }
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Print string at end of VGA buffer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Print string at end of VGA buffer, and change line after it
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        VGA_WRITER.lock().write_fmt(args).unwrap();
    });
}
