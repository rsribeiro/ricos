use x86_64::instructions::port::Port;
use crate::task::{
    spawner,
    Task,
    sleep::Sleep
};
use core::time::Duration;
use spin::Mutex;

const PIT_COMMAND_ADDRESS: u16 = 0x43;
const PIT_CHANNEL2_PORT_ADDRESS: u16 = 0x42;
const SPEAKER_PORT_ADDRESS: u16 = 0x61;
const PIT_FREQUENCY: u32 = 1_193_180;

pub static PC_SPEAKER: Mutex<PCSpeaker> = Mutex::new(PCSpeaker::new());

//https://wiki.osdev.org/PC_Speaker
pub struct PCSpeaker {
    pit_command_port: Port<u8>,
    pit_channel2_port: Port<u8>,
    speaker_port: Port<u8>
}

impl PCSpeaker {
    const fn new() -> PCSpeaker {
        PCSpeaker {
            pit_command_port: Port::new(PIT_COMMAND_ADDRESS),
            pit_channel2_port: Port::new(PIT_CHANNEL2_PORT_ADDRESS),
            speaker_port: Port::new(SPEAKER_PORT_ADDRESS)
        }
    }

    pub fn set_frequency(&mut self, frequency: u32) {
        let div = PIT_FREQUENCY / frequency;

        unsafe {
            self.pit_command_port.write(0xB6);
            self.pit_channel2_port.write(div as u8);
            self.pit_channel2_port.write((div >> 8) as u8);
        }
    }

    pub fn play_frequency(&mut self, frequency: u32) {
        self.set_frequency(frequency);

        //set pit to square wave
        unsafe {
            let tmp = self.speaker_port.read();
            if tmp != (tmp | 3) {
                self.speaker_port.write(tmp | 3);
            }
        }
    }

    pub fn stop(&mut self) {
        unsafe {
            let tmp = self.speaker_port.read() & 0xFC;
            self.speaker_port.write(tmp);
        }
    }

    pub async fn beep(&mut self) {
        self.play_frequency(750);
        Sleep::new(Duration::from_millis(55)).await;
        self.stop();
        //self.set_frequency(old_frequency);
    }
}

pub fn beep() {
    //TODO can cause deadlock
    spawner::spawn(Task::new(async {
        crate::pc_speaker::PC_SPEAKER.lock().beep().await;
    }))
}
