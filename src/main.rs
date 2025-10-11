#![no_std]
#![no_main]
#![feature(sync_unsafe_cell, fn_traits)]

mod allocator;
mod fb;
mod requests;
mod structures;

use core::alloc::Layout;
use core::fmt::Write;

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

use alloc::vec::Vec;

use crate::allocator::HHDM;
use crate::allocator::frame_allocator::{get_frame_allocator, init_frame_allocator};
use crate::allocator::kernel_allocator::{get_kernel_allocator, init_kernel_allocator, Header, KernelAllocator, KernelAllocatorWrapper};
use crate::fb::init_writer;
extern crate alloc;

#[global_allocator]
pub static GLOBAL_KERNEL_ALLOCATOR: KernelAllocatorWrapper = KernelAllocatorWrapper;

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    HHDM.call_once(|| requests::HHDM_REQUEST.get_response().unwrap().offset());

    init_writer();

    init_frame_allocator();
    init_kernel_allocator();

    let mut vector = Vec::new();

    vector.push(2);
    vector.push(90);
    vector.push(20);

    let mut vector2 = Vec::new();

    vector2.push(20);
    vector2.push(900);
    vector2.push(200);
    vector2.push(900);

    println!("{:?} {:?}", vector, vector2);

    unsafe {
        let mut c = get_kernel_allocator().start.load(Ordering::SeqCst);

        while c != core::ptr::null_mut() {
            println!("{:x} {:?}", c.addr() + size_of::<Header>(), (*c));

            c = (*c).next_block;
        }

        println!(
            "used memory {}",
            get_kernel_allocator().pointer
        );
    }
    loop {}
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("Panic {}\n{:?}", info.message(), info.location());
    loop {}
}
