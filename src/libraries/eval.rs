use crate::*;

thread_local! {
    static EVAL_FUNCTION: JSObjectFromString = JSObjectFromString::new("self.eval")
}

pub fn eval(source: &str) -> Option<JSObject> {
    let source_str: JSString = JSString::new(source);
    EVAL_FUNCTION.with(|e| e.call_1_arg(&JSObject::null(), &source_str))
}
