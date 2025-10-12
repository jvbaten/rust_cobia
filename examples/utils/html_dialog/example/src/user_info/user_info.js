function select_tab(tabName) {
    let tabcontent = document.getElementsByClassName("tabcontent");
    for (let i = 0; i < tabcontent.length; i++) {
        tabcontent[i].style.display = "none";
    }
    let tablinks = document.getElementsByClassName("tablinks");
    for (let i = 0; i < tablinks.length; i++) {
        tablinks[i].className = tablinks[i].className.replace(" active", "");
    }
    let tab = document.getElementById(tabName);
    tab.style.display = "block";
    let selector = document.getElementById("select_" + tabName);
    selector.className += " active";
}

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
    select_tab('info');
    request_resize(500, 400);
    update_content();
}

function pretty_title(str) {
    //convert snake_case to Title Case
    let spaced = str.replace(/_/g, ' ');
    return spaced.charAt(0).toUpperCase() + spaced.substr(1);
}

function update_content() {
    window.fetch(window.location.origin + "/get_user_info").then((response) => {
        if (response.ok) {
            response.text().then((text) => {
                //fill table
                const obj = JSON.parse(text);
                content = "";
                for (var prop in obj) {
                    if (Object.prototype.hasOwnProperty.call(obj, prop)) {
                        content = content + "<tr><td class='user_content'>" + pretty_title(prop) + "</td><td class='user_content'>" + obj[prop] + "</td></tr>\n";
                    }
                }
                document.getElementById('user_info').innerHTML = content;
            });
        } else {
            //try again in 1 second
            window.setTimeout(update_content, 1000);
        }
    });

}

function onAgain() {
    //update content
    update_content();
}

function onPhoto() {
    //show sub dialog
    window.fetch(window.location.origin + "/show_user_photo");
}

function onClose() {
    //terminate
    request_terminate();
}