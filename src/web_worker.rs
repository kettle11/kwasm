use crate::*;
use std::usize;
#[allow(unused)]
use wasm_set_stack_pointer;

const WASM_PAGE_SIZE: usize = 1024 * 64;

struct WorkerData {
    entry_point: Option<Box<dyn FnOnce() + Send + 'static>>,
    stack_memory: *mut u8,
    stack_size: usize,
    thread_local_storage_memory: *mut u8,
}

impl Drop for WorkerData {
    fn drop(&mut self) {
        // Deallocate this worker's stack and thread local storage.
        unsafe {
            let stack_layout =
                core::alloc::Layout::from_size_align(self.stack_size, WASM_PAGE_SIZE).unwrap();
            let thread_local_storage_layout = core::alloc::Layout::from_size_align(
                THREAD_LOCAL_STORAGE_SIZE as usize,
                THREAD_LOCAL_STORAGE_ALIGNMENT as usize,
            )
            .unwrap();
            std::alloc::dealloc(
                self.thread_local_storage_memory,
                thread_local_storage_layout,
            );
            // Is it ok to deallocate the stack memory here?
            std::alloc::dealloc(self.stack_memory, stack_layout);
        }
    }
}

pub fn spawn<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    unsafe {
        let f = Box::new(f) as Box<dyn FnOnce() + Send + 'static>;

        let stack_size = 1 << 20; // 1 MB stack size.

        let (stack_memory, stack_pointer, thread_local_storage_memory) = {
            let stack_layout =
                core::alloc::Layout::from_size_align(stack_size, WASM_PAGE_SIZE).unwrap();
            // The stack pointer should be set to the other end.
            // An unfortunate consequence of this design is that a stack overflow will just corrupt the rest of the WASM heap.
            // I suppose a warning buffer of sorts could be set and checked from time to time, but
            // that's not worth implementing now.
            let stack_memory = std::alloc::alloc(stack_layout);

            let thread_local_storage_memory = kwasm_alloc_thread_local_storage() as *mut u8;
            (
                stack_memory,
                stack_memory.offset(stack_size as isize),
                thread_local_storage_memory,
            )
        };

        let worker_data = Box::new(WorkerData {
            entry_point: Some(f),
            stack_memory,
            stack_size,
            thread_local_storage_memory,
        });

        // Pass the closure and stack pointer
        let mut message_data: [u32; 3] = [
            Box::leak(worker_data) as *mut _ as *mut std::ffi::c_void as u32,
            stack_pointer as *mut std::ffi::c_void as u32,
            thread_local_storage_memory as *mut std::ffi::c_void as u32,
        ];

        HOST_LIBRARY.message_with_slice(6, &mut message_data);
    }
}

#[no_mangle]
extern "C" fn kwasm_web_worker_entry_point(callback: u32) {
    unsafe {
        let mut b: Box<WorkerData> = Box::from_raw(callback as *mut std::ffi::c_void as *mut _);
        (b.entry_point.take().unwrap())()
    }
}
