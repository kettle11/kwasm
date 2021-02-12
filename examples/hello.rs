use kwasm::*;
fn main() {
    let library = KWasmLibrary::new(include_str!("hello.js"));
    library.send_message_to_host(0);
}
