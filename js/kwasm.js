// This is used to decode strings passed from Wasm to Javascript.
const decoder = new TextDecoder();
const encoder = new TextEncoder();

// The first library is used for null, the second is used by messages that create new libraries.
var kwasm_libraries = [undefined, undefined];

var kwasm_memory;
var kwasm_exports;
var kwasm_module;

let kwasm_imports = {
    env: {
        kwasm_message_to_host: kwasm_message_to_host,
    }
};

// Load and setup the WebAssembly library.
function initialize(wasm_library_path) {
    kwasm_memory = new WebAssembly.Memory({ initial: 32, maximum: 16384, shared: true });

    let imports = {
        env: {
            memory: kwasm_memory,
        }
    };
    imports.env = Object.assign(imports.env, kwasm_imports.env);

    fetch(wasm_library_path).then(response =>
        response.arrayBuffer()
    ).then(bytes =>
        WebAssembly.instantiate(bytes, imports)
    ).then(results => {
        kwasm_exports = results.instance.exports;
        kwasm_module = results.module;

        // I suspect this is called automatically an is unneeded to be called here.
        // In fact including this line results in an error in Firefox.
        // kwasm_exports.__wasm_init_memory();

        // Setup thread-local storage for the main thread
        const thread_local_storage = kwasm_exports.kwasm_alloc_thread_local_storage();
        kwasm_exports.__wasm_init_tls(thread_local_storage);

        // Call our start function.
        results.instance.exports.main();
    });
}

export function kwasm_message_to_host(library, command, data, data_length) {
    // Creates a view into the memory
    const message_data = new Uint8Array(kwasm_memory.buffer, data, data_length);

    // Library 0 is reserved for null.
    // Library 1 is used for built-in kwasm commands.
    if (library == 1) {
        switch (command) {
            case 0: {
                // Create a new library
                const decoded_string = decoder.decode(new Uint8Array(message_data));
                let library = new Function("kwasm_exports", "kwasm_memory", "kwasm_module", "kwasm_helpers", decoded_string)
                    (kwasm_exports, kwasm_memory, kwasm_module, kwasm_helpers);
                return kwasm_libraries.push(library) - 1;
            }
            case 1: {
                // Log
                const string = decoder.decode(new Uint8Array(message_data));
                console.log(string);
                return;
            }
            case 2: {
                // Log error
                const string = decoder.decode(new Uint8Array(message_data));
                console.error(string);
                return;
            }
            case 3:
                // Access the thread local storage size global variable created by LLVM.
                return kwasm_exports.__tls_size.value;
            case 4:
                // Access the thread local storage alignment global variable created by LLVM.
                return kwasm_exports.__tls_align.value;
        }

    } else {
        return (kwasm_libraries[library])(command, message_data);
    }
}

let kwasm_helpers = {
    pass_string_to_client: pass_string_to_client
};

// Used to pass strings back to the Rust side.
function pass_string_to_client(string) {
    const string_data = encoder.encode(string);
    let length = string_data.byteLength;
    let pointer = kwasm_exports.kwasm_reserve_space(length);
    const client_string = new Uint8Array(kwasm_memory.buffer, pointer, length);
    client_string.set(string_data);
}

// Used when working with wasm-bindgen
export function kwasm_set_memory_and_exports() {
    kwasm_memory = document.kwasm_memory;
    kwasm_exports = document.kwasm_exports;
}

export default initialize;