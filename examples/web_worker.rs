use kwasm::*;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    libraries::web_worker::spawn(|| {
        log("In worker");
        // panic!("HERE");
    });
}
