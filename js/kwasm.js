function kwasm_stuff() {
    // This is used to decode strings passed from Wasm to Javascript.
    const decoder = new TextDecoder();
    const encoder = new TextEncoder();

    // The first library is used for null, the second is used by messages that create new libraries.
    var kwasm_libraries = [undefined, undefined];
    var available_threads = 0;
    var kwasm_js_objects = [null, self];
    var kwasm_js_objects_free_indices = [];

    let kwasm_helpers = {
        pass_string_to_client: function (string) {
            // Unfortunately this can't write directly to Wasm memory (yet).
            // See this issue: https://github.com/whatwg/encoding/issues/172
            const string_data = encoder.encode(string);
            let length = string_data.byteLength;
            let pointer = self.kwasm_exports.kwasm_reserve_space(length);
            const client_string = new Uint8Array(self.kwasm_memory.buffer, pointer, length);
            client_string.set(string_data);
        },
        new_js_object: function (item) {
            let index = kwasm_js_objects_free_indices.pop();
            if (!index) {
                return kwasm_js_objects.push(item) - 1;
            } else {
                kwasm_js_objects[index] = item;
                return index;
            }
        }
    };

    /*
    let call_js = function (object, f, args_count, arg0, arg1, arg2,) {
        // let args = [arg0, arg1, arg2, 
        let a0 = kwasm_js_objects[arg0];
        let a1 = kwasm_js_objects[arg1];
        let a2 = kwasm_js_objects[arg2];
        let f = kwasm_js_objects[object];
        let result = f(a0, a1, a2);
        if (result == undefined) {
            return 0;
        } else {
            return kwasm_helpers.new_js_object(result);
        }
    }
    */

    let kwasm_import_functions = {
        kwasm_free_js_object: function (index) {
            if (index > 1) {
                let item = kwasm_js_objects[index];
                kwasm_js_objects[index] = null;
                kwasm_js_objects_free_indices.push(index);
            }
        },
        kwasm_new_string: function (data, data_length) {
            const message_data = new Uint8Array(self.kwasm_memory.buffer, data, data_length);
            const decoded_string = decoder.decode(new Uint8Array(message_data));
            return kwasm_helpers.new_js_object(decoded_string);
        },
        kwasm_js_object_property: function (index, property) {
            let object = kwasm_js_objects[index];
            let property_name = kwasm_js_objects[property];
            let property_object = object[property_name];
            if (property_object == undefined) {
                console.log(object + " does not have property: " + property_name);
                return 0;
            } else {
                return kwasm_helpers.new_js_object(property_object);
            }
        },
        kwasm_call_js_object_2_args: function (object, arg0, arg1) {
            let a0 = kwasm_js_objects[arg0];
            let a1 = kwasm_js_objects[arg1];
            let f = kwasm_js_objects[object];
            let result = f(a0, a1);
            if (result == undefined) {
                return 0;
            } else {
                return kwasm_helpers.new_js_object(result);
            }
        },
        kwasm_call_js_object_3_args: function (object, arg0, arg1, arg2) {
            let a0 = kwasm_js_objects[arg0];
            let a1 = kwasm_js_objects[arg1];
            let a2 = kwasm_js_objects[arg2];
            let f = kwasm_js_objects[object];
            let result = f(a0, a1, a2);
            if (result == undefined) {
                return 0;
            } else {
                return kwasm_helpers.new_js_object(result);
            }
        },
        kwasm_call_js_object_1_args: call_js,
        kwasm_message_to_host: function (library, command, data, data_length) {
            // Creates a view into the memory
            const message_data = new Uint8Array(self.kwasm_memory.buffer, data, data_length);

            // Library 0 is reserved for null.
            // Library 1 is used for built-in kwasm commands.
            if (library == 1) {
                switch (command) {
                    case 0: {
                        // Create a new library
                        const decoded_string = decoder.decode(new Uint8Array(message_data));
                        let library = new Function("kwasm_exports", "kwasm_memory", "kwasm_module", "kwasm_helpers", decoded_string)
                            (self.kwasm_exports, self.kwasm_memory, self.kwasm_module, kwasm_helpers);
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
                        return self.kwasm_exports.__tls_size.value || 0;
                    case 4:
                        // Access the thread local storage alignment global variable created by LLVM.
                        return self.kwasm_exports.__tls_align.value || 0;
                    case 5:
                        return available_threads;
                    case 6:
                        // This assumes that the Wasm pointer size is 32bits, which may not hold forever.
                        let entry_point_and_stack = new Uint32Array(message_data.buffer, message_data.byteOffset, message_data.length / 4);
                        create_worker(entry_point_and_stack[0], entry_point_and_stack[1], entry_point_and_stack[2]);
                        break;
                    case 7:

                        break;
                    case 8:
                        // Access a JS item field and return it as a new JS item.
                        let item_and_property = new Uint32Array(message_data.buffer, message_data.byteOffset, message_data.length / 4);
                        return new_js_item(js_items[item_and_property[0]][js_items[item_and_property[1]]]);
                }

            } else {
                return (kwasm_libraries[library])(command, message_data);
            }
        }
    };


    // Load and setup the WebAssembly library.
    // This is called when using `kwasm` without wasm-bindgen.
    function initialize(wasm_library_path) {
        available_threads = navigator.hardwareConcurrency;

        if (!crossOriginIsolated) {
            console.error("kwasm: Not Cross Origin Isolated! Number of available threads set to 1. \n SharedArrayBuffers may be disabled by the browser as well.")
            available_threads = 1;
        }

        self.kwasm_memory = new WebAssembly.Memory({ initial: 32, maximum: 16384, shared: true });

        let imports = {
            env: {
                memory: self.kwasm_memory,
            }
        };

        imports.env = Object.assign(imports.env, kwasm_import_functions);

        fetch(wasm_library_path).then(response =>
            response.arrayBuffer()
        ).then(bytes =>
            WebAssembly.instantiate(bytes, imports)
        ).then(results => {
            // If this module exports memory use that instead.
            if (results.instance.exports.memory) {
                self.kwasm_memory = results.instance.exports.memory;
            }
            self.kwasm_exports = results.instance.exports;
            self.kwasm_module = results.module;

            // Setup thread-local storage for the main thread
            if (kwasm_exports.kwasm_alloc_thread_local_storage) {
                const thread_local_storage = kwasm_exports.kwasm_alloc_thread_local_storage();
                self.kwasm_exports.__wasm_init_tls(thread_local_storage);
            }

            // Call our start function.
            results.instance.exports.main();
        });
    }

    // If we're a worker thread we'll use this.
    onmessage = function (e) {
        let imports = {
            env: {}
        };
        let memory_assigned = false;

        // Fill in all wasm-bindgen functions with a placeholder.
        // This isn't great because it means that `wasm-bindgen` things
        // won't work in worker threads.
        WebAssembly.Module.imports(e.data.kwasm_module).forEach(item => {
            if (imports[item.module] === undefined) {
                imports[item.module] = {};
            }
            if (item.kind == "function") {
                imports[item.module][item.name] = function () {
                    console.log(item.name + "is unimplemented in worker thread.");
                }
            }
            if (item.kind == "memory") {
                imports[item.module][item.name] = e.data.kwasm_memory;
                memory_assigned = true;
            }

            // This is a bit hacky to make sure that kwasm bindings still work 
            // in a worker thread even though `wasm-bindgen` mangles the include name.
            if (item.name.includes("kwasmmessagetohost")) {
                imports[item.module][item.name] = kwasm_message_to_host;
            }
        });

        imports.env.kwasm_message_to_host = kwasm_message_to_host;

        if (!memory_assigned) {
            imports.env = {
                memory: e.data.kwasm_memory
            };
        }

        self.kwasm_memory = e.data.kwasm_memory;

        WebAssembly.instantiate(e.data.kwasm_module, imports).then(results => {
            self.kwasm_exports = results.exports;

            if (self.kwasm_exports.__wbindgen_start) {
                self.kwasm_exports.__wbindgen_start();
            } else {
                self.kwasm_exports.set_stack_pointer(e.data.stack_pointer);
                self.kwasm_exports.__wasm_init_tls(e.data.thread_local_storage_pointer);
            }

            self.kwasm_exports.kwasm_web_worker_entry_point(e.data.entry_point);
        });
    }

    function create_worker(entry_point, stack_pointer, thread_local_storage_pointer) {
        let worker = new Worker(kwasm_stuff_blob);
        worker.postMessage({
            kwasm_memory: self.kwasm_memory,
            kwasm_module: self.kwasm_module,
            entry_point: entry_point,
            stack_pointer: stack_pointer,
            thread_local_storage_pointer: thread_local_storage_pointer
        });
    }

    kwasm_import_functions.initialize = initialize;

    return kwasm_import_functions;
}

const kwasm = kwasm_stuff();
var kwasm_stuff_blob = URL.createObjectURL(new Blob(
    ['(', kwasm_stuff.toString(), ')()'],
    { type: 'application/javascript' }
));

export default kwasm.initialize;

// The rest of the code here is to accommodate wasm-bindgen binding.
const kwasm_message_to_host = kwasm.kwasm_message_to_host;
const kwasm_free_js_object = kwasm.kwasm_free_js_object;
const kwasm_js_object_property = kwasm.kwasm_js_object_property;
const kwasm_new_string = kwasm.kwasm_new_string;
const kwasm_call_js_object_1_args = kwasm.kwasm_call_js_object_1_args;
const kwasm_call_js_object_2_args = kwasm.kwasm_call_js_object_2_args;
const kwasm_call_js_object_3_args = kwasm.kwasm_call_js_object_3_args;
export {
    kwasm_message_to_host as kwasm_message_to_host,
    kwasm_free_js_object as kwasm_free_js_object,
    kwasm_js_object_property as kwasm_js_object_property,
    kwasm_new_string as kwasm_new_string,
    kwasm_call_js_object_1_args as kwasm_call_js_object_1_args,
    kwasm_call_js_object_2_args as kwasm_call_js_object_2_args,
    kwasm_call_js_object_3_args as kwasm_call_js_object_3_args,
};
export function kwasm_initialize_wasmbindgen(module, memory) {
    self.kwasm_module = module;
    self.kwasm_memory = memory;
    self.kwasm_exports = document.kwasm_exports;
}
