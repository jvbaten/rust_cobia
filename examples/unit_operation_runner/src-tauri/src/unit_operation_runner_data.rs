use crate::{
    information_stream::InformationStream, material_stream::MaterialStream, number_formatter, persistence::{PersistReader, PersistWriter}, property_package_instance::{PropertyPackageInstance, PropertyPackageInstanceRef}, real_parameter::RealParameter, status::Status, status_notification::StatusNotification, stream::{Stream, StreamType}, stream_specification::{StreamSpecification, StreamSpecificationType}, table_status::{TableRow, TableStatus}, thermo_status::ThermoStatus, unit::Unit, unit_status::UnitStatus, window::{ Window, WindowNone}
};

#[cfg(target_os = "windows")]
use crate::window::WindowWindows;
#[cfg(not(target_os = "windows"))]
use crate::window::{WindowX11, WaylandAppData};


use base64::prelude::*;
use cobia::{prelude::*, *};
use std::{path::PathBuf, vec};


/// Runtime state for Unit Operation Runner.
///
/// Owns the selected unit operation, selected thermodynamics provider/package,
/// connected streams, specifications, and persistence metadata.
pub(crate) struct UnitOperationRunnerData {
    /// System-wide list of discoverable thermodynamics PMCs.
    thermo_pmcs: CobiaCollection<CapePMCRegistrationDetails>,
    /// System-wide list of discoverable unit-operation PMCs.
    unit_pmcs: CobiaCollection<CapePMCRegistrationDetails>,
    /// Currently selected unit-operation provider registration details.
    unit_provider: Option<CapePMCRegistrationDetails>,
    /// Error captured during unit creation/initialization.
    unit_creation_error: Option<String>,
    /// Unit operation
    unit: Option<Unit>,
    /// Whether the unit operation is in a solved status
    is_solved: bool,
    /// Currently selected thermodynamics provider registration details.
    thermo_provider: Option<CapePMCRegistrationDetails>,
    /// Selected package name (used for manager-based providers).
    package_name: String,
    /// Error captured during thermodynamics creation/initialization.
    thermo_creation_error: Option<String>,
    /// Active property package instance and cached interfaces/data.
    property_package: Option<PropertyPackageInstanceRef>,
    /// Connected material/energy/information streams in port order.
    streams: Vec<Stream>,
    /// Error captured while discovering/connecting ports and streams.
    streams_error: Option<String>,
    /// Bitmask of pending frontend notifications.
    status_notifications: StatusNotification,
    /// Last save/load path for this session.
    path: Option<PathBuf>,
    /// Wayland cached display info
    #[cfg(not(target_os = "windows"))]
    wayland_app_data: Option<WaylandAppData>,
}

impl UnitOperationRunnerData {
    /// Creates a new runner state and discovers available CAPE-OPEN providers.
    ///
    /// # Returns
    /// A fully initialized [`UnitOperationRunnerData`] with empty selections,
    /// no streams, and no errors.
    pub fn new() -> Self {
        let pmc_enumerator = CapePMCEnumerator::new().unwrap();
        let thermo_pmc_types = [
            cape_open::CATEGORYID_PROPERTYPACKAGEMANAGER,
            cape_open::CATEGORYID_STANDALONEPROPERTYPACKAGE,
        ];
        let unit_pmc_types = [cape_open::CATEGORYID_UNITOPERATION];
        let thermo_pmcs = pmc_enumerator.pmcs(&thermo_pmc_types).unwrap();
        let unit_pmcs = pmc_enumerator.pmcs(&unit_pmc_types).unwrap();
        Self {
            //system wide
            thermo_pmcs,
            unit_pmcs,
            //unit operation
            unit_provider: None,
            unit: None,
            unit_creation_error: None,
            is_solved: false,
            //selected thermo
            thermo_provider: None,
            package_name: String::new(),
            property_package: None,
            thermo_creation_error: None,
            //streams
            streams: Vec::new(),
            streams_error: None,
            //status notifications
            status_notifications: StatusNotification::UnitChanged|StatusNotification::ThermoChanged|StatusNotification::StatusChanged,
            //path
            path: None,
            //wayland
            #[cfg(not(target_os = "windows"))]
            wayland_app_data: None,
        }
    }

    /// Converts the native Tauri window handle into a Window.
    ///
    /// This parent handle is used by CAPE dialogs so they are owned by the
    /// application window.
    ///
    /// # Parameters
    /// - `handle`: The raw window handle provided by Tauri, wrapped in a `Result` to handle potential errors.
    /// - `display_handle`: The raw display handle provided by Tauri, wrapped in a `Result` to handle potential errors.
    ///
    /// # Returns
    /// A `Window` that can be used as a parent for CAPE dialogs, or a default/empty id if the handle is invalid or unsupported.
    #[allow(unused)]
    pub fn get_window(&mut self,handle: std::result::Result<raw_window_handle::WindowHandle<'_>,raw_window_handle::HandleError>, display_handle: std::result::Result<raw_window_handle::DisplayHandle<'_>,raw_window_handle::HandleError>) -> Box<dyn Window> {
        if let Ok(w) = handle { 
            match w.as_raw() {
                #[cfg(target_os = "windows")]
                raw_window_handle::RawWindowHandle::Win32(win32_handle) => {
                    return Box::new(WindowWindows::new(win32_handle.hwnd.get() as cobia::C::HWND));
                },
                #[cfg(not(target_os = "windows"))]
                raw_window_handle::RawWindowHandle::Xlib(xlib_handle) => {
                    let display_string= if xlib_handle.visual_id==0 {
                        unsafe { x11::xlib::XDisplayName(std::ptr::null()) }
                    } else {
                        unsafe { x11::xlib::XDisplayString(xlib_handle.visual_id as *mut _) }
                    };
                    return Box::new(WindowX11::new(unsafe { std::ffi::CStr::from_ptr(display_string) }.to_str().unwrap_or_default().to_string(), xlib_handle.window));
                },
                #[cfg(not(target_os = "windows"))]
                raw_window_handle::RawWindowHandle::Wayland(surface) => {
                    if let Ok(dsp) = display_handle {
                        match dsp.as_raw() {
                            raw_window_handle::RawDisplayHandle::Wayland(display) => {
                                if self.wayland_app_data.is_none() {
                                    self.wayland_app_data = Some(WaylandAppData::new(display.display.as_ptr()));
                                }
                                return self.wayland_app_data.as_ref().unwrap().get_window(surface.surface.as_ptr());
                            },
                            _ => {}
                        }
                    };
                },
                _ => {}
            }
        };
        // unknown window type or insufficient data
        Box::new(WindowNone::new())
    }


    /// Returns and clears accumulated status-notification flags.
    ///
    /// # Returns
    /// The notification bitmask that was pending before clearing.
    pub(crate) fn clear_status_notifications(&mut self) -> StatusNotification {
        let notifications = self.status_notifications;
        self.status_notifications = StatusNotification::None;
        notifications
    }

    /// Builds thermodynamics UI status
    ///
    /// # Returns
    /// - `Ok(ThermoStatus)` when status assembly succeeds.
    /// - `Err(e)` when provider/package interrogation fails.
    fn get_thermo_status_internal(&self) -> Result<ThermoStatus, Box<dyn std::error::Error>> {
        let mut status: ThermoStatus = Default::default();
        if self.thermo_creation_error.is_some() {
            status.error = self.thermo_creation_error.clone();
            return Ok(status);
        }
        match &self.property_package {
            Some(_) => {
                //formulate description
                let mut desc = self
                    .thermo_provider
                    .as_ref()
                    .expect("Unknown provider for initialized property package")
                    .get_name()?;
                if !self.package_name.is_empty() {
                    desc.push_str(" / ");
                    desc.push_str(&self.package_name);
                }
                status.active_package = Some(desc);
            }
            None => {
                match &self.thermo_provider {
                    Some(tp) => {
                        status.provider = Some(tp.get_name()?);
                        //create property package manager
                        let ppm = match tp
                            .create_instance(CapePMCCreationFlags::AllowRestrictedThreading)
                        {
                            Ok(obj) => obj,
                            Err(e) => {
                                return Err(format!(
                                    "Error creating instance of provider {}: {}",
                                    tp.get_name()?,
                                    e
                                )
                                .into());
                            }
                        };
                        let utils = match cape_open_1_2::CapeUtilities::from_object(&ppm) {
                            Ok(u) => u,
                            Err(e) => {
                                return Err(format!(
                                    "Error getting ICapeUtilities from provider {}: {}",
                                    tp.get_name()?,
                                    e
                                )
                                .into());
                            }
                        };
                        match utils.initialize() {
                            Ok(_) => {}
                            Err(e) => {
                                return Err(format!(
                                    "Error initializing provider {}: {}",
                                    tp.get_name()?,
                                    e
                                )
                                .into());
                            }
                        };
                        //get a list of package names
                        let ppm = match cape_open_1_2::CapeThermoPropertyPackageManager::from_object(
                            &ppm,
                        ) {
                            Ok(ppm) => ppm,
                            Err(_) => {
                                let _ = utils.terminate();
                                return Err(format!(
                                    "Provider {} is not a property package manager",
                                    tp.get_name()?
                                )
                                .into());
                            }
                        };
                        let mut names = CapeArrayStringVec::new();
                        match ppm.get_property_package_list(&mut names) {
                            Ok(_) => {
                                let mut names = names.as_string_vec();
                                names.sort_by_cached_key(|s| s.to_lowercase());
                                status.package_list = Some(names);
                            }
                            Err(e) => {
                                let _ = utils.terminate();
                                return Err(format!(
                                    "Error getting property package names from {}: {}",
                                    tp.get_name()?,
                                    e
                                )
                                .into());
                            }
                        };
                        //terminate provider
                        let _ = utils.terminate();
                    }
                    None => {
                        //return a list of providers
                        let mut pairs = self
                            .thermo_pmcs
                            .iter()
                            .map(|pmc| {
                                (pmc.get_name().unwrap(), pmc.get_uuid().unwrap().as_string())
                            })
                            .collect::<Vec<(String, String)>>();
                        pairs.sort_by_cached_key(|s| s.0.to_lowercase());
                        let mut names: Vec<String> = Vec::with_capacity(pairs.len());
                        let mut uuids: Vec<String> = Vec::with_capacity(pairs.len());
                        for (name, uuid) in pairs {
                            names.push(name);
                            uuids.push(uuid);
                        }
                        status.provider_list = Some(names);
                        status.provider_uuid_list = Some(uuids);
                    }
                };
            }
        }
        Ok(status)
    }

    /// Returns thermodynamics UI status, converting internal errors to a user
    /// visible `error` field.
    ///
    /// # Returns
    /// A [`ThermoStatus`] for the frontend.
    pub fn get_thermo_status(&self) -> ThermoStatus {
        match self.get_thermo_status_internal() {
            Ok(status) => status,
            Err(e) => {
                ThermoStatus {
                    error: Some(format!("Error: {}", e)),
                    ..Default::default()
                }
            }
        }
    }

    /// Clears the selected thermodynamics provider/package and related state.
    pub fn clear_thermo(&mut self) {
        if self.thermo_provider.is_some()
            || !self.package_name.is_empty()
            || self.property_package.is_some()
        {
            self.thermo_provider = None;
            self.package_name.clear();
            self.thermo_creation_error = None;
            if let Some(pp) = self.property_package.take() {
                pp.write().unwrap().unset_context_material();
            }
            self.status_notifications |=
                StatusNotification::InfoChanged | StatusNotification::LogChanged;
            self.thermo_changed();
        }
    }

    /// Selects a thermodynamics provider by UUID and also creates a
    /// stand-alone property package.
    ///
    /// # Parameters
    /// - `provider`: Provider UUID string.
    ///
    /// # Returns
    /// - `Ok(())` when provider selection succeeds.
    /// - `Err(e)` when lookup, creation, or initialization fails.
    fn select_thermo_provider_internal(
        &mut self,
        provider: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //convert string to uuid
        let uuid = CapeUUID::from_string(provider).expect("Invalid UUID string");
        //find provider
        let pmc_option = self.thermo_pmcs.iter().find(|pmc| match pmc.get_uuid() {
            Ok(pmc_uuid) => pmc_uuid == uuid,
            Err(_) => false,
        });
        match pmc_option {
            Some(tp) => {
                self.clear_thermo();
                self.thermo_provider = Some(tp.clone());
                //if this is a stand-alone property package, create it now
                let is_stand_alone_pp : bool = tp.implements_cat_id(&cape_open::CATEGORYID_STANDALONEPROPERTYPACKAGE).unwrap_or_default();
                if is_stand_alone_pp {
                    let thermo = match PropertyPackageInstance::new_standalone(&tp, None, CapePMCCreationFlags::Default) {
                        Ok(instance) => instance,
                        Err(e) => {
                            self.thermo_provider = None;
                            return Err(format!(
                                "Error creating stand-alone property package from provider {}: {}",
                                tp.get_name()?,
                                e
                            )
                            .into());
                        }
                    };
                    self.clear_thermo();
                    self.thermo_provider = Some(tp.clone());
                    self.property_package = Some(PropertyPackageInstanceRef::new(thermo.into()));
                    self.thermo_changed();
                    self.status_notifications |=
                        StatusNotification::InfoChanged | StatusNotification::LogChanged;
                } else {
                    self.status_notifications |=
                        StatusNotification::ThermoChanged | StatusNotification::StatusChanged;
                }
                Ok(())
            }
            None => Err(format!("Provider with UUID {} not found", provider).into()),
        }
    }

    /// Public wrapper around provider selection that captures errors into state.
    ///
    /// # Parameters
    /// - `provider`: Provider UUID string.
    pub fn select_thermo_provider(&mut self, provider: &str) {
        match self.select_thermo_provider_internal(provider) {
            Ok(_) => {}
            Err(e) => {
                self.clear_thermo();
                self.status_notifications |=
                    StatusNotification::StatusChanged | StatusNotification::LogChanged;
                let err = format!("{}", e);
                self.thermo_creation_error = Some(err.clone());
            }
        }
    }

    /// Selects a package name from the current thermodynamics provider and creates package.
    ///
    /// # Parameters
    /// - `package`: Package name exposed by the provider.
    ///
    /// # Returns
    /// - `Ok(())` when package creation and initialization succeed.
    /// - `Err(e)` on provider, package, or initialization failures.
    fn select_thermo_package_internal(
        &mut self,
        package: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //create property package manager
        let tp = match &self.thermo_provider {
            Some(tp) => tp.clone(),
            None => {
                return Err("No provider selected".into());
            }
        };
        let thermo = match PropertyPackageInstance::new_from_manager(&tp, package, None, CapePMCCreationFlags::Default) {
            Ok(instance) => instance,
            Err(e) => {
                self.thermo_provider = None;
                return Err(e);
            }
        };
        self.clear_thermo();
        self.thermo_provider = Some(tp);
        self.property_package = Some(PropertyPackageInstanceRef::new(thermo.into()));
        self.thermo_changed();
        self.status_notifications |=
            StatusNotification::InfoChanged | StatusNotification::LogChanged;
        Ok(())
    }

    /// Public wrapper around package selection that captures errors into state.
    ///
    /// # Parameters
    /// - `package`: Package name exposed by the selected provider.
    pub fn select_thermo_package(&mut self, package: &str) {
        match self.select_thermo_package_internal(package) {
            Ok(_) => {}
            Err(e) => {
                self.clear_thermo();
                self.status_notifications |= StatusNotification::StatusChanged;
                self.thermo_creation_error = Some(format!("{}", e));
            }
        }
    }

    /// Opens the thermodynamics component edit dialog.
    ///
    /// # Parameters
    /// - `parent`: Native window handle used as parent for modal UI.
    ///
    /// # Returns
    /// - `Ok(())` when edit handling succeeds.
    /// - `Err(e)` if no package is selected or edit operations fail.
    pub fn edit_thermo(&mut self, parent: CapeWindowId) -> Result<(), Box<dyn std::error::Error>> {
        match self.property_package {
            Some(ref pp) => {
                if pp.write().unwrap().edit(
                    parent,
                    self.thermo_provider.as_ref().unwrap(),
                    &self.package_name,
                )? {
                    self.thermo_changed();
                };
                Ok(())
            }
            None => Err("No property package; cannot edit".into()),
        }
    }

    /// Builds unit-operation UI status, propagating detailed provider errors.
    ///
    /// # Returns
    /// - `Ok(UnitStatus)` when status assembly succeeds.
    /// - `Err(e)` when provider interrogation fails.
    fn get_unit_status_internal(&self) -> Result<UnitStatus, Box<dyn std::error::Error>> {
        let mut status: UnitStatus = Default::default();
        if self.unit_creation_error.is_some() {
            status.error = self.unit_creation_error.clone();
            return Ok(status);
        }
        match &self.unit_provider {
            Some(_) => {
                status.active_unit = Some(
                    self.unit_provider
                        .as_ref()
                        .expect("Unable to get unit provider ref")
                        .get_name()?,
                );
            }
            None => {
                //return a list of providers
                let mut pairs = self
                    .unit_pmcs
                    .iter()
                    .map(|pmc| (pmc.get_name().unwrap(), pmc.get_uuid().unwrap().as_string()))
                    .collect::<Vec<(String, String)>>();
                pairs.sort_by_cached_key(|s| s.0.to_lowercase());
                let mut names: Vec<String> = Vec::with_capacity(pairs.len());
                let mut uuids: Vec<String> = Vec::with_capacity(pairs.len());
                for (name, uuid) in pairs {
                    names.push(name);
                    uuids.push(uuid);
                }
                status.provider_list = Some(names);
                status.provider_uuid_list = Some(uuids);
            }
        }
        Ok(status)
    }

    /// Returns unit-operation UI status, converting internal errors to a user
    /// visible `error` field.
    ///
    /// # Returns
    /// A [`UnitStatus`] snapshot for the frontend.
    pub fn get_unit_status(&self) -> UnitStatus {
        match self.get_unit_status_internal() {
            Ok(status) => status,
            Err(e) => {
                UnitStatus {
                    error: Some(format!("Error: {}", e)),
                    ..Default::default()
                }
            }
        }
    }

    /// Terminates and clears the current unit operation instance, if any.
    pub fn terminate_unit(&mut self) {
        self.unit_provider = None;
        self.unit_creation_error = None;
        self.unit = None;
    }

    /// Clears unit selection, instance, logs, and dependent status.
    pub fn clear_unit(&mut self) {
        self.terminate_unit();
        self.unit_changed();
        self.status_notifications |=
            StatusNotification::InfoChanged | StatusNotification::LogChanged;
    }

    /// Selects and initializes a unit operation by provider UUID.
    ///
    /// # Parameters
    /// - `unit_selector`: Unit provider UUID string.
    ///
    /// # Returns
    /// - `Ok(())` when creation and initialization succeed.
    /// - `Err(e)` when lookup, creation, or initialization fails.
    fn select_unit_internal(
        &mut self,
        unit_selector: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.unit_creation_error = None;
        //convert string to uuid
        let uuid = CapeUUID::from_string(unit_selector).expect("Invalid UUID string");
        //find provider
        let unit_option = self.unit_pmcs.iter().find(|pmc| match pmc.get_uuid() {
            Ok(pmc_uuid) => pmc_uuid == uuid,
            Err(_) => false,
        });
        match unit_option {
            Some(up) => {
                self.clear_unit();
                self.unit_provider = Some(up.clone());
                //create unit operation
                self.unit = match Unit::new(&up, None) {
                    Ok(unit) => Some(unit),
                    Err(e) => {
                        self.unit_creation_error = Some(format!("{}", e));
                        None
                    }
                };
                self.unit_changed();
                Ok(())
            }
            None => Err(format!("Unit Operation with UUID {} not found", unit_selector).into()),
        }
    }

    /// Public wrapper around unit selection that captures logs and errors.
    ///
    /// # Parameters
    /// - `unit_selector`: Unit provider UUID string.
    pub fn select_unit(&mut self, unit_selector: &str) {
        match self.select_unit_internal(unit_selector) {
            Ok(_) => {}
            Err(e) => {
                self.status_notifications |= StatusNotification::StatusChanged;
                self.clear_unit();
                let err = format!("{}", e);
                self.unit_creation_error = Some(err);
            }
        }
        self.status_notifications |=
            StatusNotification::InfoChanged | StatusNotification::LogChanged;
    }

    /// Opens the unit operation edit dialog.
    ///
    /// # Parameters
    /// - `parent`: Native window handle used as parent for modal UI.
    ///
    /// # Returns
    /// - `Ok(())` when edit handling succeeds.
    pub fn edit_unit(&mut self, parent: CapeWindowId) -> Result<(), Box<dyn std::error::Error>> {
        match self.unit {
            Some(ref mut unit) => {
                if unit.edit(parent, self.unit_provider.as_ref().unwrap())? {
                    self.unit_changed();
                };
                Ok(())
            }
            None => Err("No unit operation; cannot edit".into()),
        }
    }

    /// Creates or reuses a [`Stream`] object for a unit port and ensures
    /// port-to-stream connection.
    ///
    /// # Parameters
    /// - `port_name`: CAPE-OPEN port name.
    /// - `port`: Unit port interface.
    ///
    /// # Returns
    /// - `Ok(Stream)` for a connected stream object.
    /// - `Err(e)` when port metadata, thermo, or connection operations fail.
    fn stream_from_port(
        &mut self,
        port_name: &CapeStringImpl,
        port: &cape_open_1_2::CapeUnitPort,
    ) -> Result<Stream, Box<dyn std::error::Error>> {
        let is_feed = match port.get_direction() {
            Ok(dir) => match dir {
                cape_open_1_2::CapePortDirection::CapeInlet => true,
                cape_open_1_2::CapePortDirection::CapeOutlet => false,
                _ => {
                    return Err(format!(
                        "Unsupported port direction for port {}: {:?}",
                        port_name, dir
                    )
                    .into());
                }
            },
            Err(e) => {
                return Err(
                    format!("Error getting port direction for port {}: {}", port_name, e).into(),
                );
            }
        };
        match port.get_port_type() {
            Ok(port_type) => {
                match port_type {
                    cape_open_1_2::CapePortType::CapeMaterial => {
                        if self.property_package.is_none() {
                            return Err("Please configure thermodynamics.".into());
                        }
                        //check if already connected
                        if let Ok(obj) = port.get_connected_object() && unsafe { MaterialStream::check_borrow(&obj) }.is_some() {
                            return Ok(Stream::material(is_feed, port_name, obj));
                        };
                        //create a stream object with the port info and connect
                        let material_object: CapeObject = MaterialStream::with_property_package(
                            self.property_package.as_ref().unwrap().clone(),
                        );
                        //connect
                        match port.connect(&material_object) {
                            Ok(_) => Ok(Stream::material(is_feed, port_name, material_object)),
                            Err(e) => Err(format!(
                                "Error connecting stream to port {}: {}",
                                port_name, e
                            )
                            .into()),
                        }
                    }
                    cape_open_1_2::CapePortType::CapeEnergy
                    | cape_open_1_2::CapePortType::CapeInformation => {
                        //check if already connected
                        if let Ok(obj) = port.get_connected_object() && let Some(istr) = unsafe { InformationStream::check_borrow_mut(&obj) } {
                            match istr.update_from_port(
                                port,
                                is_feed,
                                port_type == cape_open_1_2::CapePortType::CapeEnergy,
                                port_name,
                            ) {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(format!(
                                        "Error updating stream from port {}: {}",
                                        port_name, e
                                    )
                                    .into());
                                }
                            };
                            return if port_type == cape_open_1_2::CapePortType::CapeEnergy {
                                Ok(Stream::energy(is_feed, port_name, obj))
                            } else {
                                Ok(Stream::information(is_feed, port_name, obj))
                            };
                        };
                        //create a stream object with the port info and connect
                        let information_stream: CapeObject = InformationStream::create();
                        match unsafe { InformationStream::borrow_mut(&information_stream) }
                            .update_from_port(
                                port,
                                is_feed,
                                port_type == cape_open_1_2::CapePortType::CapeEnergy,
                                port_name,
                            ) {
                            Ok(_) => {}
                            Err(e) => {
                                return Err(format!(
                                    "Error updating stream from port {}: {}",
                                    port_name, e
                                )
                                .into());
                            }
                        };
                        //connect
                        match port.connect(&information_stream) {
                            Ok(_) => Ok(if port_type == cape_open_1_2::CapePortType::CapeEnergy {
                                Stream::energy(is_feed, port_name, information_stream)
                            } else {
                                Stream::information(is_feed, port_name, information_stream)
                            }),
                            Err(e) => Err(format!(
                                "Error connecting stream to port {}: {}",
                                port_name, e
                            )
                            .into()),
                        }
                    }
                    _ => Err(format!(
                        "Unsupported port type for port {}: {:?}",
                        port_name, port_type
                    )
                    .into()),
                }
            }
            Err(e) => Err(format!("Error getting port type for port: {}", e).into()),
        }
    }

    /// Rebuilds stream list from unit ports and reapplies stream specifications.
    pub fn update_streams(&mut self) {
        self.streams.clear();
        self.status_notifications |=
            StatusNotification::StreamsChanged | StatusNotification::StatusChanged;
        match self.unit {
            None => {
                //no unit, no streams
            }
            Some(ref unit) => {
                //get the port collection of the unit operation
                let port_collection = match unit.ports() {
                    Ok(p) => p,
                    Err(e) => {
                        self.streams_error = Some(format!(
                            "Error getting port collection from unit operation: {}",
                            e
                        ));
                        return;
                    }
                };
                //loop over the ports
                let mut new_streams = Vec::with_capacity(port_collection.size());
                for port in port_collection {
                    let mut port_name = CapeStringImpl::new();
                    match cape_open_1_2::CapeIdentification::from_object(&port) {
                        Ok(id) => match id.get_component_name(&mut port_name) {
                            Ok(_) => {}
                            Err(e) => {
                                self.streams_error =
                                    Some(format!("Error getting port name: {}", e));
                                return;
                            }
                        },
                        Err(e) => {
                            self.streams_error = Some(format!(
                                "Error getting ICapeIdentification from port: {}",
                                e
                            ));
                            return;
                        }
                    };
                    match self.stream_from_port(&port_name, &port) {
                        Ok(s) => {
                            new_streams.push(s);
                            let s = new_streams.last_mut().unwrap();
                            //update specs
                            s.apply_specifications();
                        }
                        Err(e) => {
                            self.streams_error =
                                Some(format!("Error connecting port {}: {}", port_name, e));
                            return;
                        }
                    };
                }
                //all ok - update streams
                self.streams = new_streams;
            }
        };
    }

    /// Handles thermodynamics changes by updating/clearing material streams and
    /// refreshing dependent status.
    pub(crate) fn thermo_changed(&mut self) {
        match self.property_package {
            Some(_) => {
                //update property package in all material streams
                for stream in self.streams.iter_mut() {
                    if let StreamType::Material(obj) = &mut stream.stream_type {
                        let material_stream = unsafe { MaterialStream::borrow_mut(obj) }; //note that unchecked borrow_mut is used here, as we are sure that obj points to a MaterialStream 
                        material_stream.update_property_package(
                            self.property_package.as_ref().unwrap().clone(),
                        );
                    }
                }
            }
            None => {
                //no property package, clear all streams
                self.streams.clear();
            }
        }
        self.status_notifications |=
            StatusNotification::ThermoChanged | StatusNotification::StatusChanged;
        self.update_streams();
        self.is_solved = false;
    }

    /// Handles unit-operation changes by refreshing streams and status flags.
    fn unit_changed(&mut self) {
        self.status_notifications |= StatusNotification::UnitChanged
            | StatusNotification::StatusChanged
            | StatusNotification::ParametersChanged
            | StatusNotification::ReportsChanged;
        self.update_streams();
        self.is_solved = false;
    }

    /// Computes overall runner status based on configuration, validation, and
    /// stream specification completeness.
    ///
    /// # Returns
    /// Current [`Status`] for UI display and command gating.
    pub fn get_status(&self) -> Status {
        if let Some(unit) = &self.unit
            && self.is_solved
        {
            return match unit.calculation_error() {
                Some(ref e) => Status::Error(format!("Calculate failed: {}", e)),
                None => Status::Solved,
            };
        }
        //check if there are any material streams
        let mut has_material_streams = false;
        for stream in self.streams.iter() {
            if let StreamType::Material(_) = stream.stream_type {
                has_material_streams = true;
                break;
            }
        }
        if has_material_streams {
            //we need thermo
            match self.property_package {
                Some(_) => {}
                None => {
                    return Status::Incomplete("Please configure thermodynamics.".to_string());
                }
            }
        }
        //check if there is a unit
        match &self.unit {
            Some(unit) => {
                match self.streams_error {
                    Some(ref e) => {
                        Status::Error(format!("Streams error: {}", e))
                    }
                    None => {
                        //validate the unit operation
                        match unit.status() {
                            Some(status) => status,
                            None => {
                                //check feed stream specifications errors
                                for stream in self.streams.iter() {
                                    if stream.feed()
                                        && let Some(s) = stream.specification_error() {
                                            return Status::Incomplete(format!(
                                                "Stream {}: {}",
                                                stream.name, s
                                            ));
                                        }
                                }
                                //check incomplete feed streams
                                for stream in self.streams.iter() {
                                    if stream.feed() && !stream.is_specified() {
                                        return Status::Incomplete(format!(
                                            "Stream {} is not fully specified",
                                            stream.name
                                        ));
                                    }
                                }
                                //no errors, ready to run
                                Status::Ready
                            }
                        }
                    }
                }
            }
            None => Status::Incomplete("Please configure unit operation.".to_string()),
        }
    }

    /// Builds the stream table payload used by the frontend stream editor/view.
    ///
    /// # Returns
    /// A [`TableStatus`] containing stream headers and rows.
    pub fn get_stream_table(&self) -> TableStatus {
        //headers
        let mut headers: Vec<String> = Vec::with_capacity(2 + self.streams.len());
        headers.push("".into()); //property name
        for stream in self.streams.iter() {
            headers.push(stream.name.clone());
        }
        headers.push("".into()); //unit of measure
        let mut rows = Vec::new();
        //check which stream types are there, and check which phases are present in material streams
        let number_of_phases = match self.property_package {
            Some(_) => {
                let pp = self.property_package.as_ref().unwrap().read().unwrap();
                pp.get_number_of_phases()
            }
            None => 0,
        };
        let mut has_material_streams = false;
        let mut phase_presence: Vec<bool> = vec![false; number_of_phases];
        let mut has_energy_streams = false;
        let mut has_temperature_low = false;
        let mut has_temperature_high = false;
        let mut has_axis_freq = false;
        let mut information_stream_map_in =
            std::collections::HashMap::<String, (&str, &str)>::new(); //map from sortable key to (name,uom)
        let mut information_stream_map_out =
            std::collections::HashMap::<String, (&str, &str)>::new(); //map from sortable key to (name,uom)
        for stream in self.streams.iter() {
            match &stream.stream_type {
                StreamType::Material(material_stream) => {
                    has_material_streams = true;
                    for phase_index in unsafe { MaterialStream::borrow(material_stream) }
                        .get_present_phase_indices()
                    {
                        phase_presence[phase_index] = true;
                    }
                }
                StreamType::Energy(energy_stream) => {
                    has_energy_streams = true;
                    for par in unsafe { InformationStream::borrow(energy_stream) }.iter() {
                        match unsafe { RealParameter::borrow(par) }.name.as_ref() {
                            "temperatureLow" => {
                                has_temperature_low = true;
                            }
                            "temperatureHigh" => {
                                has_temperature_high = true;
                            }
                            "axisFrequency" => {
                                has_axis_freq = true;
                            }
                            _ => {}
                        }
                    }
                }
                StreamType::Information(information_stream) => {
                    for par in unsafe { InformationStream::borrow(information_stream) }.iter() {
                        let par = unsafe { RealParameter::borrow(par) };
                        let name: &str = par.name.as_ref();
                        let uom: &str = par.unit_of_measure.as_ref();
                        let key = format!("{}_{:?}", name, uom);
                        if stream.feed() {
                            let _ = information_stream_map_in.insert(key, (name, uom));
                        } else {
                            let _ = information_stream_map_out.insert(key, (name, uom));
                        }
                    }
                }
            }
        }
        // all energy stream rows
        if has_energy_streams {
            let energy_header = TableRow::header("Energy", None);
            //work
            let mut work = TableRow::data_row("work".into(), Some(&energy_header));
            work.add_cell("Work");
            let mut temperature_low = if has_temperature_low {
                let mut row = TableRow::data_row("temperatureLow".into(), Some(&energy_header));
                row.add_cell("Lower temperature");
                Some(row)
            } else {
                None
            };
            let mut temperature_high = if has_temperature_high {
                let mut row = TableRow::data_row("temperatureHigh".into(), Some(&energy_header));
                row.add_cell("Upper temperature");
                Some(row)
            } else {
                None
            };
            let mut axis_freq = if has_axis_freq {
                let mut row = TableRow::data_row("axisFrequency".into(), Some(&energy_header));
                row.add_cell("Axis frequency");
                Some(row)
            } else {
                None
            };
            for stream in self.streams.iter() {
                match &stream.stream_type {
                    StreamType::Energy(obj) => {
                        let energy_stream = unsafe { InformationStream::borrow(obj) }; //note that unchecked borrow is used here, as we are sure that obj points to a MaterialStream
                        let mut par_work = None;
                        let mut par_temperature_low = None;
                        let mut par_temperature_high = None;
                        let mut par_axis_freq = None;
                        for par in energy_stream.iter() {
                            let par = unsafe { RealParameter::borrow(par) };
                            match par.name.as_ref() {
                                "work" => {
                                    par_work.replace(par);
                                }
                                "temperatureLow" => {
                                    par_temperature_low.replace(par);
                                }
                                "temperatureHigh" => {
                                    par_temperature_high.replace(par);
                                }
                                "axisFrequency" => {
                                    par_axis_freq.replace(par);
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        if let Some(par_work) = par_work {
                            if stream.feed() {
                                work.add_field(
                                    number_formatter::format_number(Some(par_work.value)),
                                    Some(format!("stream_parameter\twork\t{}", stream.name)),
                                );
                            } else {
                                work.add_result(number_formatter::format_number(Some(
                                    par_work.value,
                                )));
                            }
                        } else {
                            work.add_empty();
                        }
                        if let Some(ref mut row) = temperature_low {
                            if let Some(par_temperature_low) = par_temperature_low {
                                if stream.feed() {
                                    row.add_field(
                                        number_formatter::format_number(Some(
                                            par_temperature_low.value,
                                        )),
                                        Some(format!(
                                            "stream_parameter\ttemperatureLow\t{}",
                                            stream.name
                                        )),
                                    );
                                } else {
                                    row.add_result(number_formatter::format_number(Some(
                                        par_temperature_low.value,
                                    )));
                                }
                            } else {
                                row.add_empty();
                            }
                        }
                        if let Some(ref mut row) = temperature_high {
                            if let Some(par_temperature_high) = par_temperature_high {
                                if stream.feed() {
                                    row.add_field(
                                        number_formatter::format_number(Some(
                                            par_temperature_high.value,
                                        )),
                                        Some(format!(
                                            "stream_parameter\ttemperatureHigh\t{}",
                                            stream.name
                                        )),
                                    );
                                } else {
                                    row.add_result(number_formatter::format_number(Some(
                                        par_temperature_high.value,
                                    )));
                                }
                            } else {
                                row.add_empty();
                            }
                        }
                        if let Some(ref mut row) = axis_freq {
                            if let Some(par_axis_freq) = par_axis_freq {
                                if stream.feed() {
                                    row.add_field(
                                        number_formatter::format_number(Some(par_axis_freq.value)),
                                        Some(format!(
                                            "stream_parameter\taxisFrequency\t{}",
                                            stream.name
                                        )),
                                    );
                                } else {
                                    row.add_result(number_formatter::format_number(Some(
                                        par_axis_freq.value,
                                    )));
                                }
                            } else {
                                row.add_empty();
                            }
                        }
                    }
                    _ => {
                        //blanks
                        work.add_empty();
                        if let Some(ref mut row) = temperature_low {
                            row.add_empty();
                        }
                        if let Some(ref mut row) = temperature_high {
                            row.add_empty();
                        }
                        if let Some(ref mut row) = axis_freq {
                            row.add_empty();
                        }
                    }
                }
            }
            rows.push(energy_header);
            work.add_cell("W");
            rows.push(work);
            if let Some(mut row) = temperature_low.take() {
                row.add_cell("K");
                rows.push(row);
            };
            if let Some(mut row) = temperature_high.take() {
                row.add_cell("K");
                rows.push(row);
            };
            if let Some(mut row) = axis_freq.take() {
                row.add_cell("1/s");
                rows.push(row);
            };
        }
        //all information stream rows
        if !information_stream_map_in.is_empty() || !information_stream_map_out.is_empty() {
            let mut in_keys = information_stream_map_in
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            in_keys.sort();
            let mut out_keys = information_stream_map_out
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            out_keys.sort();
            let mut keys = in_keys;
            for key in out_keys {
                if !information_stream_map_in.contains_key(&key) {
                    keys.push(key);
                }
            }
            let info_header = TableRow::header("Information", None);
            rows.push(info_header.clone());
            let mut information_stream_map = information_stream_map_in;
            for (key, uom) in information_stream_map_out {
                let _ = information_stream_map.insert(key.clone(), uom);
            }
            for key in keys {
                let (name, uom) = information_stream_map.get(&key).unwrap();
                let mut row = TableRow::data_row(key.clone(), Some(&info_header));
                row.add_cell(name);
                for stream in self.streams.iter() {
                    match &stream.stream_type {
                        StreamType::Information(obj) => {
                            let information_stream = unsafe { InformationStream::borrow(obj) }; //note that unchecked borrow is used here, as we are sure that obj points to a MaterialStream
                            let mut par_value = None;
                            for par in information_stream.iter() {
                                let par = unsafe { RealParameter::borrow(par) };
                                let par_name: &str = par.name.as_ref();
                                let par_uom: &str = par.unit_of_measure.as_ref();
                                if *name == par_name && *uom == par_uom {
                                    par_value.replace(par.value);
                                    break;
                                }
                            }
                            match &par_value {
                                Some(_) => {
                                    if stream.feed() {
                                        row.add_field(
                                            number_formatter::format_number(par_value),
                                            Some(format!(
                                                "stream_parameter\t{}\t{}",
                                                name, stream.name
                                            )),
                                        );
                                    } else {
                                        row.add_result(number_formatter::format_number(par_value));
                                    }
                                }
                                None => {
                                    row.add_empty();
                                }
                            }
                        }
                        _ => {
                            //blank
                            row.add_empty();
                        }
                    }
                }
                row.add_cell(uom);
                rows.push(row);
            }
        }
        //all material stream rows
        if has_material_streams && let Some(ref pp) = self.property_package {
            let pp = pp.read().unwrap();
            let single_compound = pp.compound_names.len() == 1;
            let overall_header = TableRow::header("Overall", None);
            //make the overall headers
            let mut total_molar_flow =
                TableRow::data_row("Total Molar Flow".into(), Some(&overall_header));
            total_molar_flow.add_cell("Flow Rate");
            let mut total_mass_flow =
                TableRow::data_row("Total Mass Flow".into(), Some(&overall_header));
            total_mass_flow.add_cell("Flow Rate");
            let mole_fraction_header = TableRow::header("Mole fractions", Some(&overall_header));
            let mut mole_fraction = Vec::new();
            for compound in pp.compound_names.iter() {
                let id = format!("molefraction_{}", compound);
                let mut row = TableRow::data_row(id, Some(&mole_fraction_header));
                row.add_cell(compound);
                mole_fraction.push(row);
            }
            let mass_fraction_header = TableRow::header("Mass fractions", Some(&overall_header));
            let mut mass_fraction = Vec::new();
            for compound in pp.compound_names.iter() {
                let id = format!("massfraction_{}", compound);
                let mut row = TableRow::data_row(id, Some(&mass_fraction_header));
                row.add_cell(compound);
                mass_fraction.push(row);
            }
            let mole_flow_rate_header =
                TableRow::header("Component Molar Flow Rate", Some(&overall_header));
            let mut mole_flow = Vec::new();
            for compound in pp.compound_names.iter() {
                let id = format!("molarflow_{}", compound);
                let mut row = TableRow::data_row(id, Some(&mole_flow_rate_header));
                row.add_cell(compound);
                mole_flow.push(row);
            }
            let mass_flow_rate_header =
                TableRow::header("Component Mass Flow Rate", Some(&overall_header));
            let mut mass_flow = Vec::new();
            for compound in pp.compound_names.iter() {
                let id = format!("massflow_{}", compound);
                let mut row = TableRow::data_row(id, Some(&mass_flow_rate_header));
                row.add_cell(compound);
                mass_flow.push(row);
            }
            let mut temperature = TableRow::data_row("Temperature".into(), Some(&overall_header));
            temperature.add_cell("Temperature");
            let mut pressure = TableRow::data_row("Pressure".into(), Some(&overall_header));
            pressure.add_cell("Pressure");
            let mut molar_vapor_fraction =
                TableRow::data_row("MolarVaporFraction".into(), Some(&overall_header));
            molar_vapor_fraction.add_cell("Vapor Fraction");
            let mut mass_vapor_fraction =
                TableRow::data_row("MassVaporFraction".into(), Some(&overall_header));
            mass_vapor_fraction.add_cell("Vapor Fraction");
            let mut molar_enthalpy =
                TableRow::data_row("MolarEnthalpy".into(), Some(&overall_header));
            molar_enthalpy.add_cell("Enthalpy");
            let mut specific_enthalpy =
                TableRow::data_row("SpecificEnthalpy".into(), Some(&overall_header));
            specific_enthalpy.add_cell("Enthalpy");
            let mut molar_volume = TableRow::data_row("MolarVolume".into(), Some(&overall_header));
            molar_volume.add_cell("Volume");
            let mut specific_volume =
                TableRow::data_row("SpecificVolume".into(), Some(&overall_header));
            specific_volume.add_cell("Volume");
            let mut molar_density =
                TableRow::data_row("MolarDensity".into(), Some(&overall_header));
            molar_density.add_cell("Density");
            let mut mass_density = TableRow::data_row("MassDensity".into(), Some(&overall_header));
            mass_density.add_cell("Density");
            let mut molar_entropy =
                TableRow::data_row("MolarEntropy".into(), Some(&overall_header));
            molar_entropy.add_cell("Entropy");
            let mut specific_entropy =
                TableRow::data_row("SpecificEntropy".into(), Some(&overall_header));
            specific_entropy.add_cell("Entropy");
            for stream in self.streams.iter() {
                match &stream.stream_type {
                    StreamType::Material(obj) => {
                        let material_stream = unsafe { MaterialStream::borrow(obj) }; //note that unchecked borrow is used here, as we are sure that obj points to a MaterialStream
                        if stream.feed() {
                            //add specs
                            let (flash_specs, compound_spec, rate_spec) =
                                material_stream.get_specification_usage();
                            let (
                                mole_fractions,
                                mass_fractions,
                                molar_flow_rates,
                                mass_flow_rates,
                                molar_total_flow,
                                mass_total_flow,
                            ) = material_stream.get_compound_specification_values(
                                &pp,
                                &compound_spec,
                                &rate_spec,
                            );
                            total_molar_flow.add_field(
                                number_formatter::format_number(molar_total_flow.0),
                                if molar_total_flow.1 {
                                    Some(format!("total_molar_flow\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            total_mass_flow.add_field(
                                number_formatter::format_number(mass_total_flow.0),
                                if mass_total_flow.1 {
                                    Some(format!("total_mass_flow\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            for (index, (value, is_field)) in mole_fractions.into_iter().enumerate()
                            {
                                mole_fraction[index].add_field(
                                    number_formatter::format_number(value),
                                    if is_field {
                                        Some(format!(
                                            "mole_fraction\t{}\t{}",
                                            stream.name, pp.compound_names[index]
                                        ))
                                    } else {
                                        None
                                    },
                                );
                            }
                            for (index, (value, is_field)) in mass_fractions.into_iter().enumerate()
                            {
                                mass_fraction[index].add_field(
                                    number_formatter::format_number(value),
                                    if is_field {
                                        Some(format!(
                                            "mass_fraction\t{}\t{}",
                                            stream.name, pp.compound_names[index]
                                        ))
                                    } else {
                                        None
                                    },
                                );
                            }
                            for (index, (value, is_field)) in
                                molar_flow_rates.into_iter().enumerate()
                            {
                                mole_flow[index].add_field(
                                    number_formatter::format_number(value),
                                    if is_field {
                                        Some(format!(
                                            "molar_flow_rate\t{}\t{}",
                                            stream.name, pp.compound_names[index]
                                        ))
                                    } else {
                                        None
                                    },
                                );
                            }
                            for (index, (value, is_field)) in
                                mass_flow_rates.into_iter().enumerate()
                            {
                                mass_flow[index].add_field(
                                    number_formatter::format_number(value),
                                    if is_field {
                                        Some(format!(
                                            "mass_flow_rate\t{}\t{}",
                                            stream.name, pp.compound_names[index]
                                        ))
                                    } else {
                                        None
                                    },
                                );
                            }
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::Temperature,
                                &flash_specs,
                            );
                            temperature.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("temperature\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::Pressure,
                                &flash_specs,
                            );
                            pressure.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("pressure\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::MolarVaporFraction,
                                &flash_specs,
                            );
                            molar_vapor_fraction.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("molar_vapor_fraction\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::MassVaporFraction,
                                &flash_specs,
                            );
                            mass_vapor_fraction.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("mass_vapor_fraction\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::MolarEnthalpy,
                                &flash_specs,
                            );
                            molar_enthalpy.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("molar_enthalpy\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::SpecificEnthalpy,
                                &flash_specs,
                            );
                            specific_enthalpy.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("specific_enthalpy\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::MolarVolume,
                                &flash_specs,
                            );
                            molar_volume.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("molar_volume\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::SpecificVolume,
                                &flash_specs,
                            );
                            specific_volume.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("specific_volume\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::MolarDensity,
                                &flash_specs,
                            );
                            molar_density.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("molar_density\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::MassDensity,
                                &flash_specs,
                            );
                            mass_density.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("mass_density\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::MolarEntropy,
                                &flash_specs,
                            );
                            molar_entropy.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("molar_entropy\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                            let (value, is_field) = material_stream.get_specification_value(
                                &pp,
                                StreamSpecificationType::SpecificEntropy,
                                &flash_specs,
                            );
                            specific_entropy.add_field(
                                number_formatter::format_number(value),
                                if is_field {
                                    Some(format!("specific_entropy\t{}", stream.name))
                                } else {
                                    None
                                },
                            );
                        } else {
                            //product stream
                            let (
                                mole_fractions,
                                mass_fractions,
                                molar_component_flow_rates,
                                mass_component_flow_rates,
                                total_molar_flow_rate,
                                total_mass_flow_rate,
                            ) = material_stream.get_overall_compound_values(&pp);
                            total_molar_flow
                                .add_result(number_formatter::format_number(total_molar_flow_rate));
                            total_mass_flow
                                .add_result(number_formatter::format_number(total_mass_flow_rate));
                            for (index, value) in mole_fractions.into_iter().enumerate() {
                                mole_fraction[index]
                                    .add_result(number_formatter::format_number(value));
                            }
                            for (index, value) in mass_fractions.into_iter().enumerate() {
                                mass_fraction[index]
                                    .add_result(number_formatter::format_number(value));
                            }
                            for (index, value) in molar_component_flow_rates.into_iter().enumerate()
                            {
                                mole_flow[index].add_result(number_formatter::format_number(value));
                            }
                            for (index, value) in mass_component_flow_rates.into_iter().enumerate()
                            {
                                mass_flow[index].add_result(number_formatter::format_number(value));
                            }
                            temperature.add_result(number_formatter::format_number(
                                material_stream
                                    .get_overall_value(&pp, StreamSpecificationType::Temperature),
                            ));
                            pressure.add_result(number_formatter::format_number(
                                material_stream
                                    .get_overall_value(&pp, StreamSpecificationType::Pressure),
                            ));
                            molar_vapor_fraction.add_result(number_formatter::format_number(
                                material_stream.get_overall_value(
                                    &pp,
                                    StreamSpecificationType::MolarVaporFraction,
                                ),
                            ));
                            mass_vapor_fraction.add_result(number_formatter::format_number(
                                material_stream.get_overall_value(
                                    &pp,
                                    StreamSpecificationType::MassVaporFraction,
                                ),
                            ));
                            molar_enthalpy.add_result(number_formatter::format_number(
                                material_stream
                                    .get_overall_value(&pp, StreamSpecificationType::MolarEnthalpy),
                            ));
                            specific_enthalpy.add_result(number_formatter::format_number(
                                material_stream.get_overall_value(
                                    &pp,
                                    StreamSpecificationType::SpecificEnthalpy,
                                ),
                            ));
                            molar_volume.add_result(number_formatter::format_number(
                                material_stream
                                    .get_overall_value(&pp, StreamSpecificationType::MolarVolume),
                            ));
                            specific_volume.add_result(number_formatter::format_number(
                                material_stream.get_overall_value(
                                    &pp,
                                    StreamSpecificationType::SpecificVolume,
                                ),
                            ));
                            molar_density.add_result(number_formatter::format_number(
                                material_stream
                                    .get_overall_value(&pp, StreamSpecificationType::MolarDensity),
                            ));
                            mass_density.add_result(number_formatter::format_number(
                                material_stream
                                    .get_overall_value(&pp, StreamSpecificationType::MassDensity),
                            ));
                            molar_entropy.add_result(number_formatter::format_number(
                                material_stream
                                    .get_overall_value(&pp, StreamSpecificationType::MolarEntropy),
                            ));
                            specific_entropy.add_result(number_formatter::format_number(
                                material_stream.get_overall_value(
                                    &pp,
                                    StreamSpecificationType::SpecificEntropy,
                                ),
                            ));
                        }
                    }
                    _ => {
                        //add empty entries
                        total_molar_flow.add_empty();
                        total_mass_flow.add_empty();
                        for i in 0..pp.compound_names.len() {
                            mole_fraction[i].add_empty();
                            mass_fraction[i].add_empty();
                            mole_flow[i].add_empty();
                            mass_flow[i].add_empty();
                        }
                        temperature.add_empty();
                        pressure.add_empty();
                        molar_vapor_fraction.add_empty();
                        mass_vapor_fraction.add_empty();
                        molar_enthalpy.add_empty();
                        specific_enthalpy.add_empty();
                        molar_volume.add_empty();
                        specific_volume.add_empty();
                        molar_density.add_empty();
                        mass_density.add_empty();
                        molar_entropy.add_empty();
                        specific_entropy.add_empty();
                    }
                }
            }
            //add rows to table
            rows.push(overall_header);
            total_molar_flow.add_cell("mol/s");
            rows.push(total_molar_flow);
            total_mass_flow.add_cell("kg/s");
            rows.push(total_mass_flow);
            if !single_compound {
                rows.push(mole_fraction_header);
                for mut row in mole_fraction {
                    row.add_cell("mol/mol");
                    rows.push(row);
                }
                rows.push(mass_fraction_header);
                for mut row in mass_fraction {
                    row.add_cell("kg/kg");
                    rows.push(row);
                }
                rows.push(mole_flow_rate_header);
                for mut row in mole_flow {
                    row.add_cell("mol/s");
                    rows.push(row);
                }
                rows.push(mass_flow_rate_header);
                for mut row in mass_flow {
                    row.add_cell("kg/s");
                    rows.push(row);
                }
            }
            if pp
                .flash_conditions
                .contains(&StreamSpecificationType::Temperature)
            {
                temperature.add_cell("K");
                rows.push(temperature);
            }
            if pp
                .flash_conditions
                .contains(&StreamSpecificationType::Pressure)
            {
                pressure.add_cell("Pa");
                rows.push(pressure);
            }
            if pp
                .flash_conditions
                .contains(&StreamSpecificationType::MolarVaporFraction)
            {
                molar_vapor_fraction.add_cell("mol/mol");
                rows.push(molar_vapor_fraction);
                mass_vapor_fraction.add_cell("kg/kg");
                rows.push(mass_vapor_fraction);
            }
            if pp
                .flash_conditions
                .contains(&StreamSpecificationType::MolarEnthalpy)
            {
                molar_enthalpy.add_cell("J/mol");
                rows.push(molar_enthalpy);
                specific_enthalpy.add_cell("J/kg");
                rows.push(specific_enthalpy);
            }
            if pp
                .flash_conditions
                .contains(&StreamSpecificationType::MolarVolume)
            {
                molar_volume.add_cell("m<sup>3</sup>/mol");
                rows.push(molar_volume);
                specific_volume.add_cell("m<sup>3</sup>/kg");
                rows.push(specific_volume);
            }
            if pp
                .flash_conditions
                .contains(&StreamSpecificationType::MolarDensity)
            {
                molar_density.add_cell("mol/m<sup>3</sup>");
                rows.push(molar_density);
                mass_density.add_cell("kg/m<sup>3</sup>");
                rows.push(mass_density);
            }
            if pp
                .flash_conditions
                .contains(&StreamSpecificationType::MolarEntropy)
            {
                molar_entropy.add_cell("J/(mol·K)");
                rows.push(molar_entropy);
                specific_entropy.add_cell("J/(kg·K)");
                rows.push(specific_entropy);
            }
            //any present phases
            for (index, present) in phase_presence.into_iter().enumerate() {
                if present {
                    let phase_name = pp.phase_labels[index].as_string();
                    let phase_header =
                        TableRow::new_header(format!("phase_{}", phase_name), &phase_name, None);
                    //make the overall headers
                    let mut molar_phase_fraction = TableRow::data_row(
                        format!("molar_phase_fraction_{}", phase_name),
                        Some(&phase_header),
                    );
                    molar_phase_fraction.add_cell("Phase Fraction");
                    let mut mass_phase_fraction = TableRow::data_row(
                        format!("mass_phase_fraction_{}", phase_name),
                        Some(&phase_header),
                    );
                    mass_phase_fraction.add_cell("Phase Fraction");
                    let mole_fraction_header = TableRow::new_header(
                        format!("x_{}", phase_name),
                        "Mole Fraction",
                        Some(&phase_header),
                    );
                    let mut mole_fraction = Vec::new();
                    for compound in pp.compound_names.iter() {
                        let id = format!("x_{}_{}", phase_name, compound);
                        let mut row = TableRow::data_row(id, Some(&mole_fraction_header));
                        row.add_cell(compound);
                        mole_fraction.push(row);
                    }
                    let mass_fraction_header = TableRow::new_header(
                        format!("w_{}", phase_name),
                        "Mass Fraction",
                        Some(&phase_header),
                    );
                    let mut mass_fraction = Vec::new();
                    for compound in pp.compound_names.iter() {
                        let id = format!("w_{}_{}", phase_name, compound);
                        let mut row = TableRow::data_row(id, Some(&mass_fraction_header));
                        row.add_cell(compound);
                        mass_fraction.push(row);
                    }
                    let mut molar_enthalpy = TableRow::data_row(
                        format!("molar_enthalpy_{}", phase_name),
                        Some(&phase_header),
                    );
                    molar_enthalpy.add_cell("Enthalpy");
                    let mut specific_enthalpy = TableRow::data_row(
                        format!("specific_enthalpy_{}", phase_name),
                        Some(&phase_header),
                    );
                    specific_enthalpy.add_cell("Enthalpy");
                    let mut molar_volume = TableRow::data_row(
                        format!("molar_volume_{}", phase_name),
                        Some(&phase_header),
                    );
                    molar_volume.add_cell("Volume");
                    let mut specific_volume = TableRow::data_row(
                        format!("specific_volume_{}", phase_name),
                        Some(&phase_header),
                    );
                    specific_volume.add_cell("Volume");
                    let mut molar_density = TableRow::data_row(
                        format!("molar_density_{}", phase_name),
                        Some(&phase_header),
                    );
                    molar_density.add_cell("Density");
                    let mut mass_density = TableRow::data_row(
                        format!("mass_density_{}", phase_name),
                        Some(&phase_header),
                    );
                    mass_density.add_cell("Density");
                    let mut molar_entropy = TableRow::data_row(
                        format!("molar_entropy_{}", phase_name),
                        Some(&phase_header),
                    );
                    molar_entropy.add_cell("Entropy");
                    let mut specific_entropy = TableRow::data_row(
                        format!("specific_entropy_{}", phase_name),
                        Some(&phase_header),
                    );
                    specific_entropy.add_cell("Entropy");
                    let fugacity_header = TableRow::new_header(
                        format!("fug_{}", phase_name),
                        "Fugacity",
                        Some(&phase_header),
                    );
                    let mut fugacity = Vec::new();
                    for compound in pp.compound_names.iter() {
                        let id = format!("fug_{}_{}", phase_name, compound);
                        let mut row = TableRow::data_row(id, Some(&fugacity_header));
                        row.add_cell(compound);
                        fugacity.push(row);
                    }
                    for stream in self.streams.iter() {
                        let mut add_empty = true;
                        if let StreamType::Material(obj) = &stream.stream_type {
                            let material_stream = unsafe { MaterialStream::borrow(obj) }; //note that unchecked borrow is used here, as we are sure that obj points to a MaterialStream
                            if material_stream.phase_is_present(index) {
                                add_empty = false;
                                let (
                                    molar_phase_frac_value,
                                    mass_phase_frac_value,
                                    molar_composition_value,
                                    mass_composition_value,
                                    molar_enthalpy_value,
                                    specific_enthalpy_value,
                                    molar_entropy_value,
                                    specific_entropy_value,
                                    molar_volume_value,
                                    specific_volume_value,
                                    molar_density_value,
                                    mass_density_value,
                                    fugacity_value,
                                ) = material_stream.get_phase_values(&pp, index);
                                molar_phase_fraction.add_result(
                                    number_formatter::format_number(molar_phase_frac_value),
                                );
                                mass_phase_fraction.add_result(
                                    number_formatter::format_number(mass_phase_frac_value),
                                );
                                match molar_composition_value {
                                    Some(values) => {
                                        for (idx, value) in values.into_iter().enumerate() {
                                            mole_fraction[idx].add_result(
                                                number_formatter::format_number(Some(value)),
                                            );
                                        }
                                    }
                                    None => {
                                        for mf in mole_fraction.iter_mut() {
                                            mf.add_empty();
                                        }
                                    }
                                };
                                match mass_composition_value {
                                    Some(values) => {
                                        for (idx, value) in values.into_iter().enumerate() {
                                            mass_fraction[idx].add_result(
                                                number_formatter::format_number(Some(value)),
                                            );
                                        }
                                    }
                                    None => {
                                        for mf in mass_fraction.iter_mut() {
                                            mf.add_empty();
                                        }
                                    }
                                };
                                molar_enthalpy.add_result(number_formatter::format_number(
                                    molar_enthalpy_value,
                                ));
                                specific_enthalpy.add_result(number_formatter::format_number(
                                    specific_enthalpy_value,
                                ));
                                molar_volume.add_result(number_formatter::format_number(
                                    molar_volume_value,
                                ));
                                specific_volume.add_result(number_formatter::format_number(
                                    specific_volume_value,
                                ));
                                molar_density.add_result(number_formatter::format_number(
                                    molar_density_value,
                                ));
                                mass_density.add_result(number_formatter::format_number(
                                    mass_density_value,
                                ));
                                molar_entropy.add_result(number_formatter::format_number(
                                    molar_entropy_value,
                                ));
                                specific_entropy.add_result(number_formatter::format_number(
                                    specific_entropy_value,
                                ));
                                match fugacity_value {
                                    Some(values) => {
                                        for (idx, value) in values.into_iter().enumerate() {
                                            fugacity[idx].add_result(
                                                number_formatter::format_number(Some(value)),
                                            );
                                        }
                                    }
                                    None => {
                                        for f in fugacity.iter_mut() {
                                            f.add_empty();
                                        }
                                    }
                                };
                            }
                        }
                        if add_empty {
                            //add empty entries
                            molar_phase_fraction.add_empty();
                            mass_phase_fraction.add_empty();
                            for i in 0..pp.compound_names.len() {
                                mole_fraction[i].add_empty();
                                mass_fraction[i].add_empty();
                                fugacity[i].add_empty();
                            }
                            molar_enthalpy.add_empty();
                            specific_enthalpy.add_empty();
                            molar_volume.add_empty();
                            specific_volume.add_empty();
                            molar_density.add_empty();
                            mass_density.add_empty();
                            molar_entropy.add_empty();
                            specific_entropy.add_empty();
                        }
                    }
                    //add rows to table
                    rows.push(phase_header);
                    molar_phase_fraction.add_cell("mol/mol");
                    rows.push(molar_phase_fraction);
                    mass_phase_fraction.add_cell("kg/kg");
                    rows.push(mass_phase_fraction);
                    if !single_compound {
                        rows.push(mole_fraction_header);
                        for mut row in mole_fraction {
                            row.add_cell("mol/mol");
                            rows.push(row);
                        }
                        rows.push(mass_fraction_header);
                        for mut row in mass_fraction {
                            row.add_cell("kg/kg");
                            rows.push(row);
                        }
                    }
                    if pp
                        .flash_conditions
                        .contains(&StreamSpecificationType::MolarEnthalpy)
                    {
                        molar_enthalpy.add_cell("J/mol");
                        rows.push(molar_enthalpy);
                        specific_enthalpy.add_cell("J/kg");
                        rows.push(specific_enthalpy);
                    }
                    if pp
                        .flash_conditions
                        .contains(&StreamSpecificationType::MolarVolume)
                    {
                        molar_volume.add_cell("m<sup>3</sup>/mol");
                        rows.push(molar_volume);
                        specific_volume.add_cell("m<sup>3</sup>/kg");
                        rows.push(specific_volume);
                    }
                    if pp
                        .flash_conditions
                        .contains(&StreamSpecificationType::MolarDensity)
                    {
                        molar_density.add_cell("mol/m<sup>3</sup>");
                        rows.push(molar_density);
                        mass_density.add_cell("kg/m<sup>3</sup>");
                        rows.push(mass_density);
                    }
                    if pp
                        .flash_conditions
                        .contains(&StreamSpecificationType::MolarEntropy)
                    {
                        molar_entropy.add_cell("J/(mol·K)");
                        rows.push(molar_entropy);
                        specific_entropy.add_cell("J/(kg·K)");
                        rows.push(specific_entropy);
                    }
                    if pp.supports_fugacity {
                        rows.push(fugacity_header);
                        for mut row in fugacity {
                            row.add_cell("Pa");
                            rows.push(row);
                        }
                    }
                }
            }
        }
        TableStatus { headers, rows }
    }

    /// Builds the unit-parameter table payload from ICapeUtilities parameters.
    ///
    /// # Returns
    /// A [`TableStatus`] representing scalar/array parameter values and edits.
    pub fn get_parameter_table(&self) -> TableStatus {
        match self.unit {
            Some(ref unit) => {
                unit.get_parameter_table(self.is_solved)
            }
            None => {
                //no unit, return empty table
                TableStatus {
                    headers: Vec::new(),
                    rows: Vec::new(),
                }
            }
        }
    }

    /// Applies a user edit to either a stream specification or a unit
    /// parameter value.
    ///
    /// # Parameters
    /// - `edit_id`: Encoded edit target identifier from the frontend table.
    /// - `value`: New textual value provided by the user.
    pub fn specify(&mut self, edit_id: &str, value: &str) {
        //clear solution
        if let Some(unit) = self.unit.as_mut() {
            unit.clear_error();
        }
        self.is_solved = false;
        for stream in self.streams.iter_mut() {
            if stream.feed() {
                continue;
            }
            stream.clear_values();
        }
        //split at first tab
        let index = edit_id.find('\t');
        let index = match index {
            Some(i) => i,
            None => {
                return;
            }
        };
        let (type_id, rest) = edit_id.split_at(index);
        let (spec, stream_id) = match type_id {
            "total_molar_flow" => (StreamSpecificationType::MolarFlowRate, rest),
            "total_mass_flow" => (StreamSpecificationType::MassFlowRate, rest),
            "mole_fraction" => {
                //split rest at second tab
                let index = rest[1..].find('\t');
                let index = match index {
                    Some(i) => i + 1,
                    None => {
                        return;
                    }
                };
                let (stream_id, compound) = rest.split_at(index);
                (
                    StreamSpecificationType::MolarFraction(compound[1..].to_string()),
                    stream_id,
                )
            }
            "mass_fraction" => {
                //split rest at second tab
                let index = rest[1..].find('\t');
                let index = match index {
                    Some(i) => i + 1,
                    None => {
                        return;
                    }
                };
                let (stream_id, compound) = rest.split_at(index);
                (
                    StreamSpecificationType::MassFraction(compound[1..].to_string()),
                    stream_id,
                )
            }
            "molar_flow_rate" => {
                //split rest at second tab
                let index = rest[1..].find('\t');
                let index = match index {
                    Some(i) => i + 1,
                    None => {
                        return;
                    }
                };
                let (stream_id, compound) = rest.split_at(index);
                (
                    StreamSpecificationType::MolarCompoundFlowRate(compound[1..].to_string()),
                    stream_id,
                )
            }
            "mass_flow_rate" => {
                //split rest at second tab
                let index = rest[1..].find('\t');
                let index = match index {
                    Some(i) => i + 1,
                    None => {
                        return;
                    }
                };
                let (stream_id, compound) = rest.split_at(index);
                (
                    StreamSpecificationType::MassCompoundFlowRate(compound[1..].to_string()),
                    stream_id,
                )
            }
            "temperature" => (StreamSpecificationType::Temperature, rest),
            "pressure" => (StreamSpecificationType::Pressure, rest),
            "molar_vapor_fraction" => (StreamSpecificationType::MolarVaporFraction, rest),
            "mass_vapor_fraction" => (StreamSpecificationType::MassVaporFraction, rest),
            "molar_enthalpy" => (StreamSpecificationType::MolarEnthalpy, rest),
            "specific_enthalpy" => (StreamSpecificationType::SpecificEnthalpy, rest),
            "molar_volume" => (StreamSpecificationType::MolarVolume, rest),
            "specific_volume" => (StreamSpecificationType::SpecificVolume, rest),
            "molar_density" => (StreamSpecificationType::MolarDensity, rest),
            "mass_density" => (StreamSpecificationType::MassDensity, rest),
            "molar_entropy" => (StreamSpecificationType::MolarEntropy, rest),
            "specific_entropy" => (StreamSpecificationType::SpecificEntropy, rest),
            "stream_parameter" => {
                //split rest at second tab
                let index = rest[1..].find('\t');
                let index = match index {
                    Some(i) => i + 1,
                    None => {
                        return;
                    }
                };
                let (name, stream) = rest.split_at(index);
                (
                    StreamSpecificationType::ParameterValue(name[1..].to_string()),
                    stream,
                )
            }
            "unit_parameter" => {
                if let Some(ref mut unit) = self.unit {
                    unit.set_parameter_value(&rest[1..], value, &mut self.status_notifications);
                }
                //not a stream update
                return;
            }
            _ => {
                //unknown spec
                return;
            }
        };
        //get stream
        let stream_id = stream_id[1..].to_string();
        let stream: &mut Stream = match self.streams.iter_mut().find(|s| s.name == stream_id) {
            Some(s) => s,
            None => {
                return;
            }
        };
        //update stream
        if let Some(ref pp) = self.property_package {
            pp.write().unwrap().clear_log();
        };
        if stream.feed() && stream.specify(spec, value) {
            if let StreamType::Material(ref mut obj) = stream.stream_type {
                let material_stream = unsafe { MaterialStream::borrow_mut(obj) };
                if let Some(ref pp) = self.property_package {
                    material_stream.add_log(pp.write().unwrap().take_log());
                };
                self.status_notifications |= StatusNotification::LogChanged;
            };
            self.status_notifications |=
                StatusNotification::StreamsChanged | StatusNotification::StatusChanged;
        }
    }

    /// Runs unit calculation and updates product-streams
    pub fn calculate_unit(&mut self) {
        self.is_solved = false;
        //get unit operation
        let unit = match &mut self.unit {
            Some(u) => u,
            None => {
                return;
            }
        };
        self.status_notifications |= StatusNotification::StreamsChanged
            | StatusNotification::StatusChanged
            | StatusNotification::ParametersChanged
            | StatusNotification::ReportsChanged
            | StatusNotification::LogChanged;
        if unit.calculate() {
            //mark the product streams as complete and calculate properties on product streams
            for stream in self.streams.iter_mut() {
                if stream.feed() {
                    continue;
                }
                if let StreamType::Material(ref mut obj) = stream.stream_type {
                    let material_stream = unsafe { MaterialStream::borrow_mut(obj) };
                    self.property_package
                        .as_ref()
                        .unwrap()
                        .write()
                        .unwrap()
                        .clear_log();
                    material_stream.mark_as_complete();
                    if material_stream.ensure_context_material().is_ok() {
                        material_stream.calc_properties_after_flash();
                    };
                    material_stream.add_log(
                        self.property_package
                            .as_ref()
                            .unwrap()
                            .write()
                            .unwrap()
                            .take_log(),
                    );
                };
            }
        }
        self.is_solved = true;
    }

    /// Clears all runtime state and releases COM references.
    ///
    /// Intended for internal teardown paths rather than regular UI resets.
    pub fn clear(&mut self) {
        //this is to drop references prior to exit; this is not called by the client
        self.streams.clear(); //holds property package references
        if let Some(pp) = self.property_package.take() {
            pp.write().unwrap().unset_context_material();
        }
        self.terminate_unit();
        self.unit_provider = None;
        self.unit_creation_error = None;
        self.unit = None;
        self.is_solved = false;
        self.thermo_provider = None;
        self.package_name.clear();
        self.thermo_creation_error = None;
        self.property_package = None;
        self.streams.clear();
        self.streams_error = None;
        self.path = None;
    }

    /// Returns available report descriptors in `<mime>\t<name>` format.
    ///
    /// # Returns
    /// List of report descriptors supported by the current unit operation.
    pub fn get_report_list(&self) -> Vec<String> {
        match self.unit {
            Some(ref unit) => unit.get_report_list(),
            None => Vec::new(),
        }
    }

    /// Returns encoded report payload as `<mime>\t<content>`.
    ///
    /// Text reports are returned as plain UTF-8 text (except HTML), while
    /// non-text payloads are Base64-encoded.
    ///
    /// # Parameters
    /// - `report`: Combined report descriptor `<mime>\t<name>`.
    /// - `image_size`: Desired image resolution `(width, height)` for image reports.
    ///
    /// # Returns
    /// Formatted response string containing MIME type and payload/error text.
    pub fn get_report_content(&self, report: &str, image_size: (i32, i32)) -> String {
        //separate mime type and report
        let index = report.find('\t');
        let index = match index {
            Some(i) => i,
            None => {
                return format!(
                    "{}\t{}",
                    "text/plain", "Internal error: invalid report specification format"
                );
            }
        };
        let (mime_type, name) = report.split_at(index);
        let name = &name[1..];
        let is_text = mime_type.starts_with("text/") && mime_type != "text/html"; //html gets encoded for iframe
        match self.unit {
            Some(ref unit) => {
                match unit.get_report_content(mime_type, name, image_size) {
                    Ok(content) => {
                        if is_text {
                            //unencoded
                            format!(
                                "{}\t{}",
                                mime_type,
                                match String::from_utf8(content) {
                                    Ok(s) => s,
                                    Err(_) => "Report content is not valid UTF-8".to_string(),
                                }
                            )
                        } else {
                            //base64 encode binary content
                            format!("{}\t{}", mime_type, BASE64_STANDARD.encode(content))
                        }
                    }
                    Err(e) => {
                        format!("{}\t{}", "text/plain", e)
                    }
                }
            }
            None => {
                format!("{}\t{}", "text/plain", "No unit operation available")
            }
        }
    }

    /// Appends one log section to a table payload.
    ///
    /// # Parameters
    /// - `log`: Log entries to append.
    /// - `rows`: Destination table-row vector.
    /// - `header_prefix`: Prefix used to build unique row IDs.
    /// - `title`: Header title shown in the table.
    fn add_log_to_table(
        log: Option<&Vec<String>>,
        additional_info: Option<&String>,
        rows: &mut Vec<TableRow>,
        header_prefix: &str,
        title: &str,
    ) {
        let mut add_rows = Vec::new();
        let header = TableRow::header(title, None);
        if let Some(log) = log {
            for (i, entry) in log.iter().enumerate() {
                let mut row = TableRow::data_row(format!("{}_{}", header_prefix, i), Some(&header));
                row.add_result(entry.clone());
                add_rows.push(row);
            }
        }
        if let Some(info) = additional_info
            && !info.is_empty() {
                let mut row = TableRow::data_row(format!("{}_error", header_prefix), Some(&header));
                row.add_result(format!("Error details: {}", info));
                add_rows.push(row);
            }
        if !add_rows.is_empty() {
            rows.push(header.clone());
            rows.extend(add_rows);
        }
    }

    /// Builds the consolidated log table payload.
    ///
    /// # Returns
    /// A [`TableStatus`] containing unit, thermo, calculation, and stream logs.
    pub fn get_log_table(&self) -> TableStatus {
        let mut headers: Vec<String> = Vec::with_capacity(1);
        let mut rows: Vec<TableRow> = Vec::new();
        headers.push(String::new()); //empty
        let unit_creation_log = self.unit.as_ref().map(|unit| unit.get_creation_log());
        Self::add_log_to_table(
            unit_creation_log,
            self.unit_creation_error.as_ref(),
            &mut rows,
            "unit_creation",
            "Create Unit",
        );
        match self.property_package.as_ref() {
            Some(pp) => {
                Self::add_log_to_table(
                    Some(pp.read().unwrap().get_creation_log()),
                    self.thermo_creation_error.as_ref(),
                    &mut rows,
                    "thermo_creation",
                    "Create Thermo",
                );
            }
            None => {
                Self::add_log_to_table(
                    None,
                    self.thermo_creation_error.as_ref(),
                    &mut rows,
                    "thermo_creation",
                    "Create Thermo",
                );
            }
        };
        let unit_calculation_log = self.unit.as_ref().map(|unit| unit.get_calculation_log());
        Self::add_log_to_table(
            unit_calculation_log,
            None,
            &mut rows,
            "unit_calculation",
            "Unit Calculation",
        );
        for stream in self.streams.iter() {
            if let StreamType::Material(ref material_stream) = stream.stream_type {
                let material = unsafe { MaterialStream::borrow_mut(material_stream) };
                Self::add_log_to_table(
                    Some(material.get_log()),
                    None,
                    &mut rows,
                    &format!("stream_{}", stream.name),
                    &stream.name,
                );
            }
        }
        TableStatus { headers, rows }
    }

    /// Builds the static information table for selected unit/thermo providers
    /// and runtime metadata.
    ///
    /// # Returns
    /// A [`TableStatus`] containing provider and environment details.
    pub fn get_info_table(&self) -> TableStatus {
        let mut headers: Vec<String> = Vec::with_capacity(2);
        let mut rows: Vec<TableRow> = Vec::new();
        //headers (none)
        headers.push(String::new());
        headers.push(String::new());
        //unit info
        if let Some(provider) = self.unit_provider.as_ref() {
            let unit_header = TableRow::header("Unit Operation", None);
            rows.push(unit_header.clone());
            let name = match provider.get_name() {
                Ok(n) => n,
                Err(_) => "N/A".to_string(),
            };
            let mut row = TableRow::data_row("unit_name".to_string(), Some(&unit_header));
            row.add_cell("name");
            row.add_result(name);
            rows.push(row);
            let description = provider.get_description().unwrap_or_default();
            let mut row =
                TableRow::data_row("unit_description".to_string(), Some(&unit_header));
            row.add_cell("description");
            row.add_result(description);
            rows.push(row);
            let version = match provider.get_component_version() {
                Ok(v) => v,
                Err(_) => "N/A".to_string(),
            };
            let mut row = TableRow::data_row("unit_version".to_string(), Some(&unit_header));
            row.add_cell("version");
            row.add_result(version);
            rows.push(row);
            if let Ok(url) = provider.get_vendor_url() {
                let mut row =
                    TableRow::data_row("unit_vendor_url".to_string(), Some(&unit_header));
                row.add_cell("vendor URL");
                row.add_result(url);
                rows.push(row);
            };
            if let Ok(url) = provider.get_help_url() {
                let mut row =
                    TableRow::data_row("unit_help_url".to_string(), Some(&unit_header));
                row.add_cell("help URL");
                row.add_result(url);
                rows.push(row);
            }
            if let Ok(about) = provider.get_about() {
                let mut row =
                    TableRow::data_row("unit_about".to_string(), Some(&unit_header));
                row.add_cell("about");
                row.add_result(about);
                rows.push(row);
            }
            if let Ok(prog_id) = provider.get_prog_id() {
                let mut row =
                    TableRow::data_row("unit_prog_id".to_string(), Some(&unit_header));
                row.add_cell("ProgID");
                row.add_result(prog_id);
                rows.push(row);
            }
            if let Ok(prog_id) = provider.get_version_independent_prog_id() {
                let mut row =
                    TableRow::data_row("unit_vi_prog_id".to_string(), Some(&unit_header));
                row.add_cell("version-independent ProgID");
                row.add_result(prog_id);
                rows.push(row);
            }
            if let Ok(uuid) = provider.get_uuid() {
                let mut row =
                    TableRow::data_row("unit_uuid".to_string(), Some(&unit_header));
                row.add_cell("UUID");
                row.add_result(uuid.to_string());
                rows.push(row);
            }
        };
        //unit info
        if let Some(provider) = self.thermo_provider.as_ref() {
            let thermo_header = TableRow::header("Thermo", None);
            rows.push(thermo_header.clone());
            let name = match provider.get_name() {
                Ok(n) => n,
                Err(_) => "N/A".to_string(),
            };
            let mut row = TableRow::data_row("thermo_name".to_string(), Some(&thermo_header));
            row.add_cell("name");
            row.add_result(name);
            rows.push(row);
            let description = provider.get_description().unwrap_or_default();
            if !self.package_name.is_empty() {
                let mut row =
                    TableRow::data_row("thermo_package".to_string(), Some(&thermo_header));
                row.add_cell("package");
                row.add_result(self.package_name.clone());
                rows.push(row);
            }
            let mut row =
                TableRow::data_row("thermo_description".to_string(), Some(&thermo_header));
            row.add_cell("description");
            row.add_result(description);
            rows.push(row);
            let version = match provider.get_component_version() {
                Ok(v) => v,
                Err(_) => "N/A".to_string(),
            };
            let mut row =
                TableRow::data_row("thermo_version".to_string(), Some(&thermo_header));
            row.add_cell("version");
            row.add_result(version);
            rows.push(row);
            if let Ok(url) = provider.get_vendor_url() {
                let mut row = TableRow::data_row(
                    "thermo_vendor_url".to_string(),
                    Some(&thermo_header),
                );
                row.add_cell("vendor URL");
                row.add_result(url);
                rows.push(row);
            };
            if let Ok(url) = provider.get_help_url() {
                let mut row =
                    TableRow::data_row("thermo_help_url".to_string(), Some(&thermo_header));
                row.add_cell("help URL");
                row.add_result(url);
                rows.push(row);
            }
            if let Ok(about) = provider.get_about() {
                let mut row =
                    TableRow::data_row("thermo_about".to_string(), Some(&thermo_header));
                row.add_cell("about");
                row.add_result(about);
                rows.push(row);
            }
            if let Ok(prog_id) = provider.get_prog_id() {
                let mut row =
                    TableRow::data_row("thermo_prog_id".to_string(), Some(&thermo_header));
                row.add_cell("ProgID");
                row.add_result(prog_id);
                rows.push(row);
            }
            if let Ok(prog_id) = provider.get_version_independent_prog_id() {
                let mut row = TableRow::data_row(
                    "thermo_vi_prog_id".to_string(),
                    Some(&thermo_header),
                );
                row.add_cell("version-independent ProgID");
                row.add_result(prog_id);
                rows.push(row);
            }
            if let Ok(uuid) = provider.get_uuid() {
                let mut row =
                    TableRow::data_row("thermo_uuid".to_string(), Some(&thermo_header));
                row.add_cell("UUID");
                row.add_result(uuid.to_string());
                rows.push(row);
            }
        };
        //COBIA details
        let cobia_header = TableRow::header("COBIA", None);
        rows.push(cobia_header.clone());
        let mut row = TableRow::data_row("cobia_version".to_string(), Some(&cobia_header));
        row.add_cell("version");
        row.add_result(cobia::get_cobia_version());
        rows.push(row);
        let mut row = TableRow::data_row("cobia_language".to_string(), Some(&cobia_header));
        row.add_cell("language");
        row.add_result(cobia::get_cobia_language());
        rows.push(row);
        //Unit Operation Runner details
        let uor_header = TableRow::header("Unit Operation Runner", None);
        rows.push(uor_header.clone());
        let mut row = TableRow::data_row("uor_version".to_string(), Some(&uor_header));
        row.add_cell("version");
        row.add_result(env!("CARGO_PKG_VERSION").to_string());
        rows.push(row);
        TableStatus { headers, rows }
    }

    /// Returns the last save/load path, if one is known.
    ///
    /// # Returns
    /// - `Some(PathBuf)` when a document path is tracked.
    /// - `None` when no document has been saved/loaded yet.
    pub fn get_path(&self) -> Option<PathBuf> {
        self.path.clone()
    }

    /// Saves the current runner configuration to an XML file.
    ///
    /// Persists selected unit/thermo providers, private persisted data, and
    /// feed-stream specifications.
    ///
    /// # Parameters
    /// - `path`: Destination file path.
    ///
    /// # Returns
    /// - `Ok(())` when serialization and file write succeed.
    /// - `Err(e)` on persistence or XML write failures.
    pub fn save(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        //save to file
        let file = std::fs::File::create(&path)?;
        let unit_private_data = match self.unit {
            Some(ref unit) => unit.save()?,
            None => None,
        };
        let thermo_private_data = if self.thermo_provider.is_some()
            && let Some(ref pp) = self.property_package {
            let pp = pp.read().unwrap();
            match cape_open_1_2::CapePersist::from_object(&pp.pp_compounds) {
                Ok(persist) => {
                    let writer = PersistWriter::root_1_2();
                    match persist.save(&writer, false as CapeBoolean) {
                        Ok(_) => Some(writer),
                        Err(e) => {
                            return Err(format!("Error saving thermo private data: {}", e).into());
                        }
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        };
        let mut writer = quick_xml::writer::Writer::new_with_indent(file, b' ', 4);
        let content_writer = writer.create_element("UnitRunner");
        content_writer.write_inner_content(|writer| {
            if let Some(provider) = self.unit_provider.as_ref() {
                match provider.get_uuid() {
                    Ok(uuid) => {
                        //write unit
                        let unit_writer=writer.create_element("UnitOperation").with_attribute(("uuid", uuid.to_string().as_str()));
                        if let Some(data) = unit_private_data {
                            unit_writer.write_inner_content(|w| {
                                w.create_element("PrivateData").write_inner_content(|writer| {
                                    match unsafe { PersistWriter::borrow(&data).write_xml(writer) } {
                                        Ok(_) => Ok(()),
                                        Err(e) => Err(std::io::Error::other(format!("Error writing unit private data to XML: {}",e)))
                                    }
                                })?;
                                Ok(())
                            })?;
                        };
                    },
                    Err(e) => {
                        return Err(std::io::Error::other(format!("Error getting UUID from unit provider: {}",e)));
                    }
                }
            };
            if let Some(provider) = self.thermo_provider.as_ref() {
                match provider.get_uuid() {
                    Ok(uuid) => {
                        //write thermo
                        let mut thermo_writer=writer.create_element("Thermo").with_attribute(("uuid", uuid.to_string().as_str()));
                        if !self.package_name.is_empty() {
                            thermo_writer=thermo_writer.with_attribute(("package_name", self.package_name.as_str()));
                        }
                        if let Some(data) = thermo_private_data {
                            thermo_writer.write_inner_content(|w| {
                                w.create_element("PrivateData").write_inner_content(|writer| {
                                    match unsafe { PersistWriter::borrow(&data).write_xml(writer) } {
                                        Ok(_) => Ok(()),
                                        Err(e) => Err(std::io::Error::other(format!("Error writing thermo private data to XML: {}",e)))
                                    }
                                })?;
                                Ok(())
                            })?;
                        };
                    },
                    Err(e) => {
                        return Err(std::io::Error::other(format!("Error getting UUID from thermo provider: {}",e)));
                    }
                }
            };
            let mut any_specs=false;
            for stream in self.streams.iter() {
                if stream.feed() && !stream.get_specifications().is_empty() {
                    any_specs=true;
                    break;
                }
            }
            if any_specs {
                writer.create_element("Specifications").write_inner_content(|spec_writer| {
                    for stream in self.streams.iter() {
                        if stream.feed() {
                            let specs=stream.get_specifications();
                            if !specs.is_empty() {
                                spec_writer.create_element("Stream").with_attribute(("name",stream.name.as_str())).write_inner_content(|stream_spec_writer|{
                                    for spec in specs.iter() {
                                        match spec.write_xml(stream_spec_writer) {
                                            Ok(_) => {},
                                            Err(e) => {
                                                return Err(std::io::Error::other(format!("Error writing stream specification to XML for stream {}: {}", stream.name, e)));
                                            }
                                        }
                                    }
                                    Ok(())
                                })?;
                            }
                        }
                    }
                    Ok(())
                })?;
            }
            //save results
            let mut error=None;
            let mut missing=None;   
            let mut write_solution= false;
            let results=writer.create_element("Results").with_attribute(("status", match self.get_status() {
                Status::Ready => "Ready",
                Status::Solved => {
                    write_solution=true;
                    "Solved"
                },
                Status::Error(e) => {
                    error=Some(e);
                    "Error"
                },
                Status::Incomplete(m) => {
                    missing=Some(m);
                    "Incomplete"
                },
            }));
            results.write_inner_content(|results_writer| {
                if let Some(e) = error {
                    results_writer.create_element("Error").write_text_content(quick_xml::events::BytesText::new(&e))?;
                };
                if let Some(m) = missing {
                    results_writer.create_element("Missing").write_text_content(quick_xml::events::BytesText::new(&m))?;
                };
                if write_solution {
                    //streams
                    let streams=results_writer.create_element("Streams");
                    streams.write_inner_content(|streams_writer| {
                        let pp = self.property_package.as_ref().unwrap().read().unwrap();
                        for stream in self.streams.iter() {
                            let stream_node=streams_writer.create_element("Stream").
                                with_attribute(("name", stream.name.as_str())).
                                with_attribute(("type", match stream.stream_type {
                                    StreamType::Material(_) => "Material",
                                    StreamType::Energy(_) => "Energy",
                                    StreamType::Information(_) => "Information",
                                })).
                                with_attribute(("direction", if stream.feed() {"in"} else {"out"}));
                            stream_node.write_inner_content(|stream_writer| {
                                match stream.stream_type {
                                    StreamType::Material(ref obj) => {
                                        let material_stream = unsafe { MaterialStream::borrow(obj) }; //note that unchecked borrow is used here, as we are sure that obj points to a MaterialStream
                                        let (
                                            mole_fractions,
                                            mass_fractions,
                                            _molar_component_flow_rates,
                                            _mass_component_flow_rates,
                                            total_molar_flow_rate,
                                            total_mass_flow_rate,
                                        ) = material_stream.get_overall_compound_values(&pp);
                                        if let Some(value)=total_molar_flow_rate {
                                            let value_node=stream_writer.create_element("MolarFlow").with_attribute(("unit", "mol/s"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                        }
                                        if let Some(value)=total_mass_flow_rate {
                                            let value_node=stream_writer.create_element("MassFlow").with_attribute(("unit", "kg/s"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                        }
                                        let mut any_mole_fraction=false;
                                        for value in mole_fractions.iter() {
                                            if value.is_some() {
                                                any_mole_fraction=true;
                                                break;
                                            }
                                        }
                                        if any_mole_fraction {
                                            let value=stream_writer.create_element("MoleFraction").with_attribute(("unit", "mol/mol"));
                                            value.write_inner_content(|value_writer| {
                                                    for (index, value) in mole_fractions.into_iter().enumerate() {
                                                    if let Some(value)=value {
                                                        let sub_value=value_writer.create_element("Component").with_attribute(("name", pp.compound_names[index].as_str()));
                                                        sub_value.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                    }
                                                }
                                                Ok(())
                                            })?;
                                        }
                                        let mut any_mass_fraction=false;
                                        for value in mass_fractions.iter() {
                                            if value.is_some() {
                                                any_mass_fraction=true;
                                                break;
                                            }
                                        }
                                        if any_mass_fraction {
                                            let value=stream_writer.create_element("MassFraction").with_attribute(("unit", "kg/kg"));
                                            value.write_inner_content(|value_writer| {
                                                    for (index, value) in mass_fractions.into_iter().enumerate() {
                                                    if let Some(value)=value {
                                                        let sub_value=value_writer.create_element("Component").with_attribute(("name", pp.compound_names[index].as_str()));
                                                        sub_value.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                    }
                                                }
                                                Ok(())
                                            })?;
                                        }
                                        if let Some(temperature) = material_stream.get_overall_value(&pp, StreamSpecificationType::Temperature) {
                                            let value_node=stream_writer.create_element("Temperature").with_attribute(("unit", "K"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&temperature.to_string()))?;
                                        }
                                        if let Some(pressure) = material_stream.get_overall_value(&pp, StreamSpecificationType::Pressure) {
                                            let value_node=stream_writer.create_element("Pressure").with_attribute(("unit", "Pa"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&pressure.to_string()))?;
                                        }
                                        if let Some(molar_vapor_fraction) = material_stream.get_overall_value(&pp, StreamSpecificationType::MolarVaporFraction) {
                                            let value_node=stream_writer.create_element("MolarVaporFraction").with_attribute(("unit", "mol/mol"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&molar_vapor_fraction.to_string()))?;
                                        }
                                        if let Some(mass_vapor_fraction) = material_stream.get_overall_value(&pp, StreamSpecificationType::MassVaporFraction) {
                                            let value_node=stream_writer.create_element("MassVaporFraction").with_attribute(("unit", "kg/kg"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&mass_vapor_fraction.to_string()))?;
                                        }
                                        if let Some(molar_enthalpy) = material_stream.get_overall_value(&pp, StreamSpecificationType::MolarEnthalpy) {
                                            let value_node=stream_writer.create_element("MolarEnthalpy").with_attribute(("unit", "J/mol"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&molar_enthalpy.to_string()))?;
                                        }
                                        if let Some(specific_enthalpy) = material_stream.get_overall_value(&pp, StreamSpecificationType::SpecificEnthalpy) {
                                            let value_node=stream_writer.create_element("SpecificEnthalpy").with_attribute(("unit", "J/kg"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&specific_enthalpy.to_string()))?;
                                        }
                                        if let Some(molar_volume) = material_stream.get_overall_value(&pp, StreamSpecificationType::MolarVolume) {
                                            let value_node=stream_writer.create_element("MolarVolume").with_attribute(("unit", "m^3/mol"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&molar_volume.to_string()))?;
                                        }
                                        if let Some(specific_volume) = material_stream.get_overall_value(&pp, StreamSpecificationType::SpecificVolume) {
                                            let value_node=stream_writer.create_element("SpecificVolume").with_attribute(("unit", "m^3/kg"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&specific_volume.to_string()))?;
                                        }
                                        if let Some(molar_density) = material_stream.get_overall_value(&pp, StreamSpecificationType::MolarDensity) {
                                            let value_node=stream_writer.create_element("MolarDensity").with_attribute(("unit", "mol/m^3"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&molar_density.to_string()))?;
                                        }
                                        if let Some(mass_density) = material_stream.get_overall_value(&pp, StreamSpecificationType::MassDensity) {
                                            let value_node=stream_writer.create_element("MassDensity").with_attribute(("unit", "kg/m^3"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&mass_density.to_string()))?;
                                        }
                                        if let Some(molar_entropy) = material_stream.get_overall_value(&pp, StreamSpecificationType::MolarEntropy) {
                                            let value_node=stream_writer.create_element("MolarEntropy").with_attribute(("unit", "J/(mol*K)"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&molar_entropy.to_string()))?;
                                        }
                                        if let Some(specific_entropy) = material_stream.get_overall_value(&pp, StreamSpecificationType::SpecificEntropy) {
                                            let value_node=stream_writer.create_element("SpecificEntropy").with_attribute(("unit", "J/(kg*K)"));
                                            value_node.write_text_content(quick_xml::events::BytesText::new(&specific_entropy.to_string()))?;
                                        }
                                        //any present phases
                                        for index in material_stream.get_present_phase_indices().into_iter() {
                                            let phase_name = pp.phase_labels[index].as_string();
                                            let phase_node=stream_writer.create_element("Phase").
                                                with_attribute(("name", phase_name.as_str()));
                                            phase_node.write_inner_content(|phase_writer| {
                                                let (
                                                    molar_phase_fraction,
                                                    mass_phase_fraction,
                                                    molar_composition,
                                                    mass_composition,
                                                    molar_enthalpy,
                                                    specific_enthalpy,
                                                    molar_entropy,
                                                    specific_entropy,
                                                    molar_volume,
                                                    specific_volume,
                                                    molar_density,
                                                    mass_density,
                                                    fugacity,
                                                ) = material_stream.get_phase_values(&pp, index);
                                                if let Some(value) = molar_phase_fraction {
                                                    let value_node=phase_writer.create_element("MolarPhaseFraction").with_attribute(("unit", "mol/mol"));
                                                    value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                }
                                                if let Some(value) = mass_phase_fraction {
                                                    let value_node=phase_writer.create_element("MassPhaseFraction").with_attribute(("unit", "kg/kg"));
                                                    value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                }
                                                if let Some(molar_composition) = molar_composition {
                                                    let value_node=phase_writer.create_element("MolarComposition").with_attribute(("unit", "mol/mol"));
                                                    value_node.write_inner_content(|value_writer| {
                                                        for (index, value) in molar_composition.into_iter().enumerate() {
                                                            let sub_value=value_writer.create_element("Component").with_attribute(("name", pp.compound_names[index].as_str()));
                                                            sub_value.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                        }
                                                        Ok(())
                                                    })?;
                                                }
                                                if let Some(mass_composition) = mass_composition {
                                                    let value_node=phase_writer.create_element("MassComposition").with_attribute(("unit", "kg/kg"));
                                                    value_node.write_inner_content(|value_writer| {
                                                        for (index, value) in mass_composition.into_iter().enumerate() {
                                                            let sub_value=value_writer.create_element("Component").with_attribute(("name", pp.compound_names[index].as_str()));
                                                            sub_value.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                        }
                                                        Ok(())
                                                    })?;
                                                }
                                                if let Some(value) = molar_enthalpy {
                                                    let value_node=phase_writer.create_element("MolarEnthalpy").with_attribute(("unit", "J/mol"));
                                                    value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                }
                                                if let Some(value) = specific_enthalpy {
                                                    let value_node=phase_writer.create_element("SpecificEnthalpy").with_attribute(("unit", "J/kg"));
                                                    value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                }
                                                if let Some(value) = molar_volume {
                                                    let value_node=phase_writer.create_element("MolarVolume").with_attribute(("unit", "m^3/mol"));
                                                    value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                }
                                                if let Some(value) = specific_volume {
                                                    let value_node=phase_writer.create_element("SpecificVolume").with_attribute(("unit", "m^3/kg"));
                                                    value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                }
                                                if let Some(value) = molar_density {
                                                    let value_node=phase_writer.create_element("MolarDensity").with_attribute(("unit", "mol/m^3"));
                                                    value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                }
                                                if let Some(value) = mass_density {
                                                    let value_node=phase_writer.create_element("MassDensity").with_attribute(("unit", "kg/m^3"));
                                                    value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                }
                                                if let Some(value) = molar_entropy {
                                                    let value_node=phase_writer.create_element("MolarEntropy").with_attribute(("unit", "J/(mol*K)"));
                                                    value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                }
                                                if let Some(value) = specific_entropy {
                                                    let value_node=phase_writer.create_element("SpecificEntropy").with_attribute(("unit", "J/(kg*K)"));
                                                    value_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                }
                                                if let Some(fugacity) = fugacity {
                                                    let value_node=phase_writer.create_element("Fugacity").with_attribute(("unit", "Pa"));
                                                    value_node.write_inner_content(|value_writer| {
                                                        for (index, value) in fugacity.into_iter().enumerate() {
                                                            let sub_value=value_writer.create_element("Component").with_attribute(("name", pp.compound_names[index].as_str()));
                                                            sub_value.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                                        }
                                                        Ok(())
                                                    })?;
                                                }
                                                Ok(())
                                            })?;
                                        }
                                    },
                                    StreamType::Energy(ref obj)|StreamType::Information(ref obj) => {
                                        let information_stream = unsafe { InformationStream::borrow(obj) }; //note that unchecked borrow is used here, as we are sure that obj points to a MaterialStream
                                        for par in information_stream.iter() {
                                            let par = unsafe { RealParameter::borrow(par) };
                                            if f64::is_nan(par.value) {
                                                continue;
                                            }
                                            let value=stream_writer.create_element("Value").with_attribute(("name", par.name.as_ref())).
                                                with_attribute(("unit", par.unit_of_measure.as_ref()));
                                            value.write_text_content(quick_xml::events::BytesText::new(&par.value.to_string()))?;
                                        }
                                    }
                                }
                                Ok(())
                            })?;
                        }
                        Ok(())
                    })?;
                    //parameters
                    if let Some(ref unit) = self.unit {
                        match unit.write_parameter_results(results_writer) {
                            Ok(_) => {},
                            Err(e) => {
                                return Err(std::io::Error::other(format!("Error writing parameter results to XML: {}", e)));
                            }
                        }
                    }
                };
                //log
                let log_status=self.get_log_table();
                let mut current_log = None;
                let mut logs=Vec::new();
                for row in log_status.rows.into_iter() {
                    if row.is_header {
                        if let Some(current)=current_log {
                            logs.push(current);
                        }
                        current_log=Some((row.id.clone(), Vec::new()));
                    } else {
                        current_log.as_mut().unwrap().1.push(row.cells[0].value.clone());
                    }
                }
                if let Some(current)=current_log {
                    logs.push(current);
                }
                for (header, rows) in logs.into_iter() {
                    let log_node=results_writer.create_element("Log").with_attribute(("name",header.as_str()));
                    log_node.write_inner_content(|log_writer| {
                        for text in rows.into_iter() {
                            let entry_node=log_writer.create_element("Message");
                            entry_node.write_text_content(quick_xml::events::BytesText::new(&text))?;
                        }
                        Ok(())
                    })?;
                }
                Ok(())
            })?;
            Ok(())
        })?;
        self.path = Some(path);
        Ok(())
    }
}

impl Drop for UnitOperationRunnerData {
    /// Ensures resources are released when runner state is dropped.
    fn drop(&mut self) {
        //clean up thermo
        self.clear();
    }
}

impl UnitOperationRunnerData {
    /// Loads runner configuration from an XML file.
    ///
    /// Recreates unit and thermodynamics state, restores persisted private
    /// data, reconnects streams, and reapplies saved feed specifications.
    ///
    /// # Parameters
    /// - `path`: Source file path.
    ///
    /// # Returns
    /// - `Ok(())` when loading and reconstruction succeed.
    /// - `Err(e)` on XML parsing, provider lookup, persistence, or init failures.
    pub fn load(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        //load from file
        let mut root = match crate::xml_reader_node::XMLReader::root(path.clone()) {
            Ok(Some(r)) => r,
            Ok(None) => {
                return Err("XML root node not found".into());
            }
            Err(e) => {
                return Err(format!("Error loading XML: {}", e).into());
            }
        };
        let mut unit = None;
        let mut unit_provider = None;
        let mut thermo_provider = None;
        let mut package_name = String::new();
        let mut property_package = None;
        let mut stream_specifications = Vec::new();
        if root.get_name() != b"UnitRunner" {
            return Err(format!(
                "Invalid root node name: expected \"UnitRunner\", found \"{}\"",
                root.get_name()
                    .iter()
                    .map(|&b| b as char)
                    .collect::<String>()
            )
            .into());
        }
        loop {
            match root.next_node() {
                Ok(None) => break,
                Err(e) => {
                    return Err(format!("Error reading XML: {}", e).into());
                }
                Ok(Some(mut node)) => {
                    match node.get_name() {
                        b"UnitOperation" => {
                            //load unit operation
                            for (key, value) in node.get_attributes() {
                                match key.as_str() {
                                    "uuid" => {
                                        let uuid_str = cobia::CapeUUID::from_string(value)?;
                                        for provider in self.unit_pmcs.iter() {
                                            if let Ok(uuid) = provider.get_uuid()
                                                && uuid == uuid_str {
                                                    unit_provider = Some(provider.clone());
                                                    break;
                                                }
                                        }
                                        if unit_provider.is_none() {
                                            return Err(format!(
                                                "Unit provider with UUID {} not found",
                                                uuid_str
                                            )
                                            .into());
                                        }
                                    }
                                    attr_name => {
                                        return Err(format!(
                                            "Unknown attribute {} on UnitOperation node",
                                            attr_name
                                        )
                                        .into());
                                    }
                                }
                            }
                            //read sub nodes
                            let mut reader = None;
                            loop {
                                match node.next_node() {
                                    Ok(None) => break,
                                    Err(e) => {
                                        return Err(format!("Error reading XML: {}", e).into());
                                    }
                                    Ok(Some(sub_node)) => match sub_node.get_name() {
                                        b"PrivateData" => {
                                            reader = Some(PersistReader::from_xml(
                                                sub_node,
                                                "/".to_string(),
                                            )?);
                                        }
                                        node_name => {
                                            return Err(format!(
                                                "Unknown node {} under UnitOperation node",
                                                node_name
                                                    .iter()
                                                    .map(|&b| b as char)
                                                    .collect::<String>()
                                            )
                                            .into());
                                        }
                                    },
                                }
                            }
                            //create unit
                            unit = Some(Unit::new(unit_provider.as_ref().unwrap(), reader)?);
                        },
                        //load thermo
                        b"Thermo" => {
                            for (key, value) in node.get_attributes() {
                                match key.as_str() {
                                    "uuid" => {
                                        let uuid_str = cobia::CapeUUID::from_string(value)?;
                                        for provider in self.thermo_pmcs.iter() {
                                            if let Ok(uuid) = provider.get_uuid()
                                                && uuid == uuid_str {
                                                    thermo_provider = Some(provider.clone());
                                                    break;
                                                }
                                        }
                                        if thermo_provider.is_none() {
                                            return Err(format!(
                                                "Thermo provider with UUID {} not found",
                                                uuid_str
                                            )
                                            .into());
                                        }
                                    }
                                    "package_name" => {
                                        package_name = value.clone();
                                    }
                                    attr_name => {
                                        return Err(format!(
                                            "Unknown attribute {} on Thermo node",
                                            attr_name
                                        )
                                        .into());
                                    }
                                }
                            }
                            if thermo_provider.is_none() {
                                return Err("Thermo provider not specified".into());
                            }
                            //read sub nodes
                            let mut reader = None;
                            loop {
                                match node.next_node() {
                                    Ok(None) => break,
                                    Err(e) => {
                                        return Err(format!("Error reading XML: {}", e).into());
                                    }
                                    Ok(Some(sub_node)) => match sub_node.get_name() {
                                        b"PrivateData" => {
                                            reader = Some(PersistReader::from_xml(
                                                sub_node,
                                                "/".to_string(),
                                            )?);
                                        }
                                        node_name => {
                                            return Err(format!(
                                                "Unknown node {} under Thermo node",
                                                node_name
                                                    .iter()
                                                    .map(|&b| b as char)
                                                    .collect::<String>()
                                            )
                                            .into());
                                        }
                                    },
                                }
                            }
                            let tp = thermo_provider.as_ref().unwrap();
                            let pp = if tp.implements_cat_id(
                                &cape_open::CATEGORYID_STANDALONEPROPERTYPACKAGE,
                            )? {
                                //stand alone
                                PropertyPackageInstance::new_standalone(tp, reader, CapePMCCreationFlags::Default)?
                            } else {
                                //from manager
                                PropertyPackageInstance::new_from_manager(
                                    tp,
                                    &package_name,
                                    reader,
                                    CapePMCCreationFlags::Default
                                )?
                            };
                            property_package = Some(PropertyPackageInstanceRef::new(pp.into()));
                        },
                        b"Specifications" => {
                            //read stream specifications
                            loop {
                                match node.next_node() {
                                    Ok(None) => break,
                                    Err(e) => {
                                        return Err(format!("Error reading XML: {}", e).into());
                                    }
                                    Ok(Some(mut sub_node)) => {
                                        match sub_node.get_name() {
                                            b"Stream" => {
                                                let mut stream_name = None;
                                                for (key, value) in sub_node.get_attributes() {
                                                    match key.as_str() {
                                                        "name" => {
                                                            stream_name = Some(value.clone());
                                                        }
                                                        attr_name => {
                                                            return Err(format!("Unknown attribute {} on Stream node", attr_name).into());
                                                        }
                                                    }
                                                }
                                                let stream_name = match stream_name {
                                                    Some(n) => n,
                                                    None => {
                                                        return Err(
                                                            "Stream name attribute missing".into(),
                                                        );
                                                    }
                                                };
                                                //load specifications from sub_node
                                                let mut specs = Vec::new();
                                                loop {
                                                    match sub_node.next_node() {
                                                        Ok(None) => break,
                                                        Err(e) => {
                                                            return Err(format!(
                                                                "Error reading XML: {}",
                                                                e
                                                            )
                                                            .into());
                                                        }
                                                        Ok(Some(spec_node)) => {
                                                            specs.push(match StreamSpecification::read_xml(spec_node) {
                                                                Ok(s) => s,
                                                                Err(e) => {
                                                                    return Err(format!("Error reading stream specification from XML for stream {}: {}", stream_name, e).into());
                                                                }
                                                            });
                                                        }
                                                    }
                                                }
                                                stream_specifications.push((stream_name, specs));
                                            }
                                            node_name => {
                                                return Err(format!(
                                                    "Unknown node {} under Specifications node",
                                                    node_name
                                                        .iter()
                                                        .map(|&b| b as char)
                                                        .collect::<String>()
                                                )
                                                .into());
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        b"Results" => {
                            //ignore
                        },
                        //unknown nodes:
                        unkown_node_name => {
                            return Err(format!(
                                "Unknown node {} under root UnitRunner node",
                                unkown_node_name
                                    .iter()
                                    .map(|&b| b as char)
                                    .collect::<String>()
                            )
                            .into());
                        }
                    }
                }
            }
        }
        //clear
        self.clear();
        self.unit = unit;
        self.unit_provider = unit_provider;
        self.thermo_provider = thermo_provider;
        self.package_name = package_name;
        self.property_package = property_package;
        self.thermo_changed();
        for (name, specs) in stream_specifications.into_iter() {
            if let Some(s) = self.streams.iter_mut().find(|s| s.name == name)
                && s.feed() {
                    s.set_specifications(specs);
                };
        }
        self.status_notifications |= StatusNotification::UnitChanged
            | StatusNotification::StatusChanged
            | StatusNotification::ParametersChanged
            | StatusNotification::ReportsChanged
            | StatusNotification::InfoChanged
            | StatusNotification::LogChanged;
        self.path = Some(path);
        Ok(())
    }
}
