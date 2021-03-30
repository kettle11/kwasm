const decoder = new TextDecoder();

function receive_message(command, data) {

    if (command == 0) {
        console.log("COMMAND 0");
        let args = new Uint32Array(data.buffer, data.byteOffset, data.length / 4);
        let string_array = new Uint8Array(new Uint8Array(data.buffer, args[0], args[1]));

        // Preserve this because the stack will be changed after this function returns.
        let callback_data = args[2];
        console.log(args);
        let s = decoder.decode(string_array);
        console.log(s);

        // This could fail, but for now don't handle those cases.
        fetch(s)
            .then(async response => {
                let result = await response.arrayBuffer();
                console.log("RESULT LENGTH");

                console.log(result.length);

                let pointer = kwasm_exports.kwasm_reserve_space(result.byteLength);
                console.log("Pointer");
                console.log(pointer);

                let destination = new Uint8Array(kwasm_memory.buffer, pointer, result.byteLength);
                destination.set(new Uint8Array(result));
                kwasm_exports.kwasm_complete_fetch(callback_data);
            });
    }
    if (command == 1) {
        console.log("COMMAND 1");
    }
    console.log("RETURN FROM FETCH");
    return 0;
}

return receive_message;