use html_dialog::{HtmlDialogHandler,HtmlDialogResourceType,HtmlDialog,Window};
use cobia::*;
use cobia::prelude::*;
use super::distillation_shortcut_unit::DistillationShortcutUnit;

pub(crate) enum UnitDialogEvent {
    GetContent,
    GetStatus,
    DataEntry,
    Streams,
}

pub(crate) struct UnitDialogHandler<'a > {
    data : json::JsonValue,
	unit: &'a mut DistillationShortcutUnit,
    modified : bool
}

impl<'a> UnitDialogHandler<'a> {

    pub fn new(parent:CapeWindowId,unit: &'a mut DistillationShortcutUnit) -> HtmlDialog<UnitDialogHandler<'a>,UnitDialogEvent> {
        //create a compound list that contains the actual compounds, and if not present, also the selected compounds
        let mut comps=unit.get_compound_list();
        let light_key=unit.get_light_key_compound();
        if !light_key.is_empty() && !comps.contains(&light_key) {
            comps.push(light_key.clone());
        }
        let heavy_key=unit.get_heavy_key_compound();
        if !heavy_key.is_empty() && !comps.contains(&heavy_key) {
            comps.push(heavy_key.clone());
        }
        let data = json::object!{
            unit_name: unit.get_name(),
            compound_list: comps,
            unit_description: unit.get_description(),
            light_key_compound: light_key,
            heavy_key_compound: heavy_key,
            light_key_compound_recovery: unit.get_light_key_compound_recovery(),
            heavy_key_compound_recovery: unit.get_heavy_key_compound_recovery(),
            reflux_ratio_factor: unit.get_reflux_ratio_factor(),
            maximum_iterations: unit.get_maximum_iterations(),
            convergence_tolerance: unit.get_convergence_tolerance(),
            number_of_stages: unit.get_number_of_stages(),
            reflux_ratio: unit.get_reflux_ratio(),
            feed_stage_location: unit.get_feed_stage_location(),
        };
        let handler=UnitDialogHandler {
            data,
            unit,
            modified: false
        };
        let mut dlg=HtmlDialog::<UnitDialogHandler,UnitDialogEvent>::new(handler);
        dlg.add("/".into(),HtmlDialogResourceType::Content((include_bytes!("gui.html"),"text/html")));
        dlg.add("/gui.css".into(),HtmlDialogResourceType::Content((include_bytes!("gui.css"),"text/css")));
        dlg.add("/gui.js".into(),HtmlDialogResourceType::Content((include_bytes!("gui.js"),"text/javascript")));
        dlg.add("/gui.png".into(),HtmlDialogResourceType::Content((include_bytes!("gui.png"),"image/png")));
        dlg.add("/content".into(),HtmlDialogResourceType::Info(UnitDialogEvent::GetContent));
        dlg.add("/streams".into(),HtmlDialogResourceType::Info(UnitDialogEvent::Streams));
        dlg.add("/status".into(),HtmlDialogResourceType::Info(UnitDialogEvent::GetStatus));
        dlg.add("/data_entry".into(),HtmlDialogResourceType::Info(UnitDialogEvent::DataEntry));
        #[cfg(target_os = "windows")] let parent= Some((parent as *mut core::ffi::c_void).into());
        dlg.set_parent(parent,true);
        dlg
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn short_error(e: COBIAError) -> String {
        match e {
            COBIAError::Message(msg) => msg,
            COBIAError::MessageWithCause(msg, _cause) => msg,
            COBIAError::Code(_) => e.into(),
            COBIAError::CAPEOPEN(ref err) => {
                if let Ok(msg) = err.get_error_text() {
                    msg
                } else {
                    e.into()
                }
            }
        }        
    }
}

impl<'a> HtmlDialogHandler<UnitDialogEvent> for UnitDialogHandler<'a> {
    fn provide_content(&mut self, event: &UnitDialogEvent, content: Option<String>, _dialog_window : Option<Window>) -> Result<(Vec<u8>,String), Box<dyn std::error::Error>> {
        match event {
            UnitDialogEvent::GetContent => {
                Ok((json::stringify(self.data.clone()).into_bytes(),"application/json".to_string()))
            },
            UnitDialogEvent::GetStatus => {
                //current status
                let status=match self.unit.validate_internal() {
                    Ok(()) => {
                        json::object!{
                            text: "Specification is complete",
                            error: false,
                        }
                    },
                    Err(e) => {
                        json::object!{
                            text: Self::short_error(e),
                            error: true,
                        }
                    }
                };
                Ok((json::stringify(status).into_bytes(),"application/json".to_string()))
            },
            UnitDialogEvent::Streams => {
                //fill the streams table
                let ports=self.unit.get_ports();
                //port name and connected objects
                let mut streams=Vec::<Option<cape_open_1_2::CapeThermoMaterial>>::with_capacity(3);
                let mut port_info=Vec::<Vec::<String>>::new();
                for port in ports {
                    //get stream
                    if let Ok(stream)=port.get_connected_object() {
                        match cape_open_1_2::CapeThermoMaterial::from_object(&stream) {
                            Ok(s) => {streams.push(Some(s));}
                            Err(_) => {streams.push(None);}
                        }
                    } else {
                        streams.push(None);
                    }
                }
                //stream connections
                let mut stream_names=Vec::<String>::with_capacity(4);
                let mut any=false;
                stream_names.push("Stream".into());
                for stream in &streams {
                    if let Some(stream) = stream {
                        //get stream name
                        any=true;
                        let mut name="<Unnamed>".to_string();
                        if let Ok(iden)=cape_open_1_2::CapeIdentification::from_object(stream) {
                            let mut stream_name=cobia::CapeStringImpl::new();
                            if iden.get_component_name(&mut stream_name).is_ok() {
                                name=stream_name.to_string();
                            }
                        }
                        stream_names.push(name);
                    } else {
                        stream_names.push("<Not connected>".into());
                    }
                }
                port_info.push(stream_names);
                if any {
                    let mut values=cobia::CapeArrayRealVec::new();
                    let str_no_basis=cobia::CapeStringImpl::new();
                    let str_mass_basis=cobia::CapeStringImpl::from("mass");
                    //stream temperature
                    let mut stream_temperature=Vec::<String>::with_capacity(4);
                    stream_temperature.push("Temperature / [°C]".into());
                    let str_temperature=cobia::CapeStringImpl::from("temperature");
                    for stream in &streams {
                        if let Some(stream) = stream {
                            match stream.get_overall_prop(&str_temperature,&str_no_basis,&mut values) {
                                Ok(()) => {
                                    if values.size()==1 {
                                        stream_temperature.push(format!("{:.2}", values[0]-273.15));
                                    } else {
                                        stream_temperature.push("N/A".into());
                                    }
                                },
                                Err(_) => {
                                    stream_temperature.push("N/A".into());
                                }
                            }
                        } else {
                            stream_temperature.push(String::new());
                        }
                    }
                    port_info.push(stream_temperature);
                    //stream pressure
                    let mut stream_pressure=Vec::<String>::with_capacity(4);
                    stream_pressure.push("Pressure / [bar]".into());
                    let str_pressure=cobia::CapeStringImpl::from("pressure");
                    for stream in &streams {
                        if let Some(stream) = stream {
                            match stream.get_overall_prop(&str_pressure,&str_no_basis,&mut values) {
                                Ok(()) => {
                                    if values.size()==1 {
                                        stream_pressure.push(format!("{:.3}", values[0]*1e-5));
                                    } else {
                                        stream_pressure.push("N/A".into());
                                    }
                                },
                                Err(_) => {
                                    stream_pressure.push("N/A".into());
                                }
                            }
                        } else {
                            stream_pressure.push(String::new());
                        }
                    }
                    port_info.push(stream_pressure);
                    //component flows and recoveries
                    let comp_list=self.unit.get_compound_list();
                    let str_flow=cobia::CapeStringImpl::from("flow");
                    let mut feed_rates=cobia::CapeArrayRealVec::new();
                    if let Some(ref feed) = streams[0] {
                        let _=feed.get_overall_prop(&str_flow,&str_mass_basis,&mut feed_rates);
                    }
                    if feed_rates.size()==comp_list.len() {
                        let mut distillate_rates=cobia::CapeArrayRealVec::new();
                        let mut bottoms_rates=cobia::CapeArrayRealVec::new();
                        if let Some(ref distillate) = streams[1] {
                            let _=distillate.get_overall_prop(&str_flow,&str_mass_basis,&mut distillate_rates);
                        }
                        if let Some(ref bottoms) = streams[2] {
                            let _=bottoms.get_overall_prop(&str_flow,&str_mass_basis,&mut bottoms_rates);
                        }
                        //iterate over compounds by index
                        comp_list.iter().enumerate().for_each(|(i,comp)| {
                            let mut stream_comp=Vec::<String>::with_capacity(4);
                            stream_comp.push("F[".to_string()+comp+"] / [kg/s]");
                            stream_comp.push(format!("{:.3}", feed_rates[i]));
                            if distillate_rates.size()==comp_list.len() {
                                if distillate_rates[i]<=0.0 {
                                    stream_comp.push("0".into());
                                } else {
                                    stream_comp.push(format!("{:.3} ({:.2}%)", distillate_rates[i],100.0*distillate_rates[i]/feed_rates[i]));
                                }
                            } else {
                                stream_comp.push(String::new());
                            }
                            if bottoms_rates.size()==comp_list.len() {
                                if bottoms_rates[i]<=0.0 {
                                    stream_comp.push("0".into());
                                } else {
                                    stream_comp.push(format!("{:.3} ({:.2}%)", bottoms_rates[i],100.0*bottoms_rates[i]/feed_rates[i]));
                                }
                            } else {
                                stream_comp.push(String::new());
                            }
                            port_info.push(stream_comp);
                        });
                    }
                }
                let port_info=json::object!{
                    table:port_info
                };
                Ok((json::stringify(port_info).into_bytes(),"application/json".to_string()))
            },
            UnitDialogEvent::DataEntry => {
                let mut error_text=String::new();
                if let Some(content) = content {
                    match json::parse(&content) {
                        Ok(data) => {
                            let value=data["value"].as_str().unwrap_or("");
                            let control_id=data["controlId"].as_str().unwrap_or("");
                            match control_id {
                                "unit_name" => {
                                    let new_name:String=value.into();
                                    if new_name.is_empty() {
                                        error_text="Unit name cannot be empty".into();
                                    } else {
                                        if new_name!=self.data["unit_name"] {
                                            self.unit.set_name(&new_name);
                                            self.data["unit_name"]=new_name.into();
                                            self.modified=true;
                                        }
                                    }
                                },
                                "unit_description" => {
                                    let new_desc:String=value.into();
                                    if new_desc!=self.data["unit_description"] {
                                        self.unit.set_description(&new_desc);
                                        self.data["unit_description"]=new_desc.into();
                                        self.modified=true;
                                    }
                                },
                                "light_key_compound" => {
                                    let new_comp:String=value.into();
                                    if new_comp!=self.data["light_key_compound"] {
                                        if !self.unit.get_compound_list().contains(&new_comp) {
                                            error_text=format!("Compound {} is not defined", new_comp);
                                        } else {
                                            match self.unit.set_light_key_compound(&new_comp) {
                                                Ok(()) => {
                                                    self.data["light_key_compound"]=new_comp.into();
                                                    self.modified=true;
                                                },
                                                Err(e) => {
                                                    error_text=Self::short_error(e);
                                                }
                                            }
                                        }
                                    }
                                },
                                 "heavy_key_compound" => {
                                    let new_comp:String=value.into();
                                    if new_comp!=self.data["heavy_key_compound"] {
                                        if !self.unit.get_compound_list().contains(&new_comp) {
                                            error_text=format!("Compound {} is not defined", new_comp);
                                        } else {
                                            match self.unit.set_heavy_key_compound(&new_comp) {
                                                Ok(()) => {
                                                    self.data["heavy_key_compound"]=new_comp.into();
                                                    self.modified=true;
                                                },
                                                Err(e) => {
                                                    error_text=Self::short_error(e);
                                                }
                                            }
                                        }
                                    }
                                },
                                "maximum_iterations" => {
                                    match value.parse::<i32>() {
                                        Ok(value) => {
                                            if value!=self.unit.get_maximum_iterations() {
                                                match self.unit.set_maximum_iterations(value) {
                                                    Ok(()) => {
                                                        self.data["maximum_iterations"]=value.into();
                                                        self.modified=true;
                                                    },
                                                    Err(e) => {
                                                        error_text=Self::short_error(e);
                                                    }
                                                };
                                            }
                                        },
                                        Err(_) => {
                                            error_text=format!("Invalid integer value: {}", value);
                                        }
                                    };
                                },
                                _ => {
                                    //these are all real values
                                    match value.parse::<f64>() {
                                        Ok(value) => {
                                            match control_id {
                                                "light_key_compound_recovery" => {
                                                    if value!=self.unit.get_light_key_compound_recovery() {
                                                        match self.unit.set_light_key_compound_recovery(value) {
                                                            Ok(()) => {
                                                                    self.data["light_key_compound_recovery"]=value.into();
                                                                    self.modified=true;
                                                                },
                                                            Err(e) => {
                                                                error_text=Self::short_error(e);
                                                            }
                                                        }

                                                    }
                                                },
                                                "heavy_key_compound_recovery" => {
                                                    if value!=self.unit.get_heavy_key_compound_recovery() {
                                                        match self.unit.set_heavy_key_compound_recovery(value) {
                                                            Ok(()) => {
                                                                    self.data["heavy_key_compound_recovery"]=value.into();
                                                                    self.modified=true;
                                                                },
                                                            Err(e) => {
                                                                error_text=Self::short_error(e);
                                                            }
                                                        }
                                                    }
                                                },
                                                "reflux_ratio_factor" => {
                                                    if value!=self.unit.get_reflux_ratio_factor() {
                                                        match self.unit.set_reflux_ratio_factor(value) {
                                                            Ok(()) => {
                                                                    self.data["reflux_ratio_factor"]=value.into();
                                                                    self.modified=true;
                                                                },
                                                            Err(e) => {
                                                                error_text=Self::short_error(e);
                                                            }
                                                        }
                                                    }
                                                },
                                                "convergence_tolerance" => {
                                                    if value!=self.unit.get_convergence_tolerance() {
                                                        match self.unit.set_convergence_tolerance(value) {
                                                            Ok(()) => {
                                                                    self.data["convergence_tolerance"]=value.into();
                                                                    self.modified=true;
                                                                },
                                                            Err(e) => {
                                                                error_text=Self::short_error(e);
                                                            }
                                                        }
                                                    }
                                                },
                                                _ => {
                                                    error_text=format!("Unknown control ID: {}", control_id);
                                                }
                                            }
                                        },
                                        Err(_) => {
                                            error_text=format!("Invalid numeric value: {}", value);
                                        }
                                    };
                                }
                            }
                        },
                        Err(e) => {
                            error_text=format!("Error parsing data: {}", e);
                        }
                    }
                } else {
                    error_text="Missing POST data".into();
                }
                let is_error=!error_text.is_empty();
                let response_json=json::object!{
                    error_text: error_text,
                    error: is_error,
                };
                Ok((json::stringify(response_json).into_bytes(),"application/json".to_string()))
            },
        }
    }
}

