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

use crate::fb::init_writer;
use crate::structures::linked_list::AlignedNode;

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    init_writer();

    let mut node = structures::linked_list::AlignedNode {
        value: AtomicU64::new(0),
        next: AtomicPtr::new(core::ptr::null_mut()),
        prev: AtomicPtr::new(core::ptr::null_mut()),
    };
    let hhdm = requests::HHDM_REQUEST.get_response().unwrap().offset();


    let mut first = true;
    let mut current = (&mut node) as *mut AlignedNode;

    for mmap in requests::MEMORY_MAP.get_response().unwrap().entries() {
        if mmap.entry_type == limine::memory_map::EntryType::USABLE {
            for i in (mmap.base+hhdm..=mmap.base+mmap.length+hhdm).step_by(4096) {
                if first {
                    unsafe {
                        (*current).value.store(i, Ordering::SeqCst);
                    }
                    first = false;
                    continue;
                }


                if i == 0 {
                    continue;
                }

                let ptr = i as *mut AlignedNode;
                unsafe {
                    (*current).next.store(ptr, Ordering::SeqCst);
                }

                let tmp_node = AlignedNode {
                    value: AtomicU64::new(i - hhdm),
                    next: AtomicPtr::new(core::ptr::null_mut()),
                    prev: AtomicPtr::new(current),
                };

                unsafe {
                    (*ptr) = tmp_node;
                }

                current = ptr;
            }
       
        }
    }

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
