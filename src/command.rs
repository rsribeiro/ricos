use spin::Mutex;
use pc_keyboard::{DecodedKey, KeyCode};
use crate::{
    vga_buffer,
    print,
    println,
    eprintln,
    error::Error
};
use alloc::{
    str::FromStr,
    string::String,
    vec::Vec
};
#[cfg(feature="acpi-feat")]
use crate::acpi;

pub static COMMAND_PROCESSOR: Mutex<CommandProcessor> = Mutex::new(CommandProcessor::new());

pub struct CommandProcessor {
    command_buffer: Vec<char>
}

enum Command {
    Help,
    Uptime,
    Color,
    Clear,
    #[cfg(feature="pc-speaker")]
    Beep,
    Chars,
    Chars2,
    Window,
    Echo,
    Panic,
    Exit,
    #[cfg(feature="acpi-feat")]
    AcpiShutdownInfo
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        match command {
            "help" => Ok(Self::Help),
            "uptime" => Ok(Self::Uptime),
            "color" => Ok(Self::Color),
            "clear" => Ok(Self::Clear),
            #[cfg(feature="pc-speaker")]
            "beep" => Ok(Self::Beep),
            "chars" => Ok(Self::Chars),
            "chars2" => Ok(Self::Chars2),
            "window" => Ok(Self::Window),
            "echo" => Ok(Self::Echo),
            "panic" => Ok(Self::Panic),
            "exit"|"quit"|"shutdown" => Ok(Self::Exit),
            #[cfg(feature="acpi-feat")]
            "acpi-shutdown-info" => Ok(Self::AcpiShutdownInfo),
            _ => Err(Error::InvalidCommand)
        }
    }
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

    fn process_command(&self, original_command: &str) -> Result<(),Error> {
        let mut args = original_command.split_ascii_whitespace();
        if let Some(command)  = args.next() {
            if original_command.find(command).unwrap() > 0 {
                return Err(Error::InvalidCommand);
            }
            let command = command.parse::<Command>()?;
            let args: Vec<&str> = args.collect();

            match command {
                Command::Help => self.help(),
                Command::Uptime => Ok(println!("System uptime is {:#?}", crate::time::get_system_uptime())),
                Command::Color => self.set_colors(args),
                Command::Clear => Ok(crate::vga_buffer::clear()),
                #[cfg(feature="pc-speaker")]
                Command::Beep => Ok(crate::pc_speaker::beep()),
                Command::Chars => {
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
                Command::Chars2 => Ok(crate::vga_buffer::chars()),
                Command::Window => self.draw_window_frame(args),
                Command::Echo => {
                    let idx = original_command.find("echo").unwrap();
                    let arg = original_command.split_at(idx+4).1;
                    Ok(println!("{}", arg))
                },
                Command::Panic => {
                    if args.is_empty() {
                        panic!();
                    } else {
                        panic!("{}", args.join(" "));
                    }
                },
                Command::Exit => crate::exit(),
                #[cfg(feature="acpi-feat")]
                Command::AcpiShutdownInfo => {
                    if let Some((port,value)) = acpi::get_shutdown_info() {
                        println!("acpi_shutdown_info: port=0x{:04x}, value=0x{:04x}", port, value);
                    } else {
                        eprintln!("Acpi shutdown info not found.");
                    }
                    Ok(())
                }
            }?;
        }
        Ok(())
    }

    fn set_colors(&self, args: Vec<&str>) -> Result<(),Error> {
        let colors= args.iter().map(|arg| arg.parse()).collect::<Result<Vec<_>,_>>()?;

        if colors.len() == 2 {
            vga_buffer::set_color(colors[0], colors[1]);
            Ok(())
        } else if colors.is_empty() {
            println!("Available colors: Black, Blue, Green, Cyan, Red, Magenta, Brown, LightGray, DarkGray, LightBlue, LightGreen, LightCyan, LightRed, Pink, Yellow, White");
            Ok(())
        } else {
            Err(Error::WrongNumberOfArguments(2))
        }
    }

    fn draw_window_frame(&self, args: Vec<&str>) -> Result<(),Error> {
        let args = args.iter().map(|arg| arg.parse()).collect::<Result<Vec<_>,_>>()?;

        if args.len() == 4 {
            vga_buffer::draw_window_frame(args[0], args[1], args[2], args[3]);
            Ok(())
        } else {
            Err(Error::WrongNumberOfArguments(4))
        }
    }

    fn help(&self) -> Result<(),Error> {
        println!("╓──────────────────────────────────────────────────────────────────────────────┐");
        println!("║                               List of Commands                               │");
        println!("╟──────────────────────────────────────────────────────────────────────────────┤");
        println!("║* help: prints this help                                                      │");
        println!("║* uptime: prints system uptime                                                │");
        println!("║* color foreground background: changes screen colors                          │");
        #[cfg(feature="pc-speaker")]
        println!("║* beep: beeps pc speaker                                                      │");
        println!("║* window origin_x origin_y width height: draws window frame                   │");
        println!("║* chars: prints all characters                                                │");
        println!("║* chars2: prints all characters                                               │");
        println!("║* clear: clears the screen buffer                                             │");
        println!("║* panic [reason]: panics with optional reason                                 │");
        println!("║* exit/quit/shutdown: shuts down the computer                                 │");
        #[cfg(feature="acpi-feat")]
        println!("║* acpi-shutdown-info: prints shutdown info from acpi                          │");
        println!("╚══════════════════════════════════════════════════════════════════════════════╛");
        Ok(())
    }
}
