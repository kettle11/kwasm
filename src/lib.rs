//! Kwasm helps Rust code interact with a web-browser host environment 
//! in a light-weight and reusable way.
//! The library also helps facilitate multi-threaded browser code.
//! It can work alongside `wasm-bindgen` or stand-alone.
//! Kwasm uses eval to initialize Javascript code from Rust libraries.

use std::cell::Cell;
use std::cell::RefCell;
use std::ffi::c_void;

#[cfg(any(feature = "wasm_bindgen_support", target_feature = "atomics"))]
use std::sync::Once;

pub mod libraries {
    pub mod console;
    pub mod fetch;
    pub use console::*;
    pub use fetch::*;
}

mod js_object;
mod panic_hook;

pub use js_object::*;

#[cfg(target_feature = "atomics")]
pub mod web_worker;

pub type Command = u32;
pub use panic_hook::setup_panic_hook;

pub(crate) const HOST_LIBRARY: KWasmLibrary = KWasmLibrary {
    library: Cell::new(1),
    source: "",
};

thread_local! {
    /// Data sent from the host.
    /// Unique to this Wasm thread.
    pub static DATA_FROM_HOST: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

pub fn log(string: &str) {
    #[cfg(feature = "wasm_bindgen_support")]
    initialize_kwasm_for_wasmbindgen();
    HOST_LIBRARY.message_with_ptr(1, string.as_ptr() as *mut u8, string.len() as u32);
}

pub fn log_error(string: &str) {
    #[cfg(feature = "wasm_bindgen_support")]
    initialize_kwasm_for_wasmbindgen();
    HOST_LIBRARY.message_with_ptr(2, string.as_ptr() as *mut u8, string.len() as u32);
}

/// This will return 1 for pages that are not cross-origin isolated, or for browsers
/// that don't support SharedArrayBuffer.
/// See here for more info about Cross Origin Isolation: https://web.dev/cross-origin-isolation-guide/
pub fn available_threads() -> u32 {
    HOST_LIBRARY.message(5)
}

#[cfg(feature = "wasm_bindgen_support")]
use wasm_bindgen::prelude::*;

#[cfg_attr(
    feature = "wasm_bindgen_support",
    wasm_bindgen(module = "/js/kwasm.js")
)]
extern "C" {
    pub fn kwasm_message_to_host(
        library: u32,
        command: Command,
        data: *mut c_void,
        data_length: u32,
    ) -> u32;
}

/// Called by the host to reserve scratch space to pass data into kwasm.
/// returns a pointer to the allocated data.
#[no_mangle]
pub extern "C" fn kwasm_reserve_space(space: usize) -> *mut u8 {
    DATA_FROM_HOST.with(|d| {
        let mut d = d.borrow_mut();
        d.clear();
        d.resize(space, 0);
        d.as_mut_ptr()
    })
}
#[cfg(feature = "wasm_bindgen_support")]
fn initialize_kwasm_for_wasmbindgen() {
    static THREAD_LOCAL_STORAGE_METADATA_INIT: Once = Once::new();
    THREAD_LOCAL_STORAGE_METADATA_INIT.call_once(|| {
        // Smuggle out the Wasm instance's exports right from under `wasm-bindgen`'s nose.
        js_sys::eval("self.kwasm_exports = wasm.exports;").unwrap();

        #[cfg_attr(
            feature = "wasm_bindgen_support",
            wasm_bindgen(module = "/js/kwasm.js")
        )]
        extern "C" {
            pub fn kwasm_initialize_wasmbindgen(module: JsValue, function_table: JsValue);
        }
        kwasm_initialize_wasmbindgen(wasm_bindgen::module(), wasm_bindgen::memory());
    });
}

/// A KWasm library is a light-weight way to communicate with the Javascript of the host browser.
/// Declare a KWasm library like so:
/// ``` static LIBRARY: KWasmLibrary = KWasmLibrary::new(include_str!("fetch.js"));
#[repr(C)]
pub struct KWasmLibrary {
    library: Cell<u32>,
    source: &'static str,
}

impl KWasmLibrary {
    pub const fn new(source: &'static str) -> Self {
        Self {
            library: Cell::new(0),
            source,
        }
    }

    fn message_inner(&self, command: Command, data: *mut c_void, data_length: u32) -> u32 {
        #[cfg(feature = "wasm_bindgen_support")]
        initialize_kwasm_for_wasmbindgen();

        #[allow(unused_unsafe)]
        unsafe {
            if self.library.get() == 0 {
                self.library.set(kwasm_message_to_host(
                    1,
                    0,
                    self.source.as_ptr() as *mut u8 as *mut _,
                    self.source.len() as u32,
                ));
            }
            kwasm_message_to_host(self.library.get(), command, data, data_length)
        }
    }

    pub fn message(&self, command: Command) -> u32 {
        self.message_inner(command, std::ptr::null_mut(), 0)
    }

    pub fn message_with_ptr<T>(&self, command: Command, data: *mut T, len: u32) -> u32 {
        self.message_inner(command, data as *mut c_void, len)
    }

    pub fn message_with_slice<T>(&self, command: Command, data: &mut [T]) -> u32 {
        self.message_inner(
            command,
            data.as_mut_ptr() as *mut c_void,
            (std::mem::size_of::<T>() * data.len()) as u32,
        )
    }

    /*
    /// Sends a message with a pointer to the &mut T and data length set to the size of T.
    /// This is useful for quickly sending a data structure from the stack.
    pub fn message_with_ref<T>(&self, command: Command, data: &mut T) -> u32 {
        #[allow(unused_unsafe)]
        unsafe {
            kwasm_message_to_host(
                self.0,
                command,
                data as &mut T as *mut T as *mut c_void,
                std::mem::size_of::<T>() as u32,
            )
        }
    }
    */
}

// The main thread needs its thread local storage initialized.
// Web Workers will also use this to allocate their own thread local storage which is deallocated
// when the worker is dropped.
#[cfg(target_feature = "atomics")]
pub(crate) static mut THREAD_LOCAL_STORAGE_SIZE: u32 = 0;
#[cfg(target_feature = "atomics")]
pub(crate) static mut THREAD_LOCAL_STORAGE_ALIGNMENT: u32 = 0;
#[cfg(target_feature = "atomics")]
static THREAD_LOCAL_STORAGE_METADATA_INIT: Once = Once::new();

#[cfg(target_feature = "atomics")]
#[no_mangle]
pub(crate) extern "C" fn kwasm_alloc_thread_local_storage() -> u32 {
    unsafe {
        THREAD_LOCAL_STORAGE_METADATA_INIT.call_once(|| {
            // Command 3 gets thread local storage size, 4 gets thread local storage alignment.
            THREAD_LOCAL_STORAGE_SIZE = HOST_LIBRARY.message(3);
            THREAD_LOCAL_STORAGE_ALIGNMENT = HOST_LIBRARY.message(4);
        });

        let thread_local_storage_layout = core::alloc::Layout::from_size_align(
            THREAD_LOCAL_STORAGE_SIZE as usize,
            THREAD_LOCAL_STORAGE_ALIGNMENT as usize,
        )
        .unwrap();
        std::alloc::alloc(thread_local_storage_layout) as u32
    }
}

/// This is a horrible hack.
/// wasm-bindgen immediately calls main if this isn't here, this gives kwasm a chance
/// to setup and then main can be called from the Javascript side.
/// It'd be nice to remove this.
/// This could be skipped when using `wasm-bindgen` without workers.
#[cfg_attr(feature = "wasm_bindgen_support", wasm_bindgen(start))]
pub fn kwasm_fake_start() {}
