use std::cell::RefCell;

// For SharedArrayBuffers (and this example) to the correct cross-origin flags must be set.
// This is how to set those flags for a server hosted locally (like `devserver`)
// devserver --header Cross-Origin-Opener-Policy='same-origin' --header Cross-Origin-Embedder-Policy='require-corp'
use kwasm::*;

fn main() {
    libraries::web_worker::spawn(|| {
        log("In worker");
    });
}
