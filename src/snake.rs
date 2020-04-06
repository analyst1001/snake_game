use crate::ring_buffer::RingBuffer;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;
use crate::vga_buffer::{BUFFER_HEIGHT, BUFFER_WIDTH, Writer, ScreenChar, Color, ColorCode};
use crate::{print, println};

const MAX_SNAKE_SIZE: usize = (BUFFER_HEIGHT - 3)*(BUFFER_WIDTH - 2);

/// Statically allocated array representing Pixels for snakes body
static mut ARRAY : [Pixel; MAX_SNAKE_SIZE] = [Pixel{row: 0,col: 0}; MAX_SNAKE_SIZE];

lazy_static! {
    pub static ref SNAKE: Mutex<Snake<'static>> = {
        let mut snake  = Snake {
            // Allow unsafe static mutable because we have single "thread" of execution currently
            body: RingBuffer::new(unsafe {&mut ARRAY}),
            direction: Direction::Left,
            turn_direction: None,
        };
        snake.body.append(Pixel{row: BUFFER_HEIGHT/2, col:BUFFER_WIDTH/2});
        snake.body.append(Pixel{row: BUFFER_HEIGHT/2, col:BUFFER_WIDTH/2 + 1});
        snake.body.append(Pixel{row: BUFFER_HEIGHT/2, col:BUFFER_WIDTH/2 + 2});
        Mutex::new(snake)
    };
    static ref HEAD_UP_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 30,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref HEAD_LEFT_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 17,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref HEAD_RIGHT_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 16,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref HEAD_DOWN_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 31,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref LU_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 217,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref LD_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 191,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref RU_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 192,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref RD_CHARACTER: ScreenChar =  ScreenChar {
        ascii_character: 218,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref UL_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 217,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref UR_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 192,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref DL_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 191,
        color_code: ColorCode::new(Color::White, Color::Black),
    };
    static ref DR_CHARACTER: ScreenChar = ScreenChar {
        ascii_character: 218,
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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pixel {
    row: usize,
    col: usize,
}

pub struct Snake<'s> {
    /// Current co-ordinates of body of the snake
    body: RingBuffer<'s, Pixel>,
    /// Current direction the snake is moving in
    direction: Direction,
    /// Direction to follow on next tick if user pressed any key after previous tick
    turn_direction: Option<Direction>,
}

impl<'s> Snake<'s> {
    /// Draw the complete snake on screen, assuming atleast length 3
    pub fn draw(&self, screen: &Mutex<Writer>) {
        screen.lock().clear_screen();
        // Draw head of the snake
        let head_pixel = self.body.peek_first();
        // Disable interrupts to avoid deadlock
        interrupts::without_interrupts(|| {
            match self.direction {
                Direction::Left => screen.lock().write_character_at(&HEAD_LEFT_CHARACTER, head_pixel.row, head_pixel.col),
                Direction::Right => screen.lock().write_character_at(&HEAD_RIGHT_CHARACTER, head_pixel.row, head_pixel.col),
                Direction::Up => screen.lock().write_character_at(&HEAD_UP_CHARACTER, head_pixel.row, head_pixel.col),
                Direction::Down => screen.lock().write_character_at(&HEAD_DOWN_CHARACTER, head_pixel.row, head_pixel.col),
            };

            // Draw body of the snake. Write two characters per iteration for current and next index.
            // The next iteration will replace the next index character, if the next index does not represent a tail.
            // Other approach is to read the last two elements at the end. and then draw the tail
            for (prev, current, next) in self.body.triple_iter() {
                match (next.row as i64 - current.row as i64, current.row as i64 - prev.row as i64, next.col as i64 - current.col as i64, current.col as i64 - prev.col as i64) {
                    (-1, 0, 0, 1) => {
                        // Left to Up
                        screen.lock().write_character_at(&LU_CHARACTER, current.row, current.col);
                        screen.lock().write_character_at(&VERTICAL_CHARACTER, next.row, next.col);
                    },
                    (1, 0, 0, 1) => {
                        // Left to Down
                        screen.lock().write_character_at(&LD_CHARACTER, current.row, current.col);
                        screen.lock().write_character_at(&VERTICAL_CHARACTER, next.row, next.col);
                    },
                    (-1, 0, 0, -1) => {
                        // Right to Up
                        screen.lock().write_character_at(&RU_CHARACTER, current.row, current.col);
                        screen.lock().write_character_at(&VERTICAL_CHARACTER, next.row, next.col);
                    },
                    (1, 0, 0, -1) => {
                        // Right to Down
                        screen.lock().write_character_at(&RD_CHARACTER, current.row, current.col);
                        screen.lock().write_character_at(&VERTICAL_CHARACTER, next.row, next.col);
                    },
                    (0, 1, -1, 0) => {
                        // Up to Left
                        screen.lock().write_character_at(&UL_CHARACTER, current.row, current.col);
                        screen.lock().write_character_at(&HORIZONTAL_CHARACTER, next.row, next.col);
                    },
                    (0, 1, 1, 0) => {
                        // Up to Right
                        screen.lock().write_character_at(&UR_CHARACTER, current.row, current.col);
                        screen.lock().write_character_at(&HORIZONTAL_CHARACTER, next.row, next.col);
                    },
                    (0, -1, -1, 0) => {
                        // Down to Left
                        screen.lock().write_character_at(&DL_CHARACTER, current.row, current.col);
                        screen.lock().write_character_at(&HORIZONTAL_CHARACTER, next.row, next.col);
                    },
                    (0, -1, 1, 0) => {
                        // Down to Right
                        screen.lock().write_character_at(&DR_CHARACTER, current.row, current.col);
                        screen.lock().write_character_at(&HORIZONTAL_CHARACTER, next.row, next.col);
                    }
                    (0, 0, 1, 1) | (0, 0, -1, -1) => {
                        // Left to Right or Right to Left
                        screen.lock().write_character_at(&HORIZONTAL_CHARACTER, current.row, current.col);
                        screen.lock().write_character_at(&HORIZONTAL_CHARACTER, next.row, next.col);
                    },
                    (1, 1, 0, 0) | (-1, -1, 0, 0) => {
                        // Up to Down or Down to Up
                        screen.lock().write_character_at(&VERTICAL_CHARACTER, current.row, current.col);
                        screen.lock().write_character_at(&VERTICAL_CHARACTER, next.row, next.col);
                    },
                    _ => panic!("Unexpected sequence of pixels: {:?} {:?} {:?}", prev, current, next),
                };
            }
        });
    }
    
    /// Process snake's movement per tick
    pub fn tick(&mut self) {
        if let Some(turn_direction) = self.turn_direction {
            if turn_direction == self.direction {
                // No change in direction
                self.move_ahead();
            }
        }
        
        match self.turn_direction {
            None => {
                // No change in direction
                self.move_ahead();
            },
            Some(Direction::Left) => {
                self.turn_left();
            },
            Some(Direction::Right) => {
                self.turn_right();
            },
            Some(Direction::Up) => {
                self.turn_up();
            },
            Some(Direction::Down) => {
                self.turn_down();
            },
        }
        // Reset for next tick
        self.turn_direction = None;
    }

    /// Make the snake take one step forward in current direction
    fn move_ahead(&mut self) {
        let head_pixel = self.body.peek_first();
        let new_head_pixel = match self.direction {
            Direction::Left => {
                Pixel {
                    row: head_pixel.row,
                    col: head_pixel.col - 1,
                }
            },
            Direction::Right => {
                Pixel {
                    row: head_pixel.row,
                    col: head_pixel.col + 1,
                }
            },
            Direction::Up => {
                Pixel {
                    row: head_pixel.row - 1,
                    col: head_pixel.col,
                }
            },
            Direction::Down => {
                Pixel {
                    row: head_pixel.row + 1,
                    col: head_pixel.col,
                }
            },
        };
        self.body.prepend(new_head_pixel);
        self.body.pop_last();
    }
    
    /// Make the snake turn to the upward direction on screen
    fn turn_up(&mut self) {
        let head_pixel = self.body.peek_first();
        let new_head_pixel = match self.direction {
            Direction::Left | Direction::Right | Direction::Up => {
                // Continue moving up, or turn the head to up
                self.direction = Direction::Up;
                Pixel {
                    row: head_pixel.row - 1,
                    col: head_pixel.col,
                }
            },
            Direction::Down => {
                // Continue moving down
                Pixel {
                    row: head_pixel.row + 1,
                    col: head_pixel.col,
                }
            },
        };
        self.body.prepend(new_head_pixel);
        self.body.pop_last();
    }

    /// Make the snake turn to the downward direction on screen
    fn turn_down(&mut self) {
        let head_pixel = self.body.peek_first();
        let new_head_pixel = match self.direction {
            Direction::Left | Direction::Right | Direction::Down => {
                // Continue moving down, or turn the head to down
                self.direction = Direction::Down;
                Pixel {
                    row: head_pixel.row + 1,
                    col: head_pixel.col,
                }
            },
            Direction::Up => {
                // Continue moving up
                Pixel {
                    row: head_pixel.row - 1,
                    col: head_pixel.col,
                }
            },
        };
        self.body.prepend(new_head_pixel);
        self.body.pop_last();
    }

    /// Make the snake turn to the left direction on screen
    fn turn_left(&mut self) {
        let head_pixel = self.body.peek_first();
        let new_head_pixel = match self.direction {
            Direction::Left | Direction::Up | Direction::Down => {
                // Continue moving left, or turn the head to left
                self.direction = Direction::Left;
                Pixel {
                    row: head_pixel.row,
                    col: head_pixel.col - 1,
                }
            },
            Direction::Right => {
                // Continue moving right
                Pixel {
                    row: head_pixel.row,
                    col: head_pixel.col + 1,
                }
            },
        };
        self.body.prepend(new_head_pixel);
        self.body.pop_last();
    }

    /// Make the snake turn to the right direction on screen
    fn turn_right(&mut self) {
        let head_pixel = self.body.peek_first();
        let new_head_pixel = match self.direction {
            Direction::Right | Direction::Up | Direction::Down => {
                // Continue moving right, or turn the head to right
                self.direction = Direction::Right;
                Pixel {
                    row: head_pixel.row,
                    col: head_pixel.col + 1,
                }
            },
            Direction::Left => {
                // Continue moving left
                Pixel {
                    row: head_pixel.row,
                    col: head_pixel.col - 1,
                }
            },
        };
        self.body.prepend(new_head_pixel);
        self.body.pop_last();
    }

    /// Set direction to turn upon next tick
    pub fn set_turn_direction(&mut self, turn_direction: Direction) {
        match self.turn_direction {
            None => {
                // First direction key press after previous tick
                self.turn_direction = Some(turn_direction)
            },
            Some(_) => {
                // Do nothing after first direction  change
            },
        }
    }
}
