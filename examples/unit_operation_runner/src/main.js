const { message } = window.__TAURI__.dialog;
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

window.addEventListener("DOMContentLoaded", () => {
    document.querySelector("#streams_button").addEventListener("click", select_streams);
    document.querySelector("#parameters_button").addEventListener("click", select_parameters);
    document.querySelector("#reports_button").addEventListener("click", select_reports);
    document.querySelector("#log_button").addEventListener("click", select_log);
    document.querySelector("#info_button").addEventListener("click", select_info);
    window.onresize=() => {
        update_current_report(true);
    };
    document.addEventListener('contextmenu', event => event.preventDefault());
    document.querySelector("#reportselect").addEventListener("change", () => {
        update_current_report(false);
    });
    let promises=[
        listen('update-thermo', (event) => {
            update_thermo_status(event.payload);
        }),
        listen('update-unit', (event) => {
            update_unit_status(event.payload);
        }),
        listen('update-streams', (event) => {
            update_stream_status(event.payload);
        }),
        listen('update-parameters', (event) => {
            update_parameter_status(event.payload);
        }),
        listen('update-reports', (event) => {
            update_reports(event.payload);
        }),
        listen('update-info', (event) => {
            update_info_status(event.payload);
        }),
        listen('update-log', (event) => {
            update_log_status(event.payload);
        }),
        listen('update-status', (event) => {
            update_status_bar(event.payload);
        })
    ];
    select_streams();
    Promise.all(promises).then(() => {
        invoke("request_update");
    });
});

async function clear_thermo() {
    invoke("clear_thermo");
}

async function edit_thermo() {
    invoke("edit_thermo").then((err_msg) => {
        if ((err_msg !== undefined) && (err_msg !== null)) {
            message(err_msg, { title: 'Failed to edit:', kind: 'error' });
        }
    });
}

async function select_thermo_provider() {
    let select_el = document.querySelector("#provider_select");
    let provider_uuid = select_el.value;
    if (provider_uuid !== "") {
        invoke("select_thermo_provider", { provider: provider_uuid });
    }
}

async function select_thermo_package() {
    let select_el = document.querySelector("#package_select");
    let str_package = select_el.value;
    if (str_package !== "") {
        invoke("select_thermo_package", { package: str_package });
    }
}

function update_thermo_status(thermo_status) {
    let thermo_status_div = document.querySelector("#thermostatus");
    if (thermo_status.error !== null) {
        thermo_status_div.innerHTML = "Error: " + escapeHTML(thermo_status.error) + " <button class=\"cancel button\" id=\"clearthermo\" title=\"Reset\"></button>";
        document.querySelector("#clearthermo").addEventListener("click", clear_thermo);
    } else if (thermo_status.active_package !== null) {
        thermo_status_div.innerHTML = escapeHTML(thermo_status.active_package) + " <button class=\"edit button\" id=\"editthermo\" title=\"Edit\"></button><button class=\"cancel button\" id=\"clearthermo\" title=\"Reset\"></button>";
        document.querySelector("#clearthermo").addEventListener("click", clear_thermo);
        document.querySelector("#editthermo").addEventListener("click", edit_thermo);
    } else if (thermo_status.provider !== null) {
        let str = "Select package from " + escapeHTML(thermo_status.provider) + " <select id=\"package_select\"><option value=\"\"></option>";
        for (let pkg of thermo_status.package_list) {
            str += "<option value=\"" + escapeHTML(pkg) + "\">" + escapeHTML(pkg) + "</option>";
        }
        str += "</select><button class=\"cancel button\" id=\"clearthermo\" title=\"Reset\"></button>";
        thermo_status_div.innerHTML = str;
        document.querySelector("#package_select").addEventListener("change", select_thermo_package);
        document.querySelector("#clearthermo").addEventListener("click", clear_thermo);
    } else {
        let str = "Select thermodynamic provider: <select id=\"provider_select\"><option value=\"\"></option>";
        for (let i = 0; i < thermo_status.provider_list.length; i++) {
            let provider = thermo_status.provider_list[i];
            let provider_uuid = thermo_status.provider_uuid_list[i];
            str += "<option value=\"" + escapeHTML(provider_uuid) + "\">" + escapeHTML(provider) + "</option>";
        }
        str += "</select>";
        thermo_status_div.innerHTML = str;
        document.querySelector("#provider_select").addEventListener("change", select_thermo_provider);
    }
}

async function update_thermo() {
    let thermo_status = await invoke("get_thermo_status");
    update_thermo_status(thermo_status);
}

async function clear_unit() {
    invoke("clear_unit");
}

async function edit_unit() {
    invoke("edit_unit").then((err_msg) => {
        if ((err_msg !== undefined) && (err_msg !== null)) {
            message(err_msg, { title: 'Failed to edit:', kind: 'error' });
        }
    });
}

async function select_unit() {
    let select_el = document.querySelector("#unit_select");
    let provider_uuid = select_el.value;
    if (provider_uuid !== "") {
        invoke("select_unit", { unitSelection: provider_uuid });
    }
}

function update_unit_status(unit_status) {
    let unit_status_div = document.querySelector("#unitstatus");
    if (unit_status.error !== null) {
        unit_status_div.innerHTML = "Error: " + escapeHTML(unit_status.error) + " <button class=\"cancel button\" id=\"clearunit\" title=\"Reset\"></button>";
        document.querySelector("#clearunit").addEventListener("click", clear_unit);
    } else if (unit_status.active_unit !== null) {
        unit_status_div.innerHTML = escapeHTML(unit_status.active_unit) + " <button class=\"edit button\" id=\"editunit\" title=\"Edit\"></button><button class=\"cancel button\" id=\"clearunit\" title=\"Reset\"></button>";
        document.querySelector("#clearunit").addEventListener("click", clear_unit);
        document.querySelector("#editunit").addEventListener("click", edit_unit);
    } else {
        let str = "Select unit operation: <select id=\"unit_select\"><option value=\"\"></option>";
        for (let i = 0; i < unit_status.provider_list.length; i++) {
            let provider = unit_status.provider_list[i];
            let provider_uuid = unit_status.provider_uuid_list[i];
            str += "<option value=\"" + escapeHTML(provider_uuid) + "\">" + escapeHTML(provider) + "</option>";
        }
        str += "</select>";
        unit_status_div.innerHTML = str;
        document.querySelector("#unit_select").addEventListener("change", select_unit);
    }
}

function select_page(pageName) {
    //should not be called directly - call the specific select function for each page, which will also update the content of the page
    let tabcontent = document.getElementsByClassName("maincontent");
    for (let i = 0; i < tabcontent.length; i++) {
        tabcontent[i].style.display = "none";
    }
    let tablinks = document.getElementsByClassName("mainlink");
    for (let i = 0; i < tablinks.length; i++) {
        tablinks[i].className = tablinks[i].className.replace(" mainselection", "");
    }
    let page = document.getElementById(pageName);
    page.style.display = "block";
    let selector = document.getElementById(pageName + "_button");
    selector.className += " mainselection";
}

function select_streams() {
    select_page("streams");
}

function select_reports() {
    select_page("reports");
}

function select_log() {
    select_page("log");
}

function select_parameters() {
    select_page("parameters");
}

function select_info() {
    select_page("info");
}

function escapeHTML(str) {
    return str.replace(/[&<>'"]/g, 
        tag => ({
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            "'": '&#39;',
            '"': '&quot;'
            }[tag]));
}

function update_status_bar(status) {
    let status_div = document.querySelector("#statusbar");
    switch (status.status) {
        case "Ready":
            status_div.innerHTML = "Ready.";
            status_div.style.color = "orange";
            break;
        case "Solved":
            status_div.innerHTML = "Solved.";
            status_div.style.color = "darkgreen";
            break;
        case "Error":
            status_div.innerHTML = "Error: " + status.message;
            status_div.style.color = "darkred";
            break;
        case "Incomplete":
            status_div.innerHTML = "Incomplete: " + status.message;
            status_div.style.color = "darkorange";
            break;
        default:
            status_div.innerHTML = "Unknown status: " + status.status;
            status_div.style.color = "black";
            break;
    }
    if (status.status==="Ready") {
        //assume automatic calculation
        status_div.innerHTML = "Solving...";
        status_div.style.color = "black";
        invoke("calculate_unit");
    }
}

function update_table_status(table_data,table_id) {
    let table = document.querySelector(table_id);
    //put a thead in with one row
    let thead = table.querySelector("thead");
    if (!thead) {
        thead = document.createElement("thead");
        table.appendChild(thead);
    }
    let header_row = thead.querySelector("tr");
    if (!header_row) {
        header_row = document.createElement("tr");
        header_row.classList.add("header-row");
        thead.appendChild(header_row);
    }
    let existing_headers = Array.from(header_row.querySelectorAll("th"));
    let same = existing_headers.length === table_data.headers.length && existing_headers.every((h, i) => h.textContent == table_data.headers[i]);
    let column_ops = []; 
    if (!same) {
        //find remove and insert operations
        let current_header_index = 0;
        let new_header_index = 0;
        while (current_header_index < existing_headers.length) {
            if (new_header_index == table_data.headers.length) {
                //all remaining headers are removed
                column_ops.push(["remove", current_header_index]);
                existing_headers[current_header_index].remove();
                existing_headers.splice(current_header_index, 1);
                continue;
            }
            if (existing_headers[current_header_index].textContent == table_data.headers[new_header_index]) {
                current_header_index++;
                new_header_index++;
                continue;
            }
            let exists = false;
            for (let i = new_header_index + 1; i < table_data.headers.length; i++) {
                if (table_data.headers[i] == existing_headers[current_header_index].textContent) {
                    exists = true;
                    break;
                }
            }
            if (exists) {
                //insert next new_header_index before current_header_index
                column_ops.push(["insert", new_header_index, current_header_index]);
                let th = document.createElement("th");
                th.textContent = table_data.headers[new_header_index];
                if (current_header_index < existing_headers.length) {
                    header_row.insertBefore(th, existing_headers[current_header_index]);
                } else {
                    header_row.appendChild(th);
                }
                existing_headers.splice(current_header_index, 0, th);
            } else {
                //does not exist, remove
                column_ops.push(["remove", current_header_index]);
                existing_headers[current_header_index].remove();
                existing_headers.splice(current_header_index, 1);
            }
        }
        //all remaining new headers are inserted
        for (let i = new_header_index; i < table_data.headers.length; i++) {
            column_ops.push(["insert", i, existing_headers.length]);
            let th = document.createElement("th");
            th.textContent = table_data.headers[i];
            header_row.appendChild(th);
            existing_headers.push(th);
        }
    }
    //create body
    let existing_rows = Array.from(table.querySelectorAll("tr"));
    let existing_row_map = new Map();
    for (let tr of existing_rows) {
        let row_id = tr.getAttribute("data-id");
        if (row_id) {
            existing_row_map.set(row_id, tr);
        }
    }
    let level_map = new Map();
    let new_rows = [];
    let row_index = 1; //account for the 1 row in the thead
    for (let row of table_data.rows) {
        let tr = existing_row_map.get(row.id);
        if (!tr) {
            tr = document.createElement("tr");
            tr.setAttribute("data-id", row.id);
            tr.setAttribute("data-expanded", "true");
        }
        tr.setAttribute("data-parent", row.parent_id);
        if (tr !== table.rows[row_index]) {
            if (row_index < table.rows.length) {
                table.insertBefore(tr, table.rows[row_index]); 
            } else {
                table.appendChild(tr);
            }
        }
        row_index = row_index + 1;
        //find indent level
        let level = (level_map.get(row.parent_id) || 0) + 1;
        level_map.set(row.id, level);
        //create or update
        if (row.is_header) {
            tr.className = "section-header row-level-" + level.toString();
            var cell;
            if (tr.cells.length === 1) {
                cell = tr.cells[0];
            } else {
                cell = tr.insertCell(-1);
            }
            cell.colSpan = table_data.headers.length;
            cell.className = "section-header-cell level-" + level.toString();
            let expanded = tr.getAttribute("data-expanded") === "true";
            cell.innerHTML = "<div class=\"row-toggle\">"+(expanded ?  "&#9660;":"&#9658;")+"</div>"+((row.cells!==null &&  row.cells.length > 0) ? row.cells[0].value : row.id);
            cell.addEventListener("click", (e) => toggle_expand(e, table_id));
        } else {
            //perform insert/delete operations on row
            tr.className = "row-level-" + level.toString();
            if (tr.cells.length == 0) {
                //create all cells
                for (let i = 0; i < row.cells.length; i++) {
                    tr.insertCell(-1);
                }
            } else {
                //update cells
                for (let op of column_ops) {
                    if (op[0] === "remove") {
                        let index = op[1];
                        if (index < tr.children.length) {
                            tr.removeChild(tr.children[index]);
                        }
                    } else if (op[0] === "insert") {
                        tr.insertCell(op[2]);
                    }
                }
            }
            tr.cells[0].className = "title-cell level-" + level.toString();
            for (let i = 0; i < row.cells.length; i++) {
                let id_count=0;
                let td = tr.cells[i];
                let cell = row.cells[i];
                if (i>0) {
                    if (i==row.cells.length-1) {
                    td.className = "uom-cell";
                    } else if (cell.edit_id !== null) { 
                        td.className = "data-cell data-field";
                    } else {
                        td.className = "data-cell";
                    }
                }
                let input = td.querySelector("input");
                let select = td.querySelector("select");
                let datalist=  td.querySelector("datalist");
                if (cell.edit_id !== null) {
                    let is_select=(cell.options !== null)&&(cell.exclusive === true);
                    if (is_select) {
                        //select
                        if (input) {
                            input.remove();
                        }
                        if (datalist) {
                            datalist.remove();
                        }
                        if (!select) {
                            select = document.createElement("select");
                            td.innerHTML = "";
                            td.appendChild(select);
                        }
                        select.className = "cell-select";
                        select.removeEventListener("change", handle_cell_edit);
                        select.addEventListener("change", handle_cell_edit);
                        select.setAttribute("data-edit-id", cell.edit_id);
                        let has_current_value=false;
                        let inner_html="";
                        let current_value=cell.value;
                        for (let option of cell.options) {
                            if (option === current_value) {
                                has_current_value = true;
                            }
                            inner_html+="<option value=\""+option+"\">"+option+"</option>";
                        }
                        if (current_value!=="" && !has_current_value) {
                            let html_value=escapeHTML(current_value);
                            inner_html+="<option value=\""+html_value+"\" selected>"+html_value+"</option>";
                        }
                        select.innerHTML=inner_html;
                        if (current_value!=="") {
                            select.value=current_value;
                        }
                    } else {
                        if (!input) {
                            input = document.createElement("input");
                            input.type = "text";
                            td.innerHTML = "";
                            td.appendChild(input);
                        }
                        input.removeEventListener("change", handle_cell_edit);
                        input.removeEventListener("blur", handle_cell_edit);
                        input.addEventListener("change", handle_cell_edit);
                        input.addEventListener("blur", handle_cell_edit);
                        input.className = "cell-input";
                        input.setAttribute("data-edit-id", cell.edit_id);
                        input.value = cell.value;
                        if (cell.options !== null) {
                            input.removeAttribute("list");
                            if (!datalist) {
                                datalist = document.createElement("datalist");
                                td.appendChild(datalist);
                            }
                            let id="options-"+row.id+"-"+id_count;
                            id_count++;
                            datalist.id=id;
                            input.setAttribute("list", id);
                            let inner_html="";
                            for (let option of cell.options) {
                                inner_html+="<option value=\""+escapeHTML(option)+"\"></option>";
                            }
                            datalist.innerHTML=inner_html;
                        } else {
                            input.removeAttribute("list");
                            if (datalist) {
                                datalist.remove();
                            }
                        }
                    }
                } else {
                    if (input) {
                        input.remove();
                    }
                    if (select) {
                        select.remove();
                    }
                    if (datalist) {
                        datalist.remove();
                    }
                    td.innerHTML = cell.value;
                }
            }
        }
        existing_row_map.delete(row.id);
    }
    while (row_index < table.rows.length) {
        table.deleteRow(row_index);
    }
    adjust_row_visibility(table_id);
}

function toggle_expand(e,table_id) {
    let sender = (e.target || e.srcElement);
    if (sender.classList.contains("row-toggle")) {
        let row = sender.parentElement.parentElement;
        let expanded = row.getAttribute("data-expanded")!== "true";
        sender.innerHTML = expanded ?  "&#9660;":"&#9658;";
        row.setAttribute("data-expanded", expanded.toString());
        adjust_row_visibility(table_id);
    }
}

function adjust_row_visibility(table_id) {
    let table = document.querySelector(table_id);
    let visibility_map = new Map();
    visibility_map.set("null", true); 
    for (let row of table.rows) {
        //check if header
        if (row.classList.contains("header-row")) {
            continue;
        }
        if (row.classList.contains("section-header")) {
            //add visibility to map
            let parent_id = row.getAttribute("data-parent");
            let expanded = row.getAttribute("data-expanded")=== "true";
            let parent_expanded = visibility_map.get(parent_id);
            row.style.display = parent_expanded ? "" : "none";
            visibility_map.set(row.getAttribute("data-id"), parent_expanded&&expanded);
        } else {
            let parent_id = row.getAttribute("data-parent");
            let visible = visibility_map.get(parent_id);
            row.style.display = visible ? "" : "none";
        }   
    }
}

function handle_cell_edit(e) {
    let sender = (e.target || e.srcElement);
    let edit_id = sender.getAttribute("data-edit-id");
    let value = sender.value;
    invoke("specify", { editId: edit_id, value: value });
}

function update_stream_status(table_data) {
    update_table_status(table_data,"#streamtable");
}

function update_parameter_status(table_data) {
    update_table_status(table_data,"#parametertable");
}

function update_reports(report_list) {
    let select = document.querySelector("#reportselect");
    let current_value = select.value;
    let parts = current_value.split("\t");
    let report_name = parts[1];
    let select_index=-1;
    let index=0;
    addloop: for (let report of report_list) {
        while (index < select.options.length) {
            if (select.options[index].value === report) {
                if (report === current_value) {
                    select_index=index;
                }
                index++;
                continue addloop;
            }
            select.options.remove(index);
        }
        let o=document.createElement("option");
        let parts = report.split("\t");
        o.value=report;
        o.text=parts[1];
        if (o.value === current_value) {
            select_index=index;
        }
        select.options.add(o,index);
        index++;
    }
    if (select_index<0 && report_list.length > 0) {
        select_index=0;
    }
    select.selectedIndex=select_index;
    update_current_report(false);
}

function update_report_from(mime_type,content) {
    let show_text=false;
    let show_table=false;
    let show_iframe=false;
    let show_image=false;
    if (mime_type!="text/html" && mime_type.startsWith("text/")) { 
        let as_table = false;
        if (mime_type.startsWith("text/plain")) {
            //see if content has at least two tabs
            let index=content.indexOf("\t");
            if (index>=0) {
                let index2=content.indexOf("\t",index+1);
                if (index2>=0) {
                    as_table=true;
                }
            }
        };
        if (as_table) {
            //split content into lines
            let lines = content.split("\n");
            //split lines at tabs
            let max_parts=0;
            const lines_at_tabs = [];
            for (let line of lines) {
                if (line.endsWith("\r")) {
                    line=line.substring(0,line.length-1);
                }
                let splitted=line.split("\t");
                max_parts=Math.max(max_parts,splitted.length);
                lines_at_tabs.push(splitted);
            }
            let table=document.querySelector("#reporttable");
            table.innerHTML="";
            for (let line of lines_at_tabs) {
                let tr=document.createElement("tr");
                for (let i=0;i<line.length;i++) {
                    let td=document.createElement("td");
                    if (line[i]==="") {
                        td.innerHTML="&nbsp;";
                    } else {
                        td.textContent=line[i];
                    }
                    if (i==(line.length-1)) {
                        td.colSpan=max_parts-i;
                    }
                    tr.appendChild(td);
                }
                table.appendChild(tr);
            }
            show_table=true;
        } else {
            let report_div = document.querySelector("#reporttext");
            report_div.textContent = content;   
            show_text=true;
        }   
    } else if (mime_type.startsWith("image/")) { 
        let img=document.querySelector("#reportimage");
        img.src="data:"+mime_type+";base64,"+content;
        show_image=true;
    } else {
        //everything else in iframe, including text/html
        let iframe=document.querySelector("#reportiframe");
        iframe.srcdoc=content;
        show_iframe=true;
    }
    document.querySelector("#reporttext").style.display = show_text ? "block" : "none";
    document.querySelector("#reporttablediv").style.display = show_table ? "block" : "none";
    document.querySelector("#reportiframe").style.display = show_iframe ? "block" : "none";
    document.querySelector("#reportimage").style.display = show_image ? "block" : "none";
}

function update_current_report(image_only) {
    let select = document.querySelector("#reportselect");
    let current_value = select.value;
    let mime_type;
    let content;
    let is_image=current_value.startsWith("image/");
    if (image_only && !is_image) {
        return;
    }
    if (current_value === "") {
        update_report_from("text/plain","No report selected.");
    } else {
        let w=800;
        let h=600;
        if (is_image) {
            document.querySelector("#reporttext").style.display = "none";
            document.querySelector("#reporttablediv").style.display = "none";
            document.querySelector("#reportiframe").style.display = "none";
            let img=document.querySelector("#reportimage");
            document.querySelector("#reportimage").style.display = "block";
            w=img.offsetWidth;
            h=img.offsetHeight;
        }
        invoke("get_report_content", { report: current_value, imageWidth: w, imageHeight: h }).then(
            (content) => {
                //separate mime type and content at tabs
                let index=content.indexOf("\t");
                if (index<0) {
                    update_report_from("text/plain","Invalid report content.");
                } else {
                    update_report_from(content.substring(0,index),content.substring(index+1));
                }
            },
            (err) => {
                update_report_from("text/plain","Failed to get report content: " + err);
            }
        );
    }
}

function update_info_status(table_data) {
    update_table_status(table_data,"#infotable");
}

function update_log_status(table_data) {
    update_table_status(table_data,"#logtable");
}
