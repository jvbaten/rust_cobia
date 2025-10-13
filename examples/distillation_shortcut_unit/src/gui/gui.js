let statusTimeOutId = null;

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
    select_tab('configure');
    request_resize(600, 600);
    document.addEventListener('contextmenu', event => event.preventDefault());
    update_content();
    update_streams();
    update_status();
}

function update_content() {
    window.fetch(window.location.origin + "/content").then((response) => {
        if (response.ok) {
            response.text().then((text) => {
                const obj = JSON.parse(text);
                let listLightKey = document.getElementById('light_key_compound');
                let listHeavyKey = document.getElementById('heavy_key_compound');
                for (var i = 0; i < obj.compound_list.length; i++) {
                    listLightKey.options[i] = new Option(obj.compound_list[i]);
                    listHeavyKey.options[i] = new Option(obj.compound_list[i]);
                }
                listLightKey.value = obj.light_key_compound;
                listHeavyKey.value = obj.heavy_key_compound;
                document.getElementById('unit_name').value = obj.unit_name;
                document.getElementById('unit_description').value = obj.unit_description;
                document.getElementById('light_key_compound_recovery').value = obj.light_key_compound_recovery;
                document.getElementById('heavy_key_compound_recovery').value = obj.heavy_key_compound_recovery;
                document.getElementById('reflux_ratio_factor').value = obj.reflux_ratio_factor;
                document.getElementById('maximum_iterations').value = obj.maximum_iterations;
                document.getElementById('convergence_tolerance').value = obj.convergence_tolerance.toExponential();
                document.getElementById('number_of_stages').value = obj.number_of_stages;
                document.getElementById('reflux_ratio').value = obj.reflux_ratio;
                document.getElementById('feed_stage_location').value = obj.feed_stage_location;
            });
        } else {
            //try again in 1 second
            window.setTimeout(update_content, 1000);
        }
    });
}

function update_streams() {
    window.fetch(window.location.origin + "/streams").then((response) => {
        if (response.ok) {
            response.text().then((text) => {
                const obj = JSON.parse(text);
                let tableContent = "<tr><td></td><td class=\"streamheader\">Feed</td><td class=\"streamheader\">Distillate</td><td class=\"streamheader\">Bottoms</td></tr>\n";
                for (const row of obj.table) {
                    tableContent += `<tr><td class="streamheader">${row[0]}<td>${row[1]}</td><td>${row[2]}</td><td>${row[3]}</td></tr>\n`;
                }
                document.getElementById('porttable').innerHTML = tableContent;
            });
        } else {
            //try again in 1 second
            window.setTimeout(update_streams, 1000);
        }
    });
}

function temporary_message(message) {
    if (statusTimeOutId) {
        window.clearTimeout(statusTimeOutId);
        statusTimeOutId = null;
    }
    let status = document.getElementById('statusbar');
    status.innerHTML = message;
    status.style.color = "red";
    status.style.backgroundColor = "yellow";
    statusTimeOutId = window.setTimeout(update_status, 5000);
}

function update_status() {
    window.fetch(window.location.origin + "/status").then((response) => {
        if (response.ok) {
            response.text().then((text) => {
                const obj = JSON.parse(text);
                let status = document.getElementById('statusbar');
                status.innerHTML = obj.text;
                status.style.color = obj.error ? "red" : "green";
                status.style.backgroundColor = "#f1f1f1";
            });
            statusTimeOutId = null;
        } else {
            //try again in 1 second
            statusTimeOutId = window.setTimeout(update_status, 1000);
        }
    });
}

function data_entry(controlId) {
    const obj = {};
    obj.value = document.getElementById(controlId).value;
    obj.controlId = controlId;
    window.fetch(window.location.origin + "/data_entry", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(obj)
    }).then((response) => {
        if (response.ok) {
            // Parse the response as JSON
            response.json().then((response) => {
                if (response.error) {
                    temporary_message(response.error_text);
                }
            },
                (reason) => {
                    temporary_message("Error parsing response: " + reason);
                }
            );
        } else {
            temporary_message(response.error);
        }
        update_content();
        update_status();
    });
}
