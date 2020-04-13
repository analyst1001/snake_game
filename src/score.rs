/* Score element displayed on screen */

use crate::vga_buffer::{Color, ColorCode, ScreenChar, Writer, BUFFER_WIDTH};
use spin::Mutex;
use x86_64::instructions::interrupts;

const SCORE_ROW: usize = 0;
const INCREMENT: u16 = 1;
const SCORE_LABEL: &str = "SCORE: ";
// Subtract length of SCORE_LABEL (7) + number of digits in max score (5)
const SCORE_COL: usize = BUFFER_WIDTH - 12;

pub struct Score {
    // Under current VGA text buffer implementation, score will be < 2000
    value: u16,
}

impl Score {
    pub fn new(initial_score: u16) -> Self {
        Score {
            value: initial_score,
        }
    }

    /// Increment the score value
    pub fn increment(&mut self) {
        self.value += INCREMENT;
    }

    /// Get the current score
    pub fn get_score(&self) -> u16 {
        self.value
    }

    /// Draw the score at top-right corner on the screen
    pub fn draw(&self, screen: &Mutex<Writer>) {
        // Disable interrupts to avoid deadlock
        interrupts::without_interrupts(|| {
            // TODO: Use format strings here
            screen
                .lock()
                .write_string_at(SCORE_LABEL, SCORE_ROW, SCORE_COL);
            let mut writer = screen.lock();
            let mut value = self.value;
            let mut i = 0;
            while value > 0 {
                let digit: u8 = (value % 10) as u8 + ('0' as u8);
                let character = ScreenChar {
                    ascii_character: digit,
                    color_code: ColorCode::new(Color::White, Color::Black),
                };
                writer.write_character_at(&character, SCORE_ROW, BUFFER_WIDTH - i - 1);
                i += 1;
                value = value / 10;
            }
        });
    }
}
