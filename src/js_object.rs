use std::cell::Cell;

#[cfg_attr(
    feature = "wasm_bindgen_support",
    wasm_bindgen(module = "/js/kwasm.js")
)]
extern "C" {
    pub fn kwasm_new_string(data: *const u8, data_length: u32) -> u32;
    pub fn kwasm_free_js_object(object: u32);
    pub fn kwasm_js_object_property(object: u32, property: u32) -> u32;
    pub fn kwasm_call_js_object_1_args(object: u32, arg0: u32) -> u32;
    pub fn kwasm_call_js_object_2_args(object: u32, arg0: u32, arg1: u32) -> u32;
    pub fn kwasm_call_js_object_3_args(object: u32, arg0: u32, arg2: u32) -> u32;
}

/// Window.self
/// Accesses the global scope.
/// https://developer.mozilla.org/en-US/docs/Web/API/Window/self
pub const JS_SELF: JSObject = JSObject {
    index: Cell::new(1),
};

pub struct JSObject {
    index: Cell<u32>,
}

impl JSObject {
    pub fn get_property(&self, string: &JSString) -> Self {
        unsafe {
            Self {
                index: Cell::new(kwasm_js_object_property(
                    self.index.get(),
                    string.get_js_object().index.get(),
                )),
            }
        }
    }

    /// Replaces the inner JSObject with the new JSObject.
    pub(crate) fn swap(&self, object: &JSObject) {
        self.index.swap(&object.index)
    }

    pub(crate) fn is_null(&self) -> bool {
        self.index.get() == 0
    }

    pub(crate) const fn null() -> Self {
        Self {
            index: Cell::new(0),
        }
    }

    pub(crate) const fn new_raw(index: u32) -> Self {
        Self {
            index: Cell::new(index),
        }
    }

    /// Call this as a function with one arg.
    pub fn call_1_arg(&self, argument: &impl JSObjectTrait) -> Option<Self> {
        let result = unsafe {
            kwasm_call_js_object_1_args(self.index.get(), argument.get_js_object().index.get())
        };
        if result == 0 {
            None
        } else {
            Some(JSObject {
                index: Cell::new(result),
            })
        }
    }
}

pub struct JSFunction(JSObject);

impl JSFunction {}

impl Drop for JSObject {
    fn drop(&mut self) {
        unsafe { kwasm_free_js_object(self.index.get()) }
    }
}

pub trait JSObjectTrait {
    fn get_js_object(&self) -> &JSObject;
}

impl JSObjectTrait for JSObject {
    fn get_js_object(&self) -> &JSObject {
        self
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
            js_object: JSObject::null(),
        }
    }
}

impl<'a> JSObjectTrait for JSString<'a> {
    /// This function defers creation of the JSString until it's actually needed.
    fn get_js_object(&self) -> &JSObject {
        if self.js_object.is_null() {
            let new_object = JSObject::new_raw(unsafe {
                kwasm_new_string(self.string.as_ptr(), self.string.len() as u32)
            });
            self.js_object.swap(&new_object)
        }
        &self.js_object
    }
}
