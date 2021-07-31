use std::{cell::Cell, ffi::c_void, ops::Deref};

#[cfg(feature = "wasm_bindgen_support")]
use wasm_bindgen::prelude::*;

#[cfg_attr(
    feature = "wasm_bindgen_support",
    wasm_bindgen(module = "/js/kwasm.js")
)]
extern "C" {
    pub(crate) fn kwasm_new_string(data: *const u8, data_length: u32) -> u32;
    pub(crate) fn kwasm_free_js_object(object: u32);
    pub(crate) fn kwasm_js_object_property(function_object: u32, property: u32) -> u32;
    pub(crate) fn kwasm_get_js_object_value_u32(object: u32) -> u32;
    pub(crate) fn kwasm_call_js_with_args(
        function_object: u32,
        this: u32,
        args_data: *const c_void,
        data_length: u32,
    ) -> u32;
    pub(crate) fn kwasm_call_js_with_args_raw(
        function_object: u32,
        this: u32,
        args_data: *const c_void,
        data_length: u32,
    ) -> u32;
    #[cfg(target_feature = "atomics")]
    pub(crate) fn kwasm_new_worker(
        entry_point: u32,
        stack_pointer: u32,
        thread_local_storage_pointer: u32,
    );
}

fn kwasm_call_js_with_args0(function_object: u32, this: u32, args: &[u32]) -> u32 {
    unsafe {
        kwasm_call_js_with_args(
            function_object,
            this,
            args.as_ptr() as *const c_void,
            args.len() as u32,
        )
    }
}

fn kwasm_call_js_with_args_raw0(function_object: u32, this: u32, args: &[u32]) -> u32 {
    unsafe {
        kwasm_call_js_with_args_raw(
            function_object,
            this,
            args.as_ptr() as *const c_void,
            args.len() as u32,
        )
    }
}

/// Window.self
/// Accesses the global scope.
/// https://developer.mozilla.org/en-US/docs/Web/API/Window/self
pub const JS_SELF: JSObject = JSObject {
    index: Cell::new(1),
};

#[derive(Debug, Clone)]
pub struct JSObject {
    index: Cell<u32>,
}

impl JSObject {
    pub const NULL: Self = JSObject {
        index: Cell::new(0),
    };

    pub fn get_property(&self, string: &str) -> Self {
        let string = JSString::new(string);
        unsafe {
            Self {
                index: Cell::new(kwasm_js_object_property(
                    self.index.get(),
                    string.index.get(),
                )),
            }
        }
    }

    pub fn index(&self) -> u32 {
        self.index.get()
    }

    // If this value is a u32, return it as a u32
    pub fn get_value_u32(&self) -> u32 {
        unsafe { kwasm_get_js_object_value_u32(self.index.get()) }
    }

    /// Replaces the inner JSObject with the new JSObject.
    pub fn swap(&self, object: &JSObject) {
        self.index.swap(&object.index)
    }

    pub fn is_null(&self) -> bool {
        self.index.get() == 0
    }

    pub const fn new_raw(index: u32) -> Self {
        Self {
            index: Cell::new(index),
        }
    }

    #[inline]
    fn check_result(result: u32) -> Option<JSObject> {
        if result == 0 {
            None
        } else {
            Some(JSObject {
                index: Cell::new(result),
            })
        }
    }

    /// Call a function with each u32 passed as a separate argument to the JavaScript side.
    pub fn call_raw(&self, this: &JSObject, args: &[u32]) -> Option<Self> {
        let result = kwasm_call_js_with_args_raw0(self.index.get(), this.index.get(), args);
        Self::check_result(result)
    }

    /// Call this as a function with one arg.
    pub fn call(&self, this: &JSObject) -> Option<Self> {
        let result = kwasm_call_js_with_args0(self.index.get(), this.index.get(), &[]);
        Self::check_result(result)
    }

    /// Call this as a function with one arg.
    pub fn call_1_arg(&self, this: &JSObject, argument: &JSObject) -> Option<Self> {
        let result =
            kwasm_call_js_with_args0(self.index.get(), this.index.get(), &[argument.index.get()]);

        Self::check_result(result)
    }

    /// Call this as a function with one arg.
    pub fn call_2_arg(&self, this: &JSObject, arg0: &JSObject, arg1: &JSObject) -> Option<Self> {
        let result = kwasm_call_js_with_args0(
            self.index.get(),
            this.index.get(),
            &[arg0.index.get(), arg1.index.get()],
        );

        Self::check_result(result)
    }
}

impl Drop for JSObject {
    fn drop(&mut self) {
        unsafe { kwasm_free_js_object(self.index.get()) }
    }
}

pub struct JSString<'a> {
    string: &'a str,
    js_object: JSObject,
}

impl<'a> JSString<'a> {
    pub const fn new(string: &'a str) -> Self {
        JSString {
            string,
            js_object: JSObject::NULL,
        }
    }
    pub fn new_from_string(string: String) -> Self {
        let js_object =
            JSObject::new_raw(unsafe { kwasm_new_string(string.as_ptr(), string.len() as u32) });
        JSString {
            string: "",
            js_object,
        }
    }
}

impl<'a> Deref for JSString<'a> {
    type Target = JSObject;
    fn deref(&self) -> &Self::Target {
        if self.js_object.is_null() {
            let new_object = JSObject::new_raw(unsafe {
                kwasm_new_string(self.string.as_ptr(), self.string.len() as u32)
            });
            self.js_object.swap(&new_object)
        }
        &self.js_object
    }
}
