use std::cell::RefCell;
use std::ffi::c_void;

pub mod libraries {
    pub mod fetch;
    pub mod web_worker;
}

mod panic_hook;

pub type Command = u32;
pub use panic_hook::setup_panic_hook;

thread_local! {
    /// Data sent from the host.
    /// Unique to this Wasm thread.
    pub static DATA_FROM_HOST: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

pub fn log(string: &str) {
    #[allow(unused_unsafe)]
    unsafe {
        kwasm_message_to_host(
            1,
            1,
            string as *const str as *const c_void as *mut c_void,
            string.len() as u32,
        );
    }
}

pub fn log_error(string: &str) {
    #[allow(unused_unsafe)]
    unsafe {
        kwasm_message_to_host(
            1,
            2,
            string as *const str as *const c_void as *mut c_void,
            string.len() as u32,
        );
    }
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

#[cfg_attr(
    feature = "wasm_bindgen_support",
    wasm_bindgen(module = "/js/kwasm.js")
)]
#[cfg(feature = "wasm_bindgen_support")]
extern "C" {
    pub fn kwasm_set_memory_and_exports();
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

    /// Declare a library from JS source.
    pub fn new(source: &str) -> Self {
        #[allow(unused_unsafe)]
        unsafe {
            let library = kwasm_message_to_host(
                1, // Library 1 is used for declaring other libraries.
                0,
                source as *const str as *const c_void as *mut c_void,
                source.len() as u32,
            );
            KWasmLibrary(library)
        }
    }

    pub fn message(&self, command: Command) -> u32 {
        #[allow(unused_unsafe)]
        unsafe {
            kwasm_message_to_host(self.0, command, std::ptr::null_mut(), 0)
        }
    }

    pub fn message_with_ptr<T>(&self, command: Command, data: *mut T, len: u32) -> u32 {
        #[allow(unused_unsafe)]
        unsafe {
            kwasm_message_to_host(self.0, command, data as *mut c_void, len)
        }
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

    pub fn message_with_slice<T>(&self, command: Command, data: &mut [T]) -> u32 {
        #[allow(unused_unsafe)]
        unsafe {
            kwasm_message_to_host(
                self.0,
                command,
                data.as_mut_ptr() as *mut c_void,
                (std::mem::size_of::<T>() * data.len()) as u32,
            )
        }
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
    kwasm_set_memory_and_exports()
}
