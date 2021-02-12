// This is used to decode strings passed from Wasm to Javascript.
const decoder = new TextDecoder();
const encoder = new TextEncoder();

// The first library is used for null, the second is used by messages that create new libraries.
var librarys = [undefined, undefined];

var kwasm_memory;
var kwasm_exports;

// Used when working with wasm-bindgen
export function kwasm_set_memory_and_exports() {
    kwasm_memory = document.kwasm_memory;
    kwasm_exports = document.kwasm_exports;
}

// Load and setup the WebAssembly library.
function initialize(wasm_library_path) {
    let env = {
        memory: new WebAssembly.Memory({ initial: 32, maximum: 16384, shared: true }),
        kwasm_host_log_string: kwasm_host_log_string,
        kwasm_host_log_error: kwasm_host_log_error,
        kwasm_host_receive_message: kwasm_host_receive_message,
    };

    let imports = {
        env: env
    };

    fetch(wasm_library_path).then(response =>
        response.arrayBuffer()
    ).then(bytes =>
        WebAssembly.instantiate(bytes, imports)
    ).then(results => {
        kwasm_memory = results.instance.exports.memory;
        kwasm_exports = results.instance.exports;
        // Call our start function.
        results.instance.exports.main();
    });
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

export function kwasm_host_log_string(pointer, length) {
    const string_data = new Uint8Array(new Uint8Array(kwasm_memory.buffer, pointer, length));
    const decoder = new TextDecoder();
    const string = decoder.decode(string_data);
    console.log(string);
}

export function kwasm_host_log_error(pointer, length) {
    const string_data = new Uint8Array(new Uint8Array(kwasm_memory.buffer, pointer, length));
    const string = decoder.decode(string_data);
    console.error(string);
}

export function kwasm_host_receive_message(library, command, data, data_length) {
    // Any message to library 1 is interpreted as a new library declaration.
    // Library 0 is reserved for null.
    if (library == 1) {
        const string_data = new Uint8Array(new Uint8Array(kwasm_memory.buffer, data, data_length));
        const decoded_string = decoder.decode(string_data);

        let library = new Function("kwasm_exports", "kwasm_memory", "kwasm_helpers", decoded_string)(kwasm_exports, kwasm_memory, kwasm_helpers);
        return librarys.push(library) - 1;
    } else {
        const message_data = new Uint8Array(kwasm_memory.buffer, data, data_length);
        return (librarys[library])(command, message_data);
    }
}

export default initialize;