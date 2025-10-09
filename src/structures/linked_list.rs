use core::sync::atomic::{AtomicPtr, AtomicU64};

#[derive(Debug)]
pub struct AlignedNode {
    pub value: AtomicU64,
    pub next: AtomicPtr<AlignedNode>,
    pub prev: AtomicPtr<AlignedNode>,
}

impl AlignedNode {
    pub fn empty() -> Self {
        Self {
            value: AtomicU64::new(0),
            next: AtomicPtr::new(core::ptr::null_mut()),
            prev: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
}

#[derive(Debug)]
pub struct LinkedList {
    pub start: *mut AlignedNode,
    pub end: *mut AlignedNode,
}
