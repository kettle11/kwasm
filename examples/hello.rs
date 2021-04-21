use kwasm::*;
fn main() {
    setup_panic_hook();
    kwasm::log("HELLO WORLD!");
    let library = KWasmLibrary::new(include_str!("hello.js"));
    library.message(0);
}
