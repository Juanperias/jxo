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

use crate::allocator::frame_allocator::{get_frame_allocator, init_frame_allocator};
use crate::allocator::HHDM;
use crate::fb::init_writer;
use crate::structures::linked_list::AlignedNode;

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    HHDM.call_once(|| requests::HHDM_REQUEST.get_response().unwrap().offset());

    init_writer();
    init_frame_allocator();
    


    let frame = get_frame_allocator();
    
    unsafe {
        let page1 = frame.alloc_page();
        let page2 = frame.alloc_page();

        println!("Page 1 at {}", page1);
        println!("Page 2 at {}", page2);

        frame.dealloc_page(page2 as *mut ());
        println!("page 2 deallocated");

        let page3 = frame.alloc_page();
        
        println!("Page 3 at {}", page3);
    }


    loop {}
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
