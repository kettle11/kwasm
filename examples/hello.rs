use kwasm::libraries::*;
use kwasm::*;

fn main() {
    setup_panic_hook();
    kwasm::log("HELLO WORLD!");
    let library = KWasmLibrary::new(include_str!("hello.js"));
    library.message(0);
    library.message(2);
    kwasm::DATA_FROM_HOST.with(|d| {
        let d = d.take();
        let s = std::str::from_utf8(&d).unwrap();
        kwasm::log(&format!("RECEIVED: {}", s))
    });

    //const document_str: JSString = JSString::new("document");
    const CONSOLE_STR: JSString = JSString::new("console");
    const LOG_STR: JSString = JSString::new("log");

    let console = JS_SELF.get_property(&CONSOLE_STR);
    let log_function = console.get_property(&LOG_STR);

    const MESSAGE: JSString = JSString::new("HI WORLD!!!");
    log_function.call_1_arg(&console, &MESSAGE);

    Console::log("LOGGING FROM THE CONSOLE");

    Console::log_js_string2(&JSString::new("HELLO0"), &JSString::new("HELLO1"));
}
