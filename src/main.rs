#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use blog_os::{
    println,
    vga_buffer::{self, Color},
    memory::{self, BootInfoFrameAllocator},
    allocator,
    logging,
    task::{Task, executor::Executor, keyboard, spawner}
};
use x86_64::VirtAddr;
use log::LevelFilter;

#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::Trace;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::Info;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    logging::init(LOG_LEVEL).unwrap();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
    .expect("heap initialization failed");

    blog_os::init();

    #[cfg(test)]
    test_main();

    log::trace!("initializing task execution");
    let mut executor = Executor::new();

    spawner::spawn(Task::new(async {
        #[cfg(feature="random")]
        vga_buffer::randomize_vga_buffer().await;

        spawner::spawn(Task::new(keyboard::print_keypresses()));

        #[cfg(feature="mouse")]
        //TODO mouse panics if keyboard key is pressed before "beep"
        spawner::spawn(Task::new(blog_os::task::mouse::process_packets()));

        #[cfg(feature="pc-speaker")]
        blog_os::pc_speaker::beep();

        vga_buffer::clear();
        vga_buffer::set_color(Color::Black, Color::LightGray);
        println!("╓──────────────────────────────────────────────────────────────────────────────┐");
        println!("║                                     RicOS                                    │");
        println!("╚══════════════════════════════════════════════════════════════════════════════╛");
        vga_buffer::set_color(Color::LightGray, Color::Black);
        println!("Type 'help' to see list of commands.");
    }));
    executor.run();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    blog_os::eprintln!("{}", info);
    blog_os::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
