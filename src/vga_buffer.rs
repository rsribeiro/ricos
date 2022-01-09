use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::{
    interrupts,
    port::Port
};
use alloc::str::FromStr;
use crate::error::Error;

#[cfg(feature="random")]
use rand::{
    SeedableRng,
    rngs::SmallRng, 
    distributions::{Distribution, Standard},
    Rng,
};
#[cfg(feature="random")]
use core::time::Duration;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::vga_buffer::_print_error(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::eprint!("{}\n", format_args!($($arg)*)));
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl FromStr for Color {
    type Err = Error;

    fn from_str(color: &str) -> Result<Self, Self::Err> {
        match color.to_lowercase().as_str() {
            "black" => Ok(Color::Black),
            "blue" => Ok(Color::Blue),
            "green" => Ok(Color::Green),
            "cyan" => Ok(Color::Cyan),
            "red" => Ok(Color::Red),
            "magenta" => Ok(Color::Magenta),
            "brown" => Ok(Color::Brown),
            "lightgray" => Ok(Color::LightGray),
            "darkgray" => Ok(Color::DarkGray),
            "lightblue" => Ok(Color::LightBlue),
            "lightgreen" => Ok(Color::LightGreen),
            "lightcyan" => Ok(Color::LightCyan),
            "lightred" => Ok(Color::LightRed),
            "pink" => Ok(Color::Pink),
            "yellow" => Ok(Color::Yellow),
            "white" => Ok(Color::White),
            _ => Err(Error::ColorParseError)
        }
    }
}

#[cfg(feature="random")]
impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        use Color::*;
        match rng.gen_range(0..=15) {
            0 => Blue,
            1 => Green,
            2 => Cyan,
            3 => Red,
            4 => Magenta,
            5 => Brown,
            6 => LightGray,
            7 => DarkGray,
            8 => LightBlue,
            9 => LightGreen,
            10 => LightCyan,
            11 => LightRed,
            12 => Pink,
            13 => Yellow,
            14 => White,
            _ => Black
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    cp437_character: u8,
    color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

struct Cursor {
    position: (usize,usize),//x,y
    cursor_high: Port<u8>,
    cursor_low: Port<u8>
}

impl Cursor {
    fn enable(&mut self, cursor_start: u8, cursor_end: u8) {
        unsafe {
            let start = (self.cursor_low.read() & 0xC0) | cursor_start;
            let end = (self.cursor_low.read() & 0xE0) | cursor_end;

            self.cursor_high.write(0x0A);
            self.cursor_low.write(start);
            self.cursor_high.write(0x0B);
            self.cursor_low.write(end);
        }
    }

    fn set_position_raw(&mut self) {
        let (x, y) = self.position;
        let pos = y * BUFFER_WIDTH + x;
        unsafe {
            self.cursor_high.write(0x0F);
            self.cursor_low.write((pos & 0xFF) as u8);
            self.cursor_high.write(0x0E);
            self.cursor_low.write(((pos >> 8) & 0xFF) as u8);
        }
    } 
}

pub struct Writer {
    position: (usize,usize),//x,y
    foreground: Color,
    background: Color,
    buffer: &'static mut Buffer,
    cursor: Cursor
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = {
        let mut writer = Writer {
            position: (0,0),
            foreground: Color::LightGray,
            background: Color::Black,
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
            cursor: Cursor {
                position: (0,0),
                cursor_high: Port::new(0x3D4),
                cursor_low: Port::new(0x3D5)
            }
        };
        writer.cursor.enable(0, 15);
        writer.cursor.set_position_raw();
        Mutex::new(writer)
    };
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.new_line()
            },
            byte => {
                if self.position.0 >= BUFFER_WIDTH {
                    self.new_line();
                }

                let color_code = ColorCode::new(self.foreground, self.background);
                self.buffer.chars[self.position.1][self.position.0].write(ScreenChar {
                    cp437_character: byte,
                    color_code: color_code,
                });
                self.position.0 += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.position.0 = 0;
        if self.position.1 == BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            
            self.clear_row(BUFFER_HEIGHT - 1);
        } else {
            self.position.1 += 1;
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            cp437_character: b'\0',
            color_code: ColorCode::new(Color::LightGray, Color::Black),
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        s.encode_utf16().map(crate::encoding::utf16_to_cp437).for_each(|b| self.write_byte(b));
        self.set_cursor_position(self.position.0, self.position.1);
    }

    pub fn set_cursor_position(&mut self, x: usize, y: usize) {
        self.cursor.position = (x,y);
        self.cursor.set_position_raw();
    }

    pub fn cursor_position_delta(&mut self, delta_x: i16, delta_y: i16) {
        self.cursor.position.0 = (self.cursor.position.0 as i16 + delta_x).clamp(0, BUFFER_WIDTH as i16 - 1) as usize;
        self.cursor.position.1 = (self.cursor.position.1 as i16 - delta_y).clamp(0, BUFFER_HEIGHT as i16 - 1) as usize;

        // log::trace!("delta_x={:?}, delta_y={:?}, pos={:?}", delta_x, delta_y, self.cursor.position);
        self.cursor.set_position_raw(); 
    }

    pub fn erase_character(&mut self) {
        if self.position.0 > 0 {
            self.position.0 -= 1;

            let color_code = ColorCode::new(Color::LightGray, Color::Black);
            self.buffer.chars[self.position.1][self.position.0].write(ScreenChar {
                cp437_character: b'\0',
                color_code: color_code,
            });
            
            self.set_cursor_position(self.position.0, self.position.1);
        }
    }

    pub fn set_foreground_color(&mut self, color: Color) {
        self.foreground = color;
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background = color;
    }

    pub fn get_background_color(&self) -> Color {
        self.background
    }

    pub fn get_foreground_color(&self) -> Color {
        self.foreground
    }

    #[cfg(feature="random")]
    pub async fn randomize_vga_buffer(&mut self) {
        let mut rng = SmallRng::seed_from_u64(0);

        for row in 0..BUFFER_HEIGHT {
            self.random_line(row, &mut rng);
            crate::task::sleep::Sleep::new(Duration::from_nanos(1)).await
        }
    }

    #[cfg(feature="random")]
    fn random_line<R: Rng>(&mut self, row: usize, rng: &mut R) {
        for col in 0..BUFFER_WIDTH {
            let cp437_character: u8 = rng.gen();
            let foreground: Color = rng.gen();
            let background: Color = rng.gen();

            let char = ScreenChar {
                cp437_character: cp437_character as u8,
                color_code: ColorCode::new(foreground, background),
            };
            self.buffer.chars[row][col].write(char);
        }
    }

    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.position = (0,0);
        self.set_cursor_position(0, 0);
    }

    fn write_screen_char_at_position(&mut self, screen_char: ScreenChar, x: usize, y: usize) {
        if x < BUFFER_WIDTH && y < BUFFER_HEIGHT {
            self.buffer.chars[y][x].write(screen_char);
        }
    }

    pub fn draw_window_frame(&mut self, origin_x: usize, origin_y: usize, width: usize, height: usize) {
        let color_code = ColorCode::new(self.foreground, self.background);

        //northwest
        self.write_screen_char_at_position(ScreenChar {
            cp437_character: 201,//╔
            color_code,
        }, origin_x, origin_y);

        if origin_x + width > 0 {
            //northeast
            self.write_screen_char_at_position(ScreenChar {
                cp437_character: 187,//╗
                color_code,
            }, origin_x + width - 1, origin_y);
        }

        //southwest
        if origin_y + height > 0 {
            self.write_screen_char_at_position(ScreenChar {
                cp437_character: 200,//╚
                color_code,
            }, origin_x, origin_y + height - 1);
        }

        //southeast
        if origin_x + width > 0 && origin_y + height > 0 {
            self.write_screen_char_at_position(ScreenChar {
                cp437_character: 188,//╝
                color_code,
            }, origin_x + width - 1, origin_y + height - 1);
        }
        
        if width > 0 {
            for col_offset in 1..width-1 {
                //north
                self.write_screen_char_at_position(ScreenChar {
                    cp437_character: 205,
                    color_code,
                }, origin_x + col_offset, origin_y);

                //south
                self.write_screen_char_at_position(ScreenChar {
                    cp437_character: 205,
                    color_code,
                }, origin_x + col_offset, origin_y + height - 1);
            }
        }

        if height > 0 {
            for row_offset in 1..height-1 {
                //west
                self.write_screen_char_at_position(ScreenChar {
                    cp437_character: 186,
                    color_code,
                }, origin_x, origin_y + row_offset);

                //east
                self.write_screen_char_at_position(ScreenChar {
                    cp437_character: 186,
                    color_code,
                }, origin_x + width - 1, origin_y + row_offset);
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

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _print_error(args: fmt::Arguments) {
    use core::fmt::Write;

    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        let curr_background = writer.get_background_color();
        let curr_foreground = writer.get_foreground_color();
        writer.set_background_color(Color::Black);
        writer.set_foreground_color(Color::Red);
        writer.write_fmt(args).unwrap();
        writer.set_background_color(curr_background);
        writer.set_foreground_color(curr_foreground);
    });
}

pub fn erase_character() {
    interrupts::without_interrupts(|| {
        WRITER.lock().erase_character();
    });
}

pub fn set_cursor_position(x: usize, y: usize) {
    interrupts::without_interrupts(|| {
        WRITER.lock().set_cursor_position(x,y);
    });
}

pub fn cursor_position_delta(delta_x: i16, delta_y: i16) {
    interrupts::without_interrupts(|| {
        WRITER.lock().cursor_position_delta(delta_x, delta_y);
    });
}

pub fn set_color(foreground: Color, background: Color) {
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writer.set_foreground_color(foreground);
        writer.set_background_color(background);
    });
}

pub fn clear() {
    WRITER.lock().clear();
}

#[cfg(feature="random")]
pub async fn randomize_vga_buffer() {
    WRITER.lock().randomize_vga_buffer().await;
}

pub fn chars() {
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        for c in 0x00..0xff_u8 {
            writer.write_byte(c);
        }
    });
}

pub fn draw_window_frame(origin_x: usize, origin_y: usize, width: usize, height: usize) {
    interrupts::without_interrupts(|| {
        WRITER.lock().draw_window_frame(origin_x, origin_y, width, height);
    });
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.cp437_character), c);
    }
}