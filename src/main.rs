#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

mod requests;
use core::fmt::Write;
mod fb;

use core::panic::PanicInfo;

use crate::fb::init_writer;

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    init_writer();

    println!("Test");
    println!("Test2");

    loop {}
}


#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
