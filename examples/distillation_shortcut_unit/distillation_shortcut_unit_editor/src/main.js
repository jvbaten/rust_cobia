let statusTimeOutId = null;
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

window.addEventListener("DOMContentLoaded", () => {
    document.addEventListener('contextmenu', event => event.preventDefault());
    let promises=[
        listen('update-content', (event) => {
            update_content(event.payload);
        }),
        listen('update-streams', (event) => {
            update_streams(event.payload);
        }),
        listen('update-status', (event) => {
            update_status(event.payload);
        }),
    ];
    select_tab('configure');
    Promise.all(promises).then(() => {
        invoke("request_update");
    });
});


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

function update_content(content) {
    let listLightKey = document.getElementById('light_key_compound');
    let listHeavyKey = document.getElementById('heavy_key_compound');
    for (var i = 0; i < content.compound_list.length; i++) {
        listLightKey.options[i] = new Option(content.compound_list[i]);
        listHeavyKey.options[i] = new Option(content.compound_list[i]);
    }
    listLightKey.value = content.light_key_compound;
    listHeavyKey.value = content.heavy_key_compound;
    document.getElementById('unit_name').value = content.unit_name;
    document.getElementById('unit_description').value = content.unit_description;
    document.getElementById('light_key_compound_recovery').value = content.light_key_compound_recovery;
    document.getElementById('heavy_key_compound_recovery').value = content.heavy_key_compound_recovery;
    document.getElementById('reflux_ratio_factor').value = content.reflux_ratio_factor;
    document.getElementById('maximum_iterations').value = content.maximum_iterations;
    document.getElementById('convergence_tolerance').value = content.convergence_tolerance.toExponential();
    document.getElementById('number_of_stages').value = content.number_of_stages;
    document.getElementById('reflux_ratio').value = content.reflux_ratio;
    document.getElementById('feed_stage_location').value = content.feed_stage_location;
}

function update_streams(streams) {
    let tableContent = "<tr><td></td><td class=\"streamheader\">Feed</td><td class=\"streamheader\">Distillate</td><td class=\"streamheader\">Bottoms</td></tr>\n";
    for (const row of streams) {
        tableContent += `<tr><td class="streamheader">${row[0]}<td>${row[1]}</td><td>${row[2]}</td><td>${row[3]}</td></tr>\n`;
    }
    document.getElementById('porttable').innerHTML = tableContent;
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

function update_status(stat) {
    let status = document.getElementById('statusbar');
    status.innerHTML = stat.text;
    status.style.color = stat.error ? "red" : "green";
    status.style.backgroundColor = "#f1f1f1";
}

function data_entry(controlId) {
    invoke("data_entry", { controlId: controlId, value: document.getElementById(controlId).value }).then((err_msg) => {
        if ((err_msg !== undefined) && (err_msg !== null)) {
            temporary_message(response.error);
        }
    });
}
