use core::fmt::Write;
use core::{
    cell::SyncUnsafeCell,
    sync::atomic::{AtomicPtr, AtomicU64, Ordering},
};
use x86_64::structures::paging::PageSize;

use crate::{
    allocator::HHDM,
    println, requests,
    structures::{
        linked_list::{self, AlignedNode, LinkedList},
        once::Once,
    },
};

pub static FRAME_ALLOCATOR: SyncUnsafeCell<Option<FrameAllocator>> = SyncUnsafeCell::new(None);

// use only last
pub struct FrameAllocator {
    pub linked_list: LinkedList,
}

impl FrameAllocator {
    pub unsafe fn alloc_page(&mut self) -> u64 {
        unsafe {
            let end = self.linked_list.end;

            let value = (*end).value.load(Ordering::SeqCst);

            let prev = (*end).prev.load(Ordering::SeqCst);

            (*prev).next = AtomicPtr::new(core::ptr::null_mut());

            core::ptr::write_bytes(value as *mut u8, 0_u8, 4096);

            self.linked_list.end = prev;

            value
        }
    }
    pub unsafe fn dealloc_page(&mut self, page: *mut ()) {
        unsafe {
            let ptr = page as *mut AlignedNode;

            let node = AlignedNode {
                value: AtomicU64::new(ptr as u64),
                next: AtomicPtr::new(core::ptr::null_mut()),
                prev: AtomicPtr::new(self.linked_list.end),
            };

            (*ptr) = node;

            (*self.linked_list.end).next = AtomicPtr::new(ptr);
            self.linked_list.end = ptr;
        }
    }
}

unsafe impl Sync for FrameAllocator {}

unsafe impl Send for FrameAllocator {}

pub fn init_frame_allocator() {
    let mut node = AlignedNode::empty();

    let mut first = true;

    let mut current = (&mut node) as *mut AlignedNode;

    for mmap in requests::MEMORY_MAP.get_response().unwrap().entries() {
        if mmap.entry_type == limine::memory_map::EntryType::USABLE {
            for i in (mmap.base + *HHDM..=mmap.base + mmap.length + *HHDM).step_by(4096) {
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
                node.prev.store(ptr, Ordering::SeqCst);

                unsafe {
                    (*current).next.store(ptr, Ordering::SeqCst);
                }

                let tmp_node = AlignedNode {
                    value: AtomicU64::new(i),
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

    unsafe {
        let ptr = (node.value.load(Ordering::SeqCst)) as *mut AlignedNode;

        (*ptr) = AlignedNode {
            value: AtomicU64::new(node.value.load(Ordering::SeqCst)),
            next: AtomicPtr::new(node.next.load(Ordering::SeqCst)),
            prev: AtomicPtr::new(core::ptr::null_mut()),
        };

        *FRAME_ALLOCATOR.get() = Some(FrameAllocator {
            linked_list: LinkedList {
                start: ptr,
                end: current,
            },
        });
    }
}

pub fn get_frame_allocator() -> &'static mut FrameAllocator {
    unsafe { FRAME_ALLOCATOR.get().as_mut().unwrap().as_mut().unwrap() }
}
