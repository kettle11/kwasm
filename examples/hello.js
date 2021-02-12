function receive_message(command, data) {
    // Each of these commands could do something different.
    // The data is an ArrayBuffer that is a view into the raw WebAssembly bytes.
    if (command == 0) {
        console.log("COMMAND 0");
    }
    if (command == 1) {
        console.log("COMMAND 1");
    }
    return 0;
}

return receive_message;