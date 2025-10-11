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

use crate::allocator::HHDM;
use crate::allocator::frame_allocator::{get_frame_allocator, init_frame_allocator};
use crate::allocator::kernel_allocator::{Header, KernelAllocator};
use crate::fb::init_writer;
use crate::structures::linked_list::AlignedNode;

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    HHDM.call_once(|| requests::HHDM_REQUEST.get_response().unwrap().offset());

    init_writer();
    init_frame_allocator();

    let mut allocator = KernelAllocator::init();

    unsafe {
        let a = allocator.alloc(Layout::from_size_align(8, 8).unwrap());
        allocator.dealloc(a);
        let b = allocator.alloc(Layout::from_size_align(16, 8).unwrap());
        let c = allocator.alloc(Layout::from_size_align(8, 8).unwrap());
        let d = allocator.alloc(Layout::from_size_align(20, 8).unwrap());

        allocator.dealloc(d);

        allocator.dealloc(b);
        allocator.dealloc(c);

        let mut c = allocator.start;

        while c != core::ptr::null_mut() {
            println!("{:x} {:?}", c.addr() + size_of::<Header>(), (*c));

            c = (*c).next_block;
        }

        println!(
            "used memory {} {}",
            allocator.pointer, allocator.allocations
        );
    }
    loop {}
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    println!("Panic {}\n{:?}", info.message(), info.location());
    loop {}
}
