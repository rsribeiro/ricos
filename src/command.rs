use spin::Mutex;
use pc_keyboard::{DecodedKey, KeyCode};
use crate::{
    vga_buffer::{self, Color},
    print,
    println,
    error_println
};
use alloc::{
    string::String,
    vec::Vec
};

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
                _ => print!("keycode {:?}", key)
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
            self.process_command(&command);
            self.command_buffer.clear();
        }
    }

    fn process_command(&self, command: &str) {
        let mut args = command.split_ascii_whitespace();
        if let Some(command)  = args.next() {
            let args: Vec<&str> = args.collect();

            log::trace!("command={:?}", command);
            log::trace!("args={:?}", args);

            match command {
                "help" => self.help(),
                "uptime" => println!("System uptime is {:?}", crate::time::get_system_uptime()),
                "color" => {
                    if args.len() == 2 {
                        let foreground = Color::from_string(args[0]);
                        let background = Color::from_string(args[1]);

                        if let (Some(foreground), Some(background)) = (foreground, background) {
                            vga_buffer::set_color(foreground, background);
                        } else {
                            error_println!("Error parsing colors {:?} or {:?}.", args[0], args[1]);
                        }
                    } else {
                        error_println!("Command 'color' expects two arguments.");
                    }
                },
                "clear"=> crate::vga_buffer::clear(),
                #[cfg(feature="pc-speaker")]
                "beep" => crate::pc_speaker::beep(),
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
                },
                "chars2" => crate::vga_buffer::chars(),
                // "test"=> { },
                "window" => vga_buffer::draw_window_frame((1,1),(10,5)),
                _ => error_println!("Command '{}' not recognized.", command)
            }
        }
    }

    fn help(&self) {
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
    }
}