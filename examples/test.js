const decoder = new TextDecoder();

let js_items = [null];
let free_indices = [];

function new_item(item) {
    let index = free_indices.pop();
    if (!index) {
        return js_items.push(item) - 1;
    } else {
        js_items[index] = item;
        return index;
    }
}

function free_item(index) {
    let item = js_items[index];
    js_items[index] = null;
    free_indices.push(index);
    return item;
}

function receive_message(command, data) {
    switch (command) {
        case 0: {
            // Register a string
            let args = new Uint32Array(data.buffer, data.byteOffset, data.length / 4);
            let string_array = new Uint8Array(new Uint8Array(data.buffer, args[0], args[1]));
            let result = decoder.decode(string_array);
            let item = new_item(result);
            return item;
        }
        case 1: {
            // Register a function
            let args = new Uint32Array(data.buffer, data.byteOffset, data.length / 4);
            let string_array = new Uint8Array(new Uint8Array(data.buffer, args[0], args[1]));
            let name = decoder.decode(string_array);
            let f = "function(data, f) {\n let offset = 0;\n let args = [";
            for (let i = 2; i < args.length; i++) {
                switch (args[i]) {
                    case 1:
                        f += "data.getUint32(offset),"
                        break
                    case 2:
                        f += "data.getFloat32(offset),"
                        break
                    case 3:
                        f += "data.getUint32(offset),"
                        break;
                }
            }
            f += "];\n f()}";
        }
        case 2: {
            // Call function
            let args = new Uint32Array(data.buffer, data.byteOffset, data.length / 4);
            js_items[args[0]](data);
        }
        /*
        case 1: {
            let args = new Uint32Array(data.buffer, data.byteOffset, data.length / 4);
            console.log(js_items[args[0]]);
        }
        */
    }
    return 0;
}

return receive_message;