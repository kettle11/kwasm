use crate::*;

thread_local! {
    static CONSOLE: ConsoleInner = ConsoleInner::new()
}

pub struct ConsoleInner {
    console_js_object: JSObject,
    log_js_object: JSObject,
    error_js_object: JSObject,
}

impl ConsoleInner {
    pub const fn new() -> Self {
        Self {
            console_js_object: JSObject::null(),
            error_js_object: JSObject::null(),
            log_js_object: JSObject::null(),
        }
    }

    fn initialize(&self) {
        const CONSOLE_STR: JSString = JSString::new("console");
        const LOG_STR: JSString = JSString::new("log");

        self.console_js_object
            .swap(&JS_SELF.get_property(&CONSOLE_STR));
        self.error_js_object
            .swap(&self.console_js_object.get_property(&JSString::new("error")));
        self.log_js_object
            .swap(&self.console_js_object.get_property(&LOG_STR));
    }

    pub fn log(&self, string: &str) {
        let js_string_object = JSString::new(string);
        self.log_js_string(&js_string_object);
    }

    pub fn log_error(&self, string: &str) {
        let js_string_object = JSString::new(string);
        self.error_js_string(&js_string_object);
    }

    pub fn log_js_string(&self, js_string: &JSString) {
        if self.console_js_object.is_null() {
            self.initialize()
        }
        self.log_js_object
            .call_1_arg(&self.console_js_object, js_string.get_js_object());
    }

    pub fn error_js_string(&self, js_string: &JSString) {
        if self.console_js_object.is_null() {
            self.initialize()
        }
        self.error_js_object
            .call_1_arg(&self.console_js_object, js_string.get_js_object());
    }
}

pub fn log(string: &str) {
    CONSOLE.with(|c| c.log(string))
}

pub fn log_error(string: &str) {
    CONSOLE.with(|c| c.log_error(string))
}

pub fn log_js_string(js_string: &JSString) {
    CONSOLE.with(|c| c.log_js_string(js_string))
}
