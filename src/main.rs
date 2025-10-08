#![no_std]
#![no_main]
#![feature(sync_unsafe_cell, fn_traits)]

mod allocator;
mod fb;
mod requests;
mod structures;

use core::fmt::Write;

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

use crate::allocator::frame_allocator::init_frame_allocator;
use crate::allocator::HHDM;
use crate::fb::init_writer;
use crate::structures::linked_list::AlignedNode;

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    HHDM.call_once(|| requests::HHDM_REQUEST.get_response().unwrap().offset());

    init_writer();
    init_frame_allocator();
    
    let mut node = structures::linked_list::AlignedNode {
        value: AtomicU64::new(0),
        next: AtomicPtr::new(core::ptr::null_mut()),
        prev: AtomicPtr::new(core::ptr::null_mut()),
    };



    let mut current = (&mut node) as *const AlignedNode; 
    
    while current != core::ptr::null() {
       unsafe {
            println!("{:?}", (*current));



           current = (*current).next.load(Ordering::Relaxed);
       }
    }


    loop {}
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
