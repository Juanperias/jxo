use core::alloc::{GlobalAlloc, Layout};

use core::fmt::Write;
use crate::println;

use crate::allocator::frame_allocator::get_frame_allocator;

#[derive(Debug)]
pub struct Header {
    pub block_size: usize,
    pub next_block: *mut Header,
    pub free: bool,
}

pub struct KernelAllocator {
    pub page: u64,
    pub start: *mut Header,

    pub pointer: u64,
}

impl KernelAllocator {
    pub fn init() -> Self {
        Self {
            page: unsafe { get_frame_allocator().alloc_page() },
            start: core::ptr::null_mut(),
            pointer: 0
        }
    }
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        unsafe {
        let size = {
            let s = size_of::<Header>() + layout.size();

            align_up(s, size_of::<usize>())
        };

        let header = Header {
            block_size: size,
            next_block: core::ptr::null_mut(),
            free: false,
        };

        let ptr = (self.page + self.pointer) as *mut Header;
                
        (*ptr) = header;

        if self.pointer > 4096 || (self.pointer + size as u64) > 4096 {
            return core::ptr::null_mut();
        }


        if self.start == core::ptr::null_mut() {   
            self.start = ptr;
        } else {
            let mut current = self.start;

            loop {
                if (*current).next_block == core::ptr::null_mut() {
                    break;
                }

                current = (*current).next_block;
            }

            (*current).next_block = ptr;
        }
   
        let pointer = self.pointer;

        self.pointer += size as u64;
     
        (self.page + pointer + size_of::<Header>() as u64) as *mut u8
        }
    }
    pub unsafe fn dealloc(&mut self, block: *mut u8) {
        unsafe {
        if self.start == core::ptr::null_mut() {
            return;
        }

        let mut current = self.start;

        while current.addr() + size_of::<Header>() != block.addr() {
            if (*current).next_block == core::ptr::null_mut() {
                break;
            }

            current = (*current).next_block;
            
        }
        (*current).free = true;

        }

    }
}

pub struct KernelAllocatorWrapper;

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

unsafe impl GlobalAlloc for KernelAllocatorWrapper {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        
    }
}
