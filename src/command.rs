use spin::Mutex;
use pc_keyboard::{DecodedKey, KeyCode};
use crate::{
    vga_buffer::{self, Color},
    print,
    println,
    eprintln
};
use alloc::{
    string::String,
    vec::Vec
};
use core::{fmt, num::ParseIntError};

#[derive(Debug)]
pub enum CommandError {
    WrongNumberOfArguments(u8),
    ColorArgumentExpected,
    NumericArgumentExpected,
    InvalidCommand
}

type CommandResult = Result<(),CommandError>;

impl From<ParseIntError> for CommandError {
    fn from(_: ParseIntError) -> Self {
        CommandError::NumericArgumentExpected
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WrongNumberOfArguments(n_args) => write!(f, "{} arguments expected.", n_args),
            Self::ColorArgumentExpected => write!(f, "Color name arguments expected."),
            Self::NumericArgumentExpected => write!(f, "Numeric arguments expected."),
            Self::InvalidCommand => write!(f, "Invalid command.")
        }
    }
}

pub static COMMAND_PROCESSOR: Mutex<CommandProcessor> = Mutex::new(CommandProcessor::new());

pub struct CommandProcessor {
    command_buffer: Vec<char>
}

impl CommandProcessor {
    const fn new() -> Self {
        CommandProcessor {
            command_buffer: Vec::new()
        }
    }

    pub fn process_key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::Unicode(character) => match character {
                '\u{0008}' => {//backspace
                    vga_buffer::erase_character();
                    self.remove_character();
                },
                '\u{0009}' => {},//tab
                '\u{000A}' => {//enter
                    println!();
                    self.finish_command();
                },
                '\u{001B}' => {},//esc
                _ => {
                    print!("{}", character);
                    self.add_character(character);
                }
            },
            DecodedKey::RawKey(key) => match key {
                KeyCode::ArrowUp => vga_buffer::cursor_position_delta(0, 1),
                KeyCode::ArrowDown => vga_buffer::cursor_position_delta(0, -1),
                KeyCode::ArrowLeft => vga_buffer::cursor_position_delta(-1, 0),
                KeyCode::ArrowRight => vga_buffer::cursor_position_delta(1, 0),
                KeyCode::AltLeft => {},
                KeyCode::F1 => {},
                KeyCode::F2 => {},
                KeyCode::F3 => {},
                KeyCode::F4 => {},
                KeyCode::F5 => {},
                KeyCode::F6 => {},
                KeyCode::F7 => {},
                KeyCode::F8 => {},
                KeyCode::F9 => {},
                KeyCode::F10 => {},
                KeyCode::F11 => {},
                KeyCode::F12 => {},
                _ => log::info!("keycode {:?}", key)
            }
        }
    }

    fn add_character(&mut self, character: char) {
        self.command_buffer.push(character);
    }

    fn remove_character(&mut self) {
        self.command_buffer.pop();
    }

    fn finish_command(&mut self) {
        if !self.command_buffer.is_empty() {
            let command: String = self.command_buffer.iter().collect();
            if let Err(err) = self.process_command(&command) {
                eprintln!("Error: {}", err);
            }
            self.command_buffer.clear();
        }
    }

    fn process_command(&self, command: &str) -> CommandResult {
        let mut args = command.split_ascii_whitespace();
        if let Some(command)  = args.next() {
            let args: Vec<&str> = args.collect();

            log::trace!("command={:?}", command);
            log::trace!("args={:?}", args);

            match command {
                "help" => self.help(),
                "uptime" => {
                    println!("System uptime is {:?}", crate::time::get_system_uptime());
                    Ok(())
                },
                "color" => self.set_colors(args),
                "clear"=> {
                    crate::vga_buffer::clear();
                    Ok(())
                },
                #[cfg(feature="pc-speaker")]
                "beep" => {
                    crate::pc_speaker::beep();
                    Ok(())
                },
                "panic" => {
                    if args.is_empty() {
                        panic!();
                    } else {
                        panic!("{}", args.join(" "));
                    }
                },
                "exit" | "quit" | "shutdown" => crate::exit(),
                "chars" => {
                    println!("\u{0000}☺☻♥♦♣♠•◘○\\n♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼");
                    // println!("\u{0000}☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼");
                    println!("\u{0020}!\"#$%&'()*+,-./0123456789:;<=>?");
                    println!("@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_");
                    println!("`abcdefghijklmnopqrstuvwxyz{{|}}~⌂");
                    println!("ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒ");
                    println!("áíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐");
                    println!("└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀");
                    println!("αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■\u{00a0}");
                    Ok(())
                },
                "chars2" => {
                    crate::vga_buffer::chars();
                    Ok(())
                },
                "window" => self.draw_window_frame(args),
                _ => Err(CommandError::InvalidCommand)
            }?;
        }
        Ok(())
    }

    fn set_colors(&self, args: Vec<&str>) -> CommandResult {
        let colors= args.iter().map(|arg| Color::from_string(arg)).collect::<Option<Vec<_>>>().ok_or(CommandError::ColorArgumentExpected)?;

        if colors.len() == 2 {
            vga_buffer::set_color(colors[0], colors[1]);
            Ok(())
        } else {
            Err(CommandError::WrongNumberOfArguments(2))
        }
    }

    fn draw_window_frame(&self, args: Vec<&str>) -> CommandResult {
        let args = args.iter().map(|a| a.parse::<usize>()).collect::<Result<Vec<_>,_>>()?;
        
        if args.len() == 4 {
            vga_buffer::draw_window_frame(args[0], args[1], args[2], args[3]);
            Ok(())
        } else {
            Err(CommandError::WrongNumberOfArguments(4))
        }
    }

    fn help(&self) -> CommandResult {
        println!("╓──────────────────────────────────────────────────────────────────────────────┐");
        println!("║                               List of Commands                               │");
        println!("╟──────────────────────────────────────────────────────────────────────────────┤");
        println!("║* help: prints this help                                                      │");
        println!("║* uptime: prints system uptime                                                │");
        println!("║* color foreground background: changes screen colors                          │");
        #[cfg(feature="pc-speaker")]
        println!("║* beep: beeps pc speaker                                                      │");
        println!("║* panic [reason]: panics with optional reason                                 │");
        println!("║* exit/quit/shutdown: shuts down the computer                                 │");
        println!("╚══════════════════════════════════════════════════════════════════════════════╛");
        Ok(())
    }
}