use limine::request::{
    FramebufferRequest, HhdmRequest, MemoryMapRequest, RequestsEndMarker, RequestsStartMarker,
};

pub static START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

pub static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();

pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

pub static MEMORY_MAP: MemoryMapRequest = MemoryMapRequest::new();

pub static END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
