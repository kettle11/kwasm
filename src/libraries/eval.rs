use crate::*;

thread_local! {
    static EVAL_FUNCTION: JSObject = JSObject::null();
}

pub fn eval(source: &str) -> Option<JSObject> {
    let source_str: JSString = JSString::new(source);

    EVAL_FUNCTION.with(|e| {
        if e.is_null() {
            const EVAL_STR: JSString = JSString::new("eval");
            e.swap(&JS_SELF.get_property(&EVAL_STR));
        }
        e.call_1_arg(&JSObject::null(), &source_str)
    })
}
