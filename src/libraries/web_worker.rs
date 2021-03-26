#[allow(unused)]
use wasm_set_stack_pointer;

use crate::*;
use std::sync::Once;

static mut WORKER_LIBRARY: KWasmLibrary = KWasmLibrary::null();
static LIBRARY_INIT: Once = Once::new();

pub const WASM_PAGE_SIZE: usize = 1024 * 64;

pub fn spawn<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    unsafe {
        LIBRARY_INIT.call_once(|| {
            WORKER_LIBRARY = KWasmLibrary::new(include_str!("web_worker.js"));
        });

        let f = Box::new(Box::new(f) as Box<dyn FnOnce() + Send + 'static>);
        let f = Box::leak(f);

        let stack = {
            let stack_size = 1 << 20; // 1 MB stack size.
            let stack_layout =
                core::alloc::Layout::from_size_align(stack_size, WASM_PAGE_SIZE).unwrap();
            // The stack pointer should be set to the other end.
            // An unfortunate consequence of this design is that a stack overflow will just corrupt the rest of the WASM heap.
            // I suppose a warning buffer of sorts could be set and checked from time to time, but
            // that's not worth implementing now.
            std::alloc::alloc(stack_layout).offset(stack_size as isize)
        };

        // Pass the closure and stack pointer
        let mut message_data: [u32; 2] = [
            f as *mut _ as *mut std::ffi::c_void as u32,
            stack as *mut std::ffi::c_void as u32,
        ];

        WORKER_LIBRARY.message_with_slice(0, &mut message_data);
    }
}

#[no_mangle]
extern "C" fn kwasm_web_worker_entry_point(callback: u32) {
    unsafe {
        log(&format!("pointer: {:?}", callback));
        let callback_box_ptr = callback as *mut std::ffi::c_void as *mut _;
        let b: Box<Box<dyn FnOnce() + Send + 'static>> = Box::from_raw(callback_box_ptr);
        b()
    }
}
