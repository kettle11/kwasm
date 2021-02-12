use std::ffi::c_void;
mod panic_hook;
pub use panic_hook::setup_panic_hook;

pub type Command = u32;

pub fn log(string: &str) {
    #[allow(unused_unsafe)]
    unsafe {
        imported::kwasm_host_log_string(string as *const str as *const c_void, string.len())
    }
}

pub fn log_error(string: &str) {
    #[allow(unused_unsafe)]
    unsafe {
        imported::kwasm_host_log_error(string as *const str as *const c_void, string.len())
    }
}

mod imported {
    use super::*;
    use std::ffi::c_void;

    #[cfg(feature = "wasm_bindgen_support")]
    use wasm_bindgen::prelude::*;

    #[cfg_attr(
        feature = "wasm_bindgen_support",
        wasm_bindgen(module = "/js/kwasm.js")
    )]
    extern "C" {
        pub fn kwasm_host_log_string(source: *const c_void, source_length: usize);
        pub fn kwasm_host_log_error(source: *const c_void, source_length: usize);
        pub fn kwasm_host_receive_message(
            library: u32,
            command: Command,
            data: *mut c_void,
            data_length: u32,
        ) -> u32;
    }

    #[cfg_attr(
        feature = "wasm_bindgen_support",
        wasm_bindgen(module = "/js/kwasm.js")
    )]
    #[cfg(feature = "wasm_bindgen_support")]
    extern "C" {
        pub fn kwasm_set_memory_and_exports();
    }
}

/// This is a horrible hack.
/// wasm-bindgen immediately calls main if this isn't here, this gives kwasm a chance
/// to setup and then main can be called from the Javascript side.
/// It'd be nice to remove this.
#[cfg(feature = "wasm_bindgen_support")]
use wasm_bindgen::prelude::*;
#[cfg_attr(feature = "wasm_bindgen_support", wasm_bindgen(start))]
pub fn kwasm_fake_start() {}

#[cfg(feature = "wasm_bindgen_support")]
#[no_mangle]
pub unsafe extern "C" fn kwasm_set_memory_and_exports_rust() {
    imported::kwasm_set_memory_and_exports()
}

use std::cell::RefCell;

thread_local! {
    /// Data sent from the host.
    /// Unique to this Wasm thread.
    pub static DATA_FROM_HOST: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

#[no_mangle]
pub extern "C" fn kwasm_reserve_space(space: usize) -> *mut u8 {
    DATA_FROM_HOST.with(|d| {
        let mut d = d.borrow_mut();
        d.clear();
        d.resize(space, 0);
        d.as_mut_ptr()
    })
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct KWasmLibrary(u32);
impl KWasmLibrary {
    pub const fn null() -> KWasmLibrary {
        KWasmLibrary(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn new(source: &str) -> Self {
        #[allow(unused_unsafe)]
        unsafe {
            let library = imported::kwasm_host_receive_message(
                1, // Library 1 is used for declaring other libraries.
                0,
                source as *const str as *const c_void as *mut c_void,
                source.len() as u32,
            );
            KWasmLibrary(library)
        }
    }

    pub fn send_message_to_host(&self, command: Command) -> u32 {
        #[allow(unused_unsafe)]
        unsafe {
            imported::kwasm_host_receive_message(self.0, command, std::ptr::null_mut(), 0)
        }
    }

    pub fn send_message_with_data_to_host<T>(&self, command: Command, data: &[T]) -> u32 {
        #[allow(unused_unsafe)]
        unsafe {
            imported::kwasm_host_receive_message(
                self.0,
                command,
                data.as_ptr() as *mut c_void,
                (std::mem::size_of::<T>() * data.len()) as u32,
            )
        }
    }
}
