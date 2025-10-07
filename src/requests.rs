use limine::request::{RequestsStartMarker, RequestsEndMarker, FramebufferRequest, HhdmRequest};

pub static START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

pub static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();

pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();


pub static END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
