use std::io::Write;

use kwasm::*;
thread_local! {
    static LIBRARY: KWasmLibrary = KWasmLibrary::new(include_str!("test.js"));
}
fn main() {
    kwasm::log("HELLO WORLD");
    let js_value = register_string("Hi there");
    log_string(js_value);
    log_string(js_value);
}

#[derive(Clone, Copy)]
struct JsValue(u32);
fn register_string(s: &str) -> JsValue {
    let mut args = [s.as_ptr() as u32, s.len() as u32];
    let result = LIBRARY.with(|l| l.message_with_slice(0, &mut args));
    JsValue(result)
}

fn log_string(s: JsValue) {
    let mut args = [s.0];
    LIBRARY.with(|l| l.message_with_slice(1, &mut args));

    <fn(JsValue)>::register_function("doThing");
}

trait Arg {
    const TYPE: u32;
    fn add_bytes(self, data: &mut Vec<u8>);
}

impl Arg for u32 {
    const TYPE: u32 = 1;
    fn add_bytes(self, data: &mut Vec<u8>) {
        data.write(&self.to_ne_bytes()).unwrap();
    }
}

impl Arg for f32 {
    const TYPE: u32 = 2;
    fn add_bytes(self, data: &mut Vec<u8>) {
        data.write(&self.to_ne_bytes()).unwrap();
    }
}

impl Arg for JsValue {
    const TYPE: u32 = 3;
    fn add_bytes(self, data: &mut Vec<u8>) {
        data.write(&self.0.to_ne_bytes()).unwrap();
    }
}

trait RegisterFunction<A> {
    type FunctionType;
    fn register_function(name: &str) -> Self::FunctionType;
}

impl<A: Arg, F: Fn(A)> RegisterFunction<A> for F {
    type FunctionType = Box<dyn Fn(A)>;
    fn register_function(name: &str) -> Self::FunctionType {
        let mut args = vec![name.as_ptr() as u32, name.len() as u32, A::TYPE];
        let result = LIBRARY.with(|l| l.message_with_slice(0, &mut args));
        Box::new(move |a: A| {
            let mut args = Vec::<u8>::new();
            result.add_bytes(&mut args);
            a.add_bytes(&mut args);
            LIBRARY.with(|l| l.message_with_slice(1, &mut args));
        })
    }
}

// Register associated functions could make it so the first arg is called.
// Registering functions this way means that on the JS side there will be one layer of indirection for the function
// call, but additional indirection when looking up arguments.
