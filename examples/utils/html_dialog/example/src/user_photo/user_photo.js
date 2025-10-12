function request_resize(width, height) {
    //for stand-alone browser components, this will work
    window.resizeTo(width, height);
    //for embedded browser components, we need to send a message to the host application
    const obj = {};
    obj.width = width;
    obj.height = height;
    window.fetch(window.location.origin + "/resize_window", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(obj)
    });
}

function request_terminate() {
    //send a message to the host application
    window.fetch(window.location.origin + "/close_window");
}

function on_load() {
    request_resize(300, 450);
}

function onClose() {
    //terminate
    request_terminate();
}