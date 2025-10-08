use core::sync::atomic::{AtomicPtr, AtomicU64};

#[derive(Debug)]
//#[repr(align(4096))]
pub struct AlignedNode {
    pub value: AtomicU64,
    pub next: AtomicPtr<AlignedNode>,
    pub prev: AtomicPtr<AlignedNode>,
}
