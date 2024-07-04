use crate::pico_sdk;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi;

/// The global allocator type.
#[derive(Default)]
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    pico_sdk::malloc(layout.size() as u32) as *mut u8
  }
  unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
    pico_sdk::free(ptr as *mut ffi::c_void);
  }
}

// TODO: Change as CortexMHeap
/// The static global allocator.
#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;
