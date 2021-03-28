function kwasm_stuff() {
    // This is used to decode strings passed from Wasm to Javascript.
    const decoder = new TextDecoder();
    const encoder = new TextEncoder();

    // The first library is used for null, the second is used by messages that create new libraries.
    var kwasm_libraries = [undefined, undefined];
    var kwasm_memory;
    var kwasm_exports;
    var kwasm_module;

    function kwasm_message_to_host(library, command, data, data_length) {
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
                case 5:
                    return available_threads;
                case 6:
                    // This assumes that the Wasm pointer size is 32bits, which may not hold forever.
                    let entry_point_and_stack = new Uint32Array(message_data.buffer, message_data.byteOffset, message_data.length / 4);
                    create_worker(entry_point_and_stack[0], entry_point_and_stack[1], entry_point_and_stack[2]);
                    break;
            }

        } else {
            return (kwasm_libraries[library])(command, message_data);
        }
    }


    let kwasm_imports = {
        env: {
            kwasm_message_to_host: kwasm_message_to_host,
        }
    };

    var available_threads = 0;

    // Load and setup the WebAssembly library.
    function initialize(wasm_library_path) {
        available_threads = navigator.hardwareConcurrency;

        if (!crossOriginIsolated) {
            console.error("kwasm: Not Cross Origin Isolated! Number of available threads set to 1. \n SharedArrayBuffers may be disabled by the browser as well.")
            available_threads = 1;
        }

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


    // If we're a worker thread we'll use this.
    onmessage = function (e) {
        console.log('Running new worker thread');

        let imports = {
            env: {
                memory: kwasm_memory,
            }
        };
        imports.env = Object.assign(imports.env, kwasm_imports.env);

        imports.env.memory = e.data.kwasm_memory;

        kwasm_memory = e.data.kwasm_memory;

        WebAssembly.instantiate(e.data.kwasm_module, imports).then(results => {
            kwasm_exports = results.exports;
            kwasm_exports.set_stack_pointer(e.data.stack_pointer);
            kwasm_exports.__wasm_init_tls(e.data.thread_local_storage_pointer);
            kwasm_exports.kwasm_web_worker_entry_point(e.data.entry_point);
        });
    }

    function create_worker(entry_point, stack_pointer, thread_local_storage_pointer) {
        let worker = new Worker(kwasm_stuff_blob);
        worker.postMessage({
            kwasm_memory: kwasm_memory,
            kwasm_module: kwasm_module,
            entry_point: entry_point,
            stack_pointer: stack_pointer,
            thread_local_storage_pointer: thread_local_storage_pointer
        });
    }

    function run_on_worker() {
        onmessage = function (e) {
            importScripts(e.data);
        }
    }

    return { kwasm_message_to_host: kwasm_message_to_host, initialize: initialize };
}
const kwasm = kwasm_stuff();

var kwasm_stuff_blob = URL.createObjectURL(new Blob(
    ['(', kwasm_stuff.toString(), ')()'],
    { type: 'application/javascript' }
));


const kwasm_message_to_host = kwasm.kwasm_message_to_host;
const initialize = kwasm.initialize;
export { kwasm_message_to_host as kwasm_message_to_host, initialize as initialize };

// Used when working with wasm-bindgen
export function kwasm_set_memory_and_exports() {
    kwasm_memory = document.kwasm_memory;
    kwasm_exports = document.kwasm_exports;
}

export default initialize;