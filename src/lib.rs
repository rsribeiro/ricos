#![no_std]
#![cfg_attr(test, no_main)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(const_in_array_repeat_expressions)]
#![feature(custom_test_frameworks)]
#![feature(wake_trait)]
#![feature(clamp)]

extern crate alloc;
extern crate rlibc;

pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod task;
pub mod time;
pub mod command;
pub mod logging;
pub mod encoding;
pub mod error;

#[cfg(feature="pc-speaker")]
pub mod pc_speaker;

use core::panic::PanicInfo;
use x86_64::instructions::port::Port;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    logging::init(log::LevelFilter::Trace).unwrap();
    init();
    test_main();
    hlt_loop()
}

pub fn init() {
    #[cfg(feature="mouse")]
    task::mouse::init();
    
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    let mut port = Port::new(0xf4);
    unsafe { port.write(exit_code as u32); }
}

pub fn exit() -> ! {
    // https://wiki.osdev.org/Shutdown
    log::trace!("trying to shutdown assuming QEMU");
    unsafe { Port::<u16>::new(0x604).write(0x2000) };

    log::trace!("trying to shutdown assuming BOCHS");
    unsafe { Port::<u16>::new(0xB004).write(0x2000) };

    log::trace!("trying to shutdown assuming VirtualBox");
    unsafe { Port::<u16>::new(0x4004).write(0x3400) };

    panic!("Shutdown failed")
}
