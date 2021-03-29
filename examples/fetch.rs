use kwasm::*;

fn main() {
    log("HELLO THERE");
    // setup_panic_hook();

    log(&format!(
        "AVAILABLE THREADS: {:?}",
        kwasm::available_threads()
    ));

    ktasks::create_workers(kwasm::available_threads());
    ktasks::spawn(async { kwasm::log("ON WORKER THREAD") }).run();
}
