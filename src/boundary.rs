/* Boundary element for the game */

use spin::Mutex;
use lazy_static::lazy_static;

use crate::vga_buffer::{BUFFER_HEIGHT, BUFFER_WIDTH, ScreenChar, ColorCode, Color, Writer};

lazy_static! {
    static ref TL_CORNER_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 218,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref TR_CORNER_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 191,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref BL_CORNER_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 192,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref BR_CORNER_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 217,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref HORIZONTAL_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 196,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref VERTICAL_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 179,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
}

pub struct Boundary {}

const FIRST_ROW: usize = 1;
const LAST_ROW: usize = BUFFER_HEIGHT - 1;
const FIRST_COL: usize = 0;
const LAST_COL: usize = BUFFER_WIDTH - 1;

impl Boundary {
    /// Draw boundary on screen
    pub fn draw(&self, screen: &Mutex<Writer>) {
        let mut writer = screen.lock();
        // Draw top-left corner
        writer.write_character_at(&TL_CORNER_CHARACTER, FIRST_ROW, FIRST_COL);
        // Draw top-right corner
        writer.write_character_at(&TR_CORNER_CHARACTER, FIRST_ROW, LAST_COL);
        // Draw bottom-left corner
        writer.write_character_at(&BL_CORNER_CHARACTER, LAST_ROW, FIRST_COL);
        // Draw bottom-right corner
        writer.write_character_at(&BR_CORNER_CHARACTER, LAST_ROW, LAST_COL);
        // Draw first and last row
        for i in (FIRST_COL+1)..LAST_COL {
            writer.write_character_at(&HORIZONTAL_CHARACTER, FIRST_ROW, i);
            writer.write_character_at(&HORIZONTAL_CHARACTER, LAST_ROW, i);
        }
        // Draw first and last columns
        for i in (FIRST_ROW+1)..LAST_ROW {
            writer.write_character_at(&VERTICAL_CHARACTER, i, FIRST_COL);
            writer.write_character_at(&VERTICAL_CHARACTER, i, LAST_COL);
        }
    }
}
