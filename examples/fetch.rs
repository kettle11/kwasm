use kwasm::*;

fn main() {
    setup_panic_hook();
    ktasks::create_workers(4);
    let _ = libraries::fetch::fetch("Hello");
}
