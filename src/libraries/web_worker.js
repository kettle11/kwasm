function run_on_worker() {
    onmessage = function (e) {
        console.log('Running new worker thread');

        let imports = {};

        WebAssembly.Module.imports(e.data.kwasm_module).forEach(item => {
            if (imports[item.module] === undefined) {
                imports[item.module] = {};
            }

            if (item.kind == "function") {
                imports[item.module][item.name] = function () {
                    console.log("Unimplemented in web worker");
                }
            }

            if (item.kind == "memory") {
                console.log(item);
                imports[item.module][item.name] = {};
            }
        });
        imports.env.memory = e.data.kwasm_memory;

        this.wasm_memory = e.data.kwasm_memory;

        WebAssembly.instantiate(e.data.kwasm_module, imports).then(results => {
            this.wasm_exports = results.exports;
            this.wasm_exports.set_stack_pointer(e.data.stack_pointer);
            this.wasm_exports.__wasm_init_memory();
            this.wasm_exports.__wasm_init_tls(e.data.thread_local_storage_pointer);

            this.wasm_exports.kwasm_web_worker_entry_point(e.data.entry_point);
        });
    }
}

var create_worker_blob_url = URL.createObjectURL(new Blob(
    ['(', run_on_worker.toString(), ')()'],
    { type: 'application/javascript' }
));

function create_worker(entry_point, stack_pointer, thread_local_storage_pointer) {
    console.log(kwasm_module);
    let worker = new Worker(create_worker_blob_url);

    worker.postMessage({
        kwasm_memory: kwasm_memory,
        kwasm_module: kwasm_module,
        entry_point: entry_point,
        stack_pointer: stack_pointer,
        thread_local_storage_pointer: thread_local_storage_pointer
    });
}

function receive_message(command, data) {
    // Each of these commands could do something different.
    // The data is an ArrayBuffer that is a view into the raw WebAssembly bytes.
    switch (command) {
        case 0:
            // This assumes that the Wasm pointer size is 32bits, which may not hold forever.
            let entry_point_and_stack = new Uint32Array(data.buffer, data.byteOffset, data.length / 4);
            create_worker(entry_point_and_stack[0], entry_point_and_stack[1], entry_point_and_stack[2]);
            break;
        case 1:
            // Access the thread local storage size global variable created by LLVM.
            return kwasm_exports.__tls_size.value;
        case 2:
            // Access the thread local storage alignment global variable created by LLVM.
            return kwasm_exports.__tls_align.value;

    }

    return 0;
}

return receive_message;