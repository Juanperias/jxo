use crate::structures::once::Once;

pub mod frame_allocator;
pub mod kernel_allocator;

pub static HHDM: Once<u64> = Once::new();
