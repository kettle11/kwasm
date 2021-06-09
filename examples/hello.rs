use kwasm::libraries::*;
use kwasm::*;

fn main() {
    setup_panic_hook();
    console::log("HELLO WORLD!");

    //const document_str: JSString = JSString::new("document");
    const CONSOLE_STR: JSString = JSString::new("console");
    const LOG_STR: JSString = JSString::new("log");

    let console = JS_SELF.get_property(&CONSOLE_STR);
    let log_function = console.get_property(&LOG_STR);

    const MESSAGE: JSString = JSString::new("HI WORLD!!!");
    log_function.call_1_arg(&console, &MESSAGE);

    console::log("LOGGING FROM THE CONSOLE");

    eval("console.log('EVAL SEEMS TO WORK')");
}
