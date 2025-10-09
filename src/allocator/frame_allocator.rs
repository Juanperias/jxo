use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

use crate::{allocator::HHDM, println, requests, structures::{linked_list::{AlignedNode, LinkedList}, once::Once}};

pub static FRAME_ALLOCATOR: Once<FrameAllocator> = Once::new();

// use only last
pub struct FrameAllocator {
    pub linked_list: Option<LinkedList>, 
}

impl FrameAllocator {
}


pub fn init_frame_allocator() -> FrameAllocator {
    let mut node = AlignedNode::empty();

    let mut first = true;

    let mut current = (&mut node) as *mut AlignedNode;

    for mmap in requests::MEMORY_MAP.get_response().unwrap().entries() {
        if mmap.entry_type == limine::memory_map::EntryType::USABLE {
            for i in (mmap.base+*HHDM..=mmap.base+mmap.length+*HHDM).step_by(4096) {
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
                    value: AtomicU64::new(i - *HHDM),
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
        let last = &(*(node).prev.load(Ordering::SeqCst));
   
        let ptr = node.value.load(Ordering::SeqCst) as *mut AlignedNode;

        (*ptr) = AlignedNode {
                    value: AtomicU64::new(node.value.load(Ordering::SeqCst)),
                    next: AtomicPtr::new(node.next.load(Ordering::SeqCst)),
                    prev: AtomicPtr::new(core::ptr::null_mut()),
        };

        return FrameAllocator {
            linked_list: Some(LinkedList {
                start: Some(AlignedNode {
                    value: AtomicU64::new(node.value.load(Ordering::SeqCst)),
                    next: AtomicPtr::new(node.next.load(Ordering::SeqCst)),
                    prev: AtomicPtr::new(core::ptr::null_mut()),
                }),
                end: Some(AlignedNode {
                    value: AtomicU64::new(last.value.load(Ordering::SeqCst)),
                    next: AtomicPtr::new(last.next.load(Ordering::SeqCst)),
                    prev: AtomicPtr::new(last.prev.load(Ordering::SeqCst)),
                }),
            })
        };
    }

}
