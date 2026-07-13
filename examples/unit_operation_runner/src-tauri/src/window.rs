

pub(crate) trait Window {
    fn cape_window_id(&self) -> cobia::CapeWindowId;
}

pub(crate) struct WindowNone {
    cape_window_id: cobia::CapeWindowId,
}

impl WindowNone {
    pub(crate) fn new() -> Self {
        Self {
            cape_window_id: std::default::Default::default(),
        }
    }
}

impl Window for WindowNone {
    fn cape_window_id(&self) -> cobia::CapeWindowId {
        self.cape_window_id.clone()
    }
}

#[cfg(target_os = "windows")]
pub(crate) struct WindowWindows {
    cape_window_id: cobia::CapeWindowId,
}

#[cfg(target_os = "windows")]
impl WindowWindows {
    pub(crate) fn new(hwnd : cobia::C::HWND) -> Self {
        Self {
            cape_window_id: cobia::CapeWindowId::FromRaw(hwnd),
        }
    }

}

#[cfg(target_os = "windows")]
impl Window for WindowWindows {
    fn cape_window_id(&self) -> cobia::CapeWindowId {
        self.cape_window_id.clone() 
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) struct WindowX11 {
    cape_window_id: cobia::CapeWindowId,
}

#[cfg(not(target_os = "windows"))]
impl WindowX11 {
    pub(crate) fn new(display_str: String, window: u64) -> Self {
        Self {
            cape_window_id: cobia::CapeWindowId::x11(display_str, window),
        }
    }
}

#[cfg(not(target_os = "windows"))]  
impl Window for WindowX11 {
    fn cape_window_id(&self) -> cobia::CapeWindowId {
        self.cape_window_id.clone()
    }
}


#[cfg(not(target_os = "windows"))]
pub(crate) struct WaylandAppData {
    connection: Option<wayland_client::Connection>,
    foreign_v2_protocol_name: Option<(u32,u32)>,
    foreign_v1_protocol_name: Option<(u32,u32)>,
    interface_foreign_v2 : Option<wayland_protocols::xdg::foreign::zv2::client::zxdg_exporter_v2::ZxdgExporterV2>,
    interface_foreign_v1 : Option<wayland_protocols::xdg::foreign::zv1::client::zxdg_exporter_v1::ZxdgExporterV1>,
}

#[cfg(not(target_os = "windows"))]
impl WaylandAppData {

    pub(crate) fn new(display: *mut std::ffi::c_void) -> Self {
        let mut app_data=WaylandAppData {
            connection: None,
            foreign_v2_protocol_name: None,
            foreign_v1_protocol_name: None,
            interface_foreign_v2: None,
            interface_foreign_v1: None,
        };
        let back_end=unsafe { wayland_backend::sys::client::Backend::from_foreign_display(display as *mut wayland_sys::client::wl_display) };
        let connection=wayland_client::Connection::from_backend(back_end);
        app_data.connection = Some(connection.clone());
        let display=connection.display();
        let mut event_queue = connection.new_event_queue();
        let qhandle = event_queue.handle();
        let registry = display.get_registry(&qhandle,());
        event_queue.roundtrip(&mut app_data).unwrap();
        if let Some((v2_name, v2_version)) = &app_data.foreign_v2_protocol_name {
            app_data.interface_foreign_v2=Some(registry.bind::<wayland_protocols::xdg::foreign::zv2::client::zxdg_exporter_v2::ZxdgExporterV2,_,_>(*v2_name, *v2_version,&qhandle,()));
        } else if let Some((v1_name, v1_version)) = &app_data.foreign_v1_protocol_name {
            app_data.interface_foreign_v1=Some(registry.bind::<wayland_protocols::xdg::foreign::zv1::client::zxdg_exporter_v1::ZxdgExporterV1,_,_>(*v1_name, *v1_version,&qhandle,()));
        }
        app_data
    }

    pub(crate) fn get_window(&self,surface: *mut std::ffi::c_void) -> Box<dyn Window> {
        use wayland_client::Proxy;
        if let Ok(id) = unsafe {wayland_backend::sys::client::ObjectId::from_ptr(wayland_client::protocol::wl_surface::WlSurface::interface(),surface as *mut wayland_sys::client::wl_proxy) } {
            if let Some(interface_v2)=&self.interface_foreign_v2 {
                if let Ok(surface_proxy) = wayland_client::protocol::wl_surface::WlSurface::from_id(self.connection.as_ref().unwrap(), id) {
                    //export the handle
                    let mut export_id = WindowWaylandExportID::new();
                    let mut event_queue = self.connection.as_ref().unwrap().new_event_queue();
                    let qhandle = event_queue.handle();
                    let exported=interface_v2.export_toplevel(&surface_proxy, &qhandle, ());
                    event_queue.roundtrip(&mut export_id).unwrap();
                    if let Some(handle) = export_id.handle {
                       return Box::new(WindowWayland::v2(handle, exported));
                    }
                    //should not happen
                    exported.destroy();
                }
            } else if let Some(interface_v1)=&self.interface_foreign_v1 {
                if let Ok(surface_proxy) = wayland_client::protocol::wl_surface::WlSurface::from_id(self.connection.as_ref().unwrap(), id) {
                    //export the handle
                    let mut export_id = WindowWaylandExportID::new();
                    let mut event_queue = self.connection.as_ref().unwrap().new_event_queue();
                    let qhandle = event_queue.handle();
                    let exported=interface_v1.export(&surface_proxy, &qhandle, ());
                    event_queue.roundtrip(&mut export_id).unwrap();
                    if let Some(handle) = export_id.handle {
                       return Box::new(WindowWayland::v1(handle, exported));
                    }
                    //should not happen
                    exported.destroy();
                }
            }
        }
        //not available - return none
        return Box::new(WindowNone::new());
    }

}

#[cfg(not(target_os = "windows"))]
impl wayland_client::Dispatch<wayland_protocols::xdg::foreign::zv2::client::zxdg_exporter_v2::ZxdgExporterV2, ()> for WaylandAppData {
    fn event(_wayland_app_data: &mut Self,_: &wayland_protocols::xdg::foreign::zv2::client::zxdg_exporter_v2::ZxdgExporterV2,_event: wayland_protocols::xdg::foreign::zv2::client::zxdg_exporter_v2::Event,_: &(),_: &wayland_client::Connection,_: &wayland_client::QueueHandle<Self>) {
        //no events
    }
}

#[cfg(not(target_os = "windows"))]
impl wayland_client::Dispatch<wayland_protocols::xdg::foreign::zv1::client::zxdg_exporter_v1::ZxdgExporterV1, ()> for WaylandAppData {
    fn event(_wayland_app_data: &mut Self,_: &wayland_protocols::xdg::foreign::zv1::client::zxdg_exporter_v1::ZxdgExporterV1,_event: wayland_protocols::xdg::foreign::zv1::client::zxdg_exporter_v1::Event,_: &(),_: &wayland_client::Connection,_: &wayland_client::QueueHandle<Self>) {
        //no events
    }
}

#[cfg(not(target_os = "windows"))]
impl wayland_client::Dispatch<wayland_client::protocol::wl_registry::WlRegistry, ()> for WaylandAppData {
    fn event(wayland_app_data: &mut Self,_: &wayland_client::protocol::wl_registry::WlRegistry,event: wayland_client::protocol::wl_registry::Event,_: &(),_: &wayland_client::Connection,_: &wayland_client::QueueHandle<Self>) {
        if let wayland_client::protocol::wl_registry::Event::Global { name, interface, version } = event {
            //println!("wayland protocol announced: {:?}", interface);
            if interface == "zxdg_exporter_v2" {
                wayland_app_data.foreign_v2_protocol_name=Some((name, version));
            } else if interface == "zxdg_exporter_v1" {
                wayland_app_data.foreign_v1_protocol_name=Some((name, version));
            }
        }
    }
}

#[cfg(not(target_os = "windows"))] 
impl Drop for WaylandAppData {
    fn drop(&mut self) {
        if let Some(interface_v2)=&self.interface_foreign_v2 {
            interface_v2.destroy();
        }
        if let Some(interface_v1)=&self.interface_foreign_v1 {
            interface_v1.destroy();
        }
    }
}



#[cfg(not(target_os = "windows"))]
pub(crate) struct WindowWayland {
    cape_window_id: cobia::CapeWindowId,
    v1 : Option<wayland_protocols::xdg::foreign::zv1::client::zxdg_exported_v1::ZxdgExportedV1>,
    v2 : Option<wayland_protocols::xdg::foreign::zv2::client::zxdg_exported_v2::ZxdgExportedV2>
}

#[cfg(not(target_os = "windows"))]
impl WindowWayland {

    pub fn v2(handle: String, exported: wayland_protocols::xdg::foreign::zv2::client::zxdg_exported_v2::ZxdgExportedV2) -> Self {
        Self {
            cape_window_id: cobia::CapeWindowId::wayland(handle, cobia::C::eWaylandForeignProtocol_XDG_foreign_v2),
            v1: None,
            v2: Some(exported),
        }
    }

    pub fn v1(handle: String, exported: wayland_protocols::xdg::foreign::zv1::client::zxdg_exported_v1::ZxdgExportedV1) -> Self {
        Self {
            cape_window_id: cobia::CapeWindowId::wayland(handle, cobia::C::eWaylandForeignProtocol_XDG_foreign_v1),
            v1: Some(exported),
            v2: None,
        }
    }

}

#[cfg(not(target_os = "windows"))]  
impl Window for WindowWayland {
    fn cape_window_id(&self) -> cobia::CapeWindowId {
        self.cape_window_id.clone()
    }
}

#[cfg(not(target_os = "windows"))]  
impl Drop for WindowWayland {
    fn drop(&mut self) {
        if let Some(exported)=&self.v2 {
            exported.destroy();
        }
        if let Some(exported)=&self.v1 {
            exported.destroy();
        }
    }
}

#[cfg(not(target_os = "windows"))]
struct WindowWaylandExportID {
    handle: Option<String>,
}

#[cfg(not(target_os = "windows"))]
impl WindowWaylandExportID {

    pub fn new() -> Self {
        Self {
            handle: None,
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl wayland_client::Dispatch<wayland_protocols::xdg::foreign::zv2::client::zxdg_exported_v2::ZxdgExportedV2, ()> for WindowWaylandExportID {
    fn event(export_id: &mut Self,_: &wayland_protocols::xdg::foreign::zv2::client::zxdg_exported_v2::ZxdgExportedV2,event: wayland_protocols::xdg::foreign::zv2::client::zxdg_exported_v2::Event,_: &(),_: &wayland_client::Connection,_: &wayland_client::QueueHandle<Self>) {
        if let wayland_protocols::xdg::foreign::zv2::client::zxdg_exported_v2::Event::Handle { handle } = event {
            export_id.handle = Some(handle);
        }
    }
}

#[cfg(not(target_os = "windows"))]
impl wayland_client::Dispatch<wayland_protocols::xdg::foreign::zv1::client::zxdg_exported_v1::ZxdgExportedV1, ()> for WindowWaylandExportID {
    fn event(export_id: &mut Self,_: &wayland_protocols::xdg::foreign::zv1::client::zxdg_exported_v1::ZxdgExportedV1,event: wayland_protocols::xdg::foreign::zv1::client::zxdg_exported_v1::Event,_: &(),_: &wayland_client::Connection,_: &wayland_client::QueueHandle<Self>) {
        if let wayland_protocols::xdg::foreign::zv1::client::zxdg_exported_v1::Event::Handle { handle } = event {
            export_id.handle = Some(handle);
        }
    }
}


