use core::alloc::{GlobalAlloc, Layout};

use crate::println;
use core::fmt::Write;

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
    pub allocations: u64,

    pub pointer: u64,
}

impl KernelAllocator {
    pub fn init() -> Self {
        Self {
            page: unsafe { get_frame_allocator().alloc_page() },
            start: core::ptr::null_mut(),
            allocations: 0,
            pointer: 0,
        }
    }
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        unsafe {
            let size = {
                let s = size_of::<Header>() + layout.size();

                align_up(s, size_of::<usize>())
            };

            if self.start != core::ptr::null_mut() {
                let mut current = self.start;

                while !((*current).block_size == size && (*current).free) {
                    if (*current).next_block.is_null() {
                        current = core::ptr::null_mut();
                        break;
                    }

                    current = (*current).next_block;
                }

                if !current.is_null() {
                    (*current).free = false;
                    return (current.addr() as u64 + size_of::<Header>() as u64) as *mut u8;
                }
            }

            let header = Header {
                block_size: size,
                next_block: core::ptr::null_mut(),
                free: false,
            };

            let ptr = (self.page as *mut u8).add(self.pointer as usize) as *mut Header;

            (*ptr) = header;

            if self.pointer > 4096 || (self.pointer + size as u64) > 4096 {
                return core::ptr::null_mut();
            }

            if self.start.is_null() {
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
            self.allocations += 1;

            (self.page + pointer + size_of::<Header>() as u64) as *mut u8
        }
    }
    pub unsafe fn dealloc(&mut self, block: *mut u8) {
        unsafe {
            if self.start.is_null() {
                return;
            }

            let mut current = self.start;
            let mut prev: *mut Header = core::ptr::null_mut();

            while current.addr() + core::mem::size_of::<Header>() != block.addr() {
                if (*current).next_block.is_null() {
                    return;
                }
                prev = current;
                current = (*current).next_block;
            }

            if current == self.start {
                if !(*self.start).next_block.is_null() {
                    self.start = (*self.start).next_block;
                } else {
                    self.start = core::ptr::null_mut();
                }
            } else {
                if !prev.is_null() {
                    (*prev).next_block = (*current).next_block;
                }
            }

            self.pointer = self.pointer.wrapping_sub((*current).block_size as u64);
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
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {}
}
