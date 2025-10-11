use core::{alloc::{GlobalAlloc, Layout}, cell::SyncUnsafeCell, sync::atomic::{AtomicPtr, Ordering}};

use crate::allocator::frame_allocator::get_frame_allocator;

#[derive(Debug)]
pub struct Header {
    pub block_size: usize,
    pub next_block: *mut Header,
    pub free: bool,
}

pub const MAX_ALLOCATION_SIZE: u64 = 4096;

pub static KERNEL_ALLOCATOR: SyncUnsafeCell<Option<KernelAllocator>> = SyncUnsafeCell::new(None);

pub fn init_kernel_allocator() {
    unsafe {
        *KERNEL_ALLOCATOR.get() = Some(KernelAllocator::init());
    }
}

pub fn get_kernel_allocator() -> &'static mut KernelAllocator {
    unsafe { KERNEL_ALLOCATOR.get().as_mut().unwrap().as_mut().unwrap() }
}


pub struct KernelAllocator {
    pub page: u64,
    pub start: AtomicPtr<Header>,
    pub pointer: u64,
}

impl KernelAllocator {
    pub fn init() -> Self {
        Self {
            page: unsafe { get_frame_allocator().alloc_page() },
            start: AtomicPtr::new(core::ptr::null_mut()),
            pointer: 0,
        }
    }
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        unsafe {
            let size = {
                let s = size_of::<Header>() + layout.size();

                align_up(s, size_of::<usize>())
            };

            if self.start.load(Ordering::SeqCst) != core::ptr::null_mut() {
                let mut current = self.start.load(Ordering::SeqCst);

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

            if self.pointer > MAX_ALLOCATION_SIZE || (self.pointer + size as u64) > MAX_ALLOCATION_SIZE {
                return core::ptr::null_mut();
            }

            if self.start.load(Ordering::SeqCst).is_null() {
                self.start.store(ptr, Ordering::SeqCst);
            } else {
                let mut current = self.start.load(Ordering::SeqCst);

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
            if self.start.load(Ordering::SeqCst).is_null() {
                return;
            }

            let mut current = self.start.load(Ordering::SeqCst);
            let mut prev: *mut Header = core::ptr::null_mut();

            while current.addr() + core::mem::size_of::<Header>() != block.addr() {
                if (*current).next_block.is_null() {
                    return;
                }
                prev = current;
                current = (*current).next_block;
            }

            if current == self.start.load(Ordering::SeqCst) {
                if !(*self.start.load(Ordering::SeqCst)).next_block.is_null() {
                    self.start.store((*self.start.load(Ordering::SeqCst)).next_block, Ordering::SeqCst);
                } else {
                    self.start.store(core::ptr::null_mut(), Ordering::SeqCst);
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
        unsafe {
            get_kernel_allocator().alloc(layout) 
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        unsafe {
            get_kernel_allocator().dealloc(ptr);
        }
    }
}
