use crate::{
    number_formatter,
    persistence::PersistWriter,
    simulation_context::SimulationContext,
    status::Status,
    status_notification::StatusNotification,
    table_status::{TableRow, TableStatus},
};
use cobia::{prelude::*, *};
use std::cmp::min;

#[cfg(not(target_os = "windows"))]
use crate::persistence::PersistReader;

pub(crate) struct Unit {
    /// Unit operation interface
    unit: cape_open_1_2::CapeUnit,
    /// Utilities interface for the unit operation
    utilities: cape_open_1_2::CapeUtilities,
    /// Error captured during unit calculation.
    calculation_error: Option<String>,
    /// Log messages produced while calculating the unit operation.
    calculation_log: Vec<String>,
    /// Log messages produced while creating/configuring the unit operation.
    creation_log: Vec<String>,
    /// Shared simulation context object passed to CAPE-OPEN components.
    sim_context: CapeObject,
}

impl Unit {
    /// Creates a new `Unit` instance by instantiating the unit operation from the provider.
    ///
    /// # Parameters
    /// - `provider`: Details of the unit operation to instantiate, including the provider and type information.
    /// - `persisted_data_reader`: Optional object implementing CapePersistReader containing persisted data to load into the unit operation after creation, used for depersistence and isolated editing scenarios.
    ///
    /// # Returns
    /// - `Ok(Unit)` if the unit operation is successfully instantiated.
    /// - `Err(e)` if there is an error during instantiation, with `e` describing the failure.
    pub fn new(
        provider: &CapePMCRegistrationDetails,
        persisted_data_reader: Option<CapeObject>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (sim_context, unit, utilities, log) = Self::create_unit(
            provider,
            CapePMCCreationFlags::Default,
            persisted_data_reader,
        )?;
        Ok(Self {
            calculation_error: None,
            unit,
            utilities,
            calculation_log: log,
            creation_log: Vec::new(),
            sim_context,
        })
    }

    /// Creates a new `Unit` instance in an isolated process by instantiating the unit operation from the provider.
    ///
    /// # Parameters
    /// - `provider`: Details of the unit operation to instantiate, including the provider and type information.
    /// - `persisted_data_reader`: Optional object implementing CapePersistReader containing persisted data to load into the unit operation after creation, used for depersistence and isolated editing scenarios.
    ///
    /// # Returns
    /// - `Ok(Unit)` if the unit operation is successfully instantiated.
    /// - `Err(e)` if there is an error during instantiation, with `e` describing the failure.
    #[cfg(not(target_os = "windows"))]
    fn new_isolated(
        provider: &CapePMCRegistrationDetails,
        persisted_data_reader: Option<CapeObject>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (sim_context, unit, utilities, log) = Self::create_unit(
            provider,
            CapePMCCreationFlags::IsolatedProcess,
            persisted_data_reader,
        )?;
        Ok(Self {
            calculation_error: None,
            unit,
            utilities,
            calculation_log: log,
            creation_log: Vec::new(),
            sim_context,
        })
    }

    /// Creates and initializes a unit operation instance from the selected provider, applying persisted data if provided.
    ///
    /// The unit operation is created as part of unit opearation selection, depersistence, and in case of
    /// isolated editing. Order of operation is:
    ///
    /// - create
    /// - set simulation context
    /// - depersist (if data provided)
    /// - initialize
    ///
    /// # Parameters
    /// - `provider`: Optional provider details to create the unit operation from.
    /// - `flags`: Creation flags to control unit operation instantiation.
    /// - `persisted_data_reader`: Optional CAPE-OPEN object containing persisted unit operation data to load after creation.
    ///
    /// # Returns
    /// - `Ok((simulation_context,unit_operation, utilities, creation_log))` when creation and initialization succeed.
    /// - `Err(e)` when creation or initialization fails.
    fn create_unit(
        provider: &CapePMCRegistrationDetails,
        flags: CapePMCCreationFlags,
        persisted_data_reader: Option<CapeObject>,
    ) -> Result<
        (
            CapeObject,
            cape_open_1_2::CapeUnit,
            cape_open_1_2::CapeUtilities,
            Vec<String>,
        ),
        Box<dyn std::error::Error>,
    > {
        let sim_context: CapeObject = SimulationContext::create();
        unsafe { SimulationContext::borrow_mut(&sim_context) }.clear_messages();
        match provider.create_instance(flags | CapePMCCreationFlags::AllowRestrictedThreading) {
            Ok(u) => {
                let context = cape_open_1_2::CapeSimulationContext::from_object(&sim_context)
                    .expect("Unable to get ICapeSimulationContext from unit operation sim context");
                let unit = match cape_open_1_2::CapeUnit::from_object(&u) {
                    Ok(unit) => unit,
                    Err(e) => {
                        return Err(format!(
                            "Error getting ICapeUnit from unit operation \"{}\": {}",
                            provider.get_name()?,
                            e
                        )
                        .into());
                    }
                };
                let utils = match cape_open_1_2::CapeUtilities::from_object(&u) {
                    Ok(utils) => utils,
                    Err(e) => {
                        return Err(format!(
                            "Error getting ICapeUtilities from unit operation \"{}\": {}",
                            provider.get_name()?,
                            e
                        )
                        .into());
                    }
                };
                let _ = utils.set_simulation_context(&context);
                //load from persistence
                if let Some(reader) = persisted_data_reader {
                    let persist = match cape_open_1_2::CapePersist::from_object(&u) {
                        Ok(p) => p,
                        Err(e) => {
                            return Err(format!("Cannot depersist unit operation \"{}\"; cannot obtain IPersist: {}", provider.get_name()?, e).into());
                        }
                    };
                    let reader = cape_open_1_2::CapePersistReader::from_object(&reader).unwrap();
                    persist.load(&reader)?;
                };
                //initialize
                match utils.initialize() {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(format!(
                            "Error initializing unit operation \"{}\": {}",
                            provider.get_name()?,
                            e
                        )
                        .into());
                    }
                };
                let creation_log =
                    unsafe { SimulationContext::borrow_mut(&sim_context) }.take_messages();
                Ok((sim_context, unit, utils, creation_log))
            }
            Err(e) => {
                Err(format!(
                    "Error creating instance of unit operation \"{}\": {}",
                    provider.get_name()?,
                    e
                )
                .into())
            }
        }
    }

    /// Clears any existing calculation error error for the unit operation.
    pub fn clear_error(&mut self) {
        self.calculation_error = None;
    }

    /// Gets the log messages produced during unit operation creation and initialization.
    ///
    /// # Returns
    /// - `&Vec<String>` containing log messages from unit operation creation, loading and initialization.
    pub fn get_creation_log(&self) -> &Vec<String> {
        &self.creation_log
    }

    /// Gets the log messages produced during unit operation calculation.
    ///
    /// # Returns
    /// - `&Vec<String>` containing log messages from unit operation calculation.
    pub fn get_calculation_log(&self) -> &Vec<String> {
        &self.calculation_log
    }

    /// Opens the unit-operation edit dialog
    ///
    /// # Parameters
    /// - `parent`: Native window handle used as parent for modal UI.
    ///
    /// # Returns
    /// - `Ok(())` when edit handling succeeds.
    /// - `Err(e)` on persistence, initialization, or edit failures.
    #[cfg(target_os = "windows")]
    pub fn edit(
        &mut self,
        parent: CapeWindowId,
        _: &CapePMCRegistrationDetails,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let utils = match cape_open_1_2::CapeUtilities::from_object(&self.unit) {
            Ok(u) => u,
            Err(e) => {
                return Err(
                    format!("Error getting ICapeUtilities from unit operation: {}", e).into(),
                );
            }
        };
        match utils.edit(parent) {
            Ok(mode) => Ok(mode == cape_open_1_2::CapeEditResult::CapeModified),
            Err(e) => Err(format!("Error editing unit operation: {}", e).into()),
        }
    }

    /// Duplicates stream connections from one unit operation instance to another.
    ///
    /// Used to associate connected streams with a duplicate of a unit operation,
    /// when creating an isolated instance for editing on non-Windows platforms.
    ///
    /// # Parameters
    /// - `from`: Source unit operation whose ports and connections will be inspected.
    /// - `to`: Target unit operation to which matching ports will be connected to the same streams.
    ///
    /// # Returns
    /// - `Ok(())` when duplication completes (even if some ports fail to connect).
    #[cfg(not(target_os = "windows"))]
    fn duplicate_stream_connections(&self, from: &Unit) -> Result<(), Box<dyn std::error::Error>> {
        let ports_from = match from.ports() {
            Ok(ports) => ports,
            Err(_) => {
                //presumably a unit operation with no ports
                return Ok(());
            }
        };
        let ports_to = match self.ports() {
            Ok(ports) => ports,
            Err(e) => {
                //duplicate unit must have ports
                return Err(format!("Error getting ports from unit operation: {}", e).into());
            }
        };
        for port_from in ports_from.iter() {
            let connected_stream = match port_from.get_connected_object() {
                Ok(stream) => stream,
                Err(_) => {
                    //not connected, skip
                    continue;
                }
            };
            let iden = match cape_open_1_2::CapeIdentification::from_object(&port_from) {
                Ok(i) => i,
                Err(_) => {
                    //don't deal with port w/o name
                    continue;
                }
            };
            let mut port_name = CapeStringImpl::new();
            match iden.get_component_name(&mut port_name) {
                Ok(_) => {}
                Err(_) => {
                    //don't deal with port w/o name
                    continue;
                }
            };
            let port_to = match ports_to.item_by_name(&port_name) {
                Ok(p) => p,
                Err(_) => {
                    //no matching port, skip
                    continue;
                }
            };
            let _ = port_to.connect(&connected_stream);
        }
        Ok(())
    }

    /// Opens the unit-operation edit dialog using an isolated edit instance.
    ///
    /// On linux and other platforms, chances are that windowing toolkits
    /// PMCs use GTL-4 and as this process uses GTL-3, we cannot pop up the
    /// PMCs GUI inside this process. To work around this, we create an isolated
    /// instance of the unit operation, in an isolated process.
    ///
    /// The route is to persist the unit operation to be edited, create an isolated
    /// instance of the same unit operation, depersist into the isolated instance,
    /// and then call edit on the isolated instance. If the edit is successful,
    /// we persist the edited unit operation and depersist back into the main
    /// instance.
    ///
    /// # Parameters
    /// - `parent`: Native window handle used as parent for modal UI.
    ///
    /// # Returns
    /// - `Ok(())` when edit handling succeeds.
    /// - `Err(e)` on persistence, initialization, or edit failures.
    #[cfg(not(target_os = "windows"))]
    pub fn edit(
        &mut self,
        parent: CapeWindowId,
        provider: &CapePMCRegistrationDetails,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        //create an isolated unit operation for editing
        let private_data = PersistReader::reader_from_pmc_persistence(&self.utilities)?;
        let unit2 = Self::new_isolated(provider, private_data)?;
        unit2.duplicate_stream_connections(&self)?;
        let mode = match unit2.utilities.edit(parent) {
            Ok(mode) => mode,
            Err(e) => {
                return Err(format!("Error editing unit operation: {}", e).into());
            }
        };
        if mode == cape_open_1_2::CapeEditResult::CapeModified {
            //save the edited unit
            let private_data = PersistReader::reader_from_pmc_persistence(&unit2.utilities)?;
            //create a new instance
            let (sim_context, unit, utilities, log) =
                Self::create_unit(provider, CapePMCCreationFlags::Default, private_data)?;
            let _ = self.utilities.terminate();
            self.unit = unit;
            self.utilities = utilities;
            self.sim_context = sim_context;
            self.creation_log = log;
            let _ = self.duplicate_stream_connections(&unit2);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Retrieves the port collection of the unit operation.
    ///
    /// # Returns
    /// - `Ok(ports)` containing the port collection if retrieval is successful.
    /// - `Err(e)` if there is an error retrieving the ports, with `e` describing the failure.
    pub fn ports(
        &self,
    ) -> Result<
        cape_open_1_2::CapeCollection<cape_open_1_2::CapeUnitPort>,
        Box<dyn std::error::Error>,
    > {
        match self.unit.ports() {
            Ok(ports) => Ok(ports),
            Err(e) => {
                Err(format!("Error getting ports from unit operation: {}", e).into())
            }
        }
    }

    /// Get the unit operation's calculation error message, if any.
    ///
    /// # Returns
    /// - `Some(error_message)` if there is a calculation error message available.
    pub fn calculation_error(&self) -> Option<&String> {
        self.calculation_error.as_ref()
    }

    /// Gets the unit operation's status to report, if any
    ///
    /// # Returns
    /// - `Some(Status)` if there is a status to report (e.g. error or incomplete specifications)
    pub fn status(&self) -> Option<Status> {
        let mut msg = CapeStringImpl::new();
        match self.unit.validate(&mut msg) {
            Ok(is_valid) => {
                if is_valid == 0 {
                    Some(Status::Error(format!("Unit operation is invalid: {}", msg)))
                } else {
                    None
                }
            }
            Err(e) => Some(Status::Incomplete(format!("Validation failed: {}", e))),
        }
    }

    /// Builds the unit-parameter table payload from ICapeUtilities parameters.
    ///
    /// # Returns
    /// A [`TableStatus`] representing scalar/array parameter values and edits.
    pub fn get_parameter_table(&self, is_solved: bool) -> TableStatus {
        let mut headers: Vec<String> = Vec::with_capacity(3);
        let mut rows: Vec<TableRow> = Vec::new();
        let mut uom_buf = CapeArrayRealVec::new();
        let mut string_buf = CapeStringImpl::new();
        let mut array_string_buf = CapeArrayStringVec::new();
        if let Ok(collection) = self.utilities.get_parameters() {
            //headers
            headers.push("Parameter".into());
            headers.push("Value".into());
            headers.push("".into()); //unit of measure
            for parameter in collection {
                let name = match cape_open_1_2::CapeIdentification::from_object(&parameter) {
                    Ok(id) => {
                        let mut name_str = CapeStringImpl::new();
                        match id.get_component_name(&mut name_str) {
                            Ok(_) => name_str.to_string(),
                            Err(_) => {
                                continue; //skip if we can't get a name
                            }
                        }
                    }
                    Err(_) => {
                        continue; //skip if we can't get ICapeIdentification
                    }
                };
                let is_input = match parameter.get_mode() {
                    Ok(mode) => {
                        match mode {
                            cape_open_1_2::CapeParamMode::CapeInput => true,
                            cape_open_1_2::CapeParamMode::CapeOutput => false,
                            _ => {
                                //should not be used, unsupported
                                continue;
                            }
                        }
                    }
                    Err(_) => {
                        continue; //skip if we can't get mode
                    }
                };
                match parameter.get_type() {
                    Ok(param_type) => {
                        match param_type {
                            cape_open_1_2::CapeParamType::CapeParameterReal
                            | cape_open_1_2::CapeParamType::CapeParameterInteger
                            | cape_open_1_2::CapeParamType::CapeParameterString
                            | cape_open_1_2::CapeParamType::CapeParameterBoolean => {
                                //scalar
                                let mut row = TableRow::data_row(name.clone(), None);
                                row.add_cell(&name);
                                let (value, uom, options, exclusive) = match param_type {
                                    cape_open_1_2::CapeParamType::CapeParameterReal => {
                                        match cape_open_1_2::CapeRealParameter::from_object(
                                            &parameter,
                                        ) {
                                            Ok(real_param) => {
                                                let value = if is_input || is_solved {
                                                    match real_param.get_value() {
                                                        Ok(v) => {
                                                            number_formatter::format_number(
                                                                Some(v),
                                                            )
                                                        }
                                                        Err(_) => "".to_string(),
                                                    }
                                                } else {
                                                    "".to_string()
                                                };
                                                let uom = match real_param
                                                    .get_dimensionality(&mut uom_buf)
                                                {
                                                    Ok(_) => {
                                                        Some(crate::dimensionality::as_string(
                                                            uom_buf.as_vec(),
                                                        ))
                                                    }
                                                    Err(_) => None,
                                                };
                                                (value, uom, None, None)
                                            }
                                            Err(_) => {
                                                continue;
                                            }
                                        }
                                    }
                                    cape_open_1_2::CapeParamType::CapeParameterInteger => {
                                        match cape_open_1_2::CapeIntegerParameter::from_object(
                                            &parameter,
                                        ) {
                                            Ok(int_param) => {
                                                let value = if is_input || is_solved {
                                                    match int_param.get_value() {
                                                        Ok(v) => v.to_string(),
                                                        Err(_) => "".to_string(),
                                                    }
                                                } else {
                                                    "".to_string()
                                                };
                                                (value, None, None, None)
                                            }
                                            Err(_) => {
                                                continue;
                                            }
                                        }
                                    }
                                    cape_open_1_2::CapeParamType::CapeParameterString => {
                                        match cape_open_1_2::CapeStringParameter::from_object(
                                            &parameter,
                                        ) {
                                            Ok(string_param) => {
                                                let value = if is_input || is_solved {
                                                    match string_param
                                                        .get_value(&mut string_buf)
                                                    {
                                                        Ok(_) => string_buf.to_string(),
                                                        Err(_) => "".to_string(),
                                                    }
                                                } else {
                                                    "".to_string()
                                                };
                                                let (options, mut exclusive) = if is_input {
                                                    (
                                                        match string_param.get_option_list(
                                                            &mut array_string_buf,
                                                        ) {
                                                            Ok(_) => Some(
                                                                array_string_buf
                                                                    .as_string_vec(),
                                                            ),
                                                            Err(_) => None,
                                                        },
                                                        match string_param
                                                            .get_restricted_to_list()
                                                        {
                                                            Ok(exclusive) => {
                                                                Some(exclusive != 0)
                                                            }
                                                            Err(_) => None,
                                                        },
                                                    )
                                                } else {
                                                    (None, None)
                                                };
                                                if options.as_ref().is_none() {
                                                    exclusive = None;
                                                }
                                                (value, None, options, exclusive)
                                            }
                                            Err(_) => {
                                                continue;
                                            }
                                        }
                                    }
                                    cape_open_1_2::CapeParamType::CapeParameterBoolean => {
                                        match cape_open_1_2::CapeBooleanParameter::from_object(
                                            &parameter,
                                        ) {
                                            Ok(bool_param) => {
                                                let value = if is_input || is_solved {
                                                    match bool_param.get_value() {
                                                        Ok(value) => {
                                                            if value != 0 {
                                                                "true".to_string()
                                                            } else {
                                                                "false".to_string()
                                                            }
                                                        }
                                                        Err(_) => "".to_string(),
                                                    }
                                                } else {
                                                    "".to_string()
                                                };
                                                let (options, exclusive) = if is_input {
                                                    (
                                                        Some(vec![
                                                            "true".to_string(),
                                                            "false".to_string(),
                                                        ]),
                                                        Some(true),
                                                    )
                                                } else {
                                                    (None, None)
                                                };
                                                (value, None, options, exclusive)
                                            }
                                            Err(_) => {
                                                continue;
                                            }
                                        }
                                    }
                                    _ => {
                                        unreachable!()
                                    }
                                };
                                if is_input {
                                    row.with_options(
                                        value,
                                        Some(format!("unit_parameter\t{}", name)),
                                        options,
                                        exclusive,
                                    );
                                } else {
                                    row.add_result(value);
                                }
                                if let Some(uom_str) = uom {
                                    row.add_cell(&uom_str);
                                } else {
                                    row.add_cell("");
                                }
                                rows.push(row);
                            }
                            cape_open_1_2::CapeParamType::CapeParameterArrayReal
                            | cape_open_1_2::CapeParamType::CapeParameterArrayInteger
                            | cape_open_1_2::CapeParamType::CapeParameterArrayString
                            | cape_open_1_2::CapeParamType::CapeParameterArrayBoolean => {
                                //array
                                if let Ok(array_parameter) = cape_open_1_2::CapeArrayParameter::from_object(&parameter) {
                                    const MAX_ARRAY_ELEMENTS: usize = 50; //to prevent UI overload, we limit the number of array elements that can be displayed
                                    let mut sizes = CapeArrayIntegerVec::new();
                                    if array_parameter.get_size(&mut sizes).is_ok() {
                                        let mut element_count = 1;
                                        for size in sizes.as_vec() {
                                            element_count *= *size as usize;
                                        }
                                        let displayed_element_count =
                                            min(MAX_ARRAY_ELEMENTS, element_count);
                                        let mut values =
                                            Vec::with_capacity(displayed_element_count);
                                        let mut uom = String::new();
                                        let mut exclusive = false;
                                        let mut options = None;
                                        values.resize(
                                            displayed_element_count,
                                            String::new(),
                                        );
                                        match param_type {
                                            cape_open_1_2::CapeParamType::CapeParameterArrayReal => {
                                                match cape_open_1_2::CapeArrayRealParameter::from_object(&parameter) {
                                                    Ok(real_array_param) => {
                                                        match real_array_param.get_dimensionality(&mut uom_buf) {
                                                            Ok(_) => {uom=crate::dimensionality::as_string(uom_buf.as_vec())},
                                                            Err(_) => {continue;}
                                                        };
                                                        if is_input||is_solved {
                                                            let mut flat_values=CapeArrayRealVec::new();
                                                            if real_array_param.get_value(&mut flat_values).is_ok() {
                                                                for (i,value) in flat_values.as_vec().iter().enumerate() {
                                                                    if i<MAX_ARRAY_ELEMENTS {
                                                                        values[i]=number_formatter::format_number(Some(*value));
                                                                    } else {
                                                                        break;
                                                                    }
                                                                }
                                                            };
                                                        }
                                                    },
                                                    Err(_) => {
                                                        continue;
                                                    }
                                                }
                                            },
                                            cape_open_1_2::CapeParamType::CapeParameterArrayInteger => {
                                                if is_input||is_solved {
                                                    match cape_open_1_2::CapeArrayIntegerParameter::from_object(&parameter) {
                                                        Ok(int_array_param) => {
                                                            let mut flat_values=CapeArrayIntegerVec::new();
                                                            if int_array_param.get_value(&mut flat_values).is_ok() {
                                                                for (i,value) in flat_values.as_vec().iter().enumerate() {
                                                                    if i<MAX_ARRAY_ELEMENTS {
                                                                        values[i]=value.to_string();
                                                                    } else {
                                                                        break;
                                                                    }
                                                                }
                                                            };
                                                        },
                                                        Err(_) => {
                                                            continue;
                                                        }
                                                    }
                                                }
                                            },
                                            cape_open_1_2::CapeParamType::CapeParameterArrayString => {
                                                if is_input {
                                                    match cape_open_1_2::CapeArrayStringParameter::from_object(&parameter) {
                                                        Ok(string_array_param) => {
                                                            if is_solved {
                                                                let mut flat_values=CapeArrayStringVec::new();
                                                                if string_array_param.get_value(&mut flat_values).is_ok() {
                                                                    for (i,value) in flat_values.iter().enumerate() {
                                                                        if i<MAX_ARRAY_ELEMENTS {
                                                                            values[i]=value.as_string();
                                                                        } else {
                                                                            break;
                                                                        }
                                                                    }
                                                                };
                                                                if is_input {
                                                                    options=match string_array_param.get_option_list(&mut array_string_buf) {
                                                                        Ok(_) => Some(array_string_buf.as_string_vec()),
                                                                        Err(_) => None,
                                                                    };
                                                                    match string_array_param.get_restricted_to_list() {
                                                                        Ok(exclusive) => exclusive!=0,
                                                                        Err(_) => false,
                                                                    };
                                                                }
                                                            }
                                                        },
                                                        Err(_) => {
                                                            continue;
                                                        }
                                                    }
                                                }
                                            },
                                            cape_open_1_2::CapeParamType::CapeParameterArrayBoolean => {
                                                if is_input||is_solved {
                                                    match cape_open_1_2::CapeArrayBooleanParameter::from_object(&parameter) {
                                                        Ok(bool_array_param) => {
                                                            let mut flat_values=CapeArrayBooleanVec::new();
                                                            if bool_array_param.get_value(&mut flat_values).is_ok() {
                                                                for (i,value) in flat_values.as_vec().iter().enumerate() {
                                                                    if i<MAX_ARRAY_ELEMENTS {
                                                                        values[i]=if *value!=0 {"true".to_string()} else {"false".to_string()};
                                                                    } else {
                                                                        break;
                                                                    }
                                                                }
                                                            };
                                                        },
                                                        Err(_) => {
                                                            continue;
                                                        }
                                                    }
                                                }
                                                if is_input {
                                                    exclusive=true;
                                                    options=Some(vec!["true".to_string(),"false".to_string()]);
                                                }
                                            },
                                            _ => {unreachable!()},
                                        };
                                        let header_row = TableRow::header(&name, None);
                                        rows.push(header_row.clone());
                                        let mut indices = vec![0; sizes.size()];
                                        let exclusive = if options.is_none() {
                                            None
                                        } else {
                                            Some(exclusive)
                                        };
                                        for str_value in values {
                                            let mut disp_name = name.clone();
                                            disp_name.push('[');
                                            let mut edit_id =
                                                format!("unit_parameter\t{}", name);
                                            for (i, index) in indices.iter().enumerate() {
                                                if i > 0 {
                                                    disp_name.push(',');
                                                }
                                                disp_name.push_str(
                                                    &(index + 1).to_string(),
                                                );
                                                edit_id.push_str(&format!(
                                                    "\t{}",
                                                    index
                                                ));
                                            }
                                            disp_name.push(']');
                                            let mut row = TableRow::data_row(
                                                disp_name.clone(),
                                                Some(&header_row),
                                            );
                                            row.add_result(disp_name);
                                            if is_input {
                                                row.with_options(
                                                    str_value,
                                                    Some(edit_id),
                                                    options.clone(),
                                                    exclusive,
                                                );
                                            } else {
                                                row.add_result(str_value);
                                            }
                                            row.add_cell(&uom);
                                            rows.push(row);
                                            //increment indices, column-major order
                                             for (i, index) in indices.iter_mut().enumerate() {
                                                *index += 1;
                                                if *index < sizes.as_vec()[i] as usize {
                                                    break;
                                                }
                                                *index = 0;
                                            }
                                        }
                                        if displayed_element_count < element_count {
                                            let mut row = TableRow::data_row(
                                                name + "_more",
                                                Some(&header_row),
                                            );
                                            row.add_result(format!(
                                                "(showing {} of {} elements)...",
                                                displayed_element_count, element_count
                                            ));
                                            row.add_cell("");
                                            row.add_cell("");
                                            rows.push(row);
                                        }
                                    };
                                }
                            }
                        };
                    }
                    Err(_) => {
                        //error getting parameter type
                        continue;
                    }
                };
            }
        }
        TableStatus { headers, rows }
    }

    /// Builds the unit-parameter table payload from ICapeUtilities parameters.
    ///
    /// # Returns
    /// A [`TableStatus`] representing scalar/array parameter values and edits.
    pub fn write_parameter_results<T:std::io::Write>(&self, results_writer: &mut quick_xml::Writer<T>) -> Result<(), Box<dyn std::error::Error>> {
        let mut uom_buf = CapeArrayRealVec::new();
        let mut string_buf = CapeStringImpl::new();
        if let Ok(collection) = self.utilities.get_parameters() {
            if let Ok(count)=collection.get_count() && count>0 {
                let parameters_node=results_writer.create_element("Parameters");
                parameters_node.write_inner_content(|parameters_writer| {
                    for parameter in collection {
                        let name = match cape_open_1_2::CapeIdentification::from_object(&parameter) {
                            Ok(id) => {
                                let mut name_str = CapeStringImpl::new();
                                match id.get_component_name(&mut name_str) {
                                    Ok(_) => name_str.to_string(),
                                    Err(_) => {
                                        continue; //skip if we can't get a name
                                    }
                                }
                            }
                            Err(_) => {
                                continue; //skip if we can't get ICapeIdentification
                            }
                        };
                        let is_input = match parameter.get_mode() {
                            Ok(mode) => {
                                match mode {
                                    cape_open_1_2::CapeParamMode::CapeInput => true,
                                    cape_open_1_2::CapeParamMode::CapeOutput => false,
                                    _ => {
                                        //should not be used, unsupported
                                        continue;
                                    }
                                }
                            }
                            Err(_) => {
                                continue; //skip if we can't get mode
                            }
                        };
                        let parameter_type = match parameter.get_type() {
                            Ok(param_type) => param_type,
                            Err(_) => continue, //skip if we can't get type
                        };
                        let mut parameter_node=parameters_writer.create_element("Parameter")
                            .with_attribute(("name", name.as_str()))
                            .with_attribute(("mode", if is_input {"input"} else {"output"}));
                        match parameter_type {
                            cape_open_1_2::CapeParamType::CapeParameterReal => {
                                if let Ok(real_param) = cape_open_1_2::CapeRealParameter::from_object(&parameter) {
                                    if let Ok(()) = real_param.get_dimensionality(&mut uom_buf) {
                                        parameter_node = parameter_node.with_attribute(("unit", crate::dimensionality::as_string(uom_buf.as_vec()).as_str()));
                                    }
                                }
                            },
                            cape_open_1_2::CapeParamType::CapeParameterArrayReal => {
                                if let Ok(real_array_param) = cape_open_1_2::CapeArrayRealParameter::from_object(&parameter) {
                                    if let Ok(()) = real_array_param.get_dimensionality(&mut uom_buf) {
                                        parameter_node = parameter_node.with_attribute(("unit", crate::dimensionality::as_string(uom_buf.as_vec()).as_str()));
                                    }
                                }
                            },
                            _ => {}
                        }
                        match parameter_type {
                            cape_open_1_2::CapeParamType::CapeParameterReal => {
                                match cape_open_1_2::CapeRealParameter::from_object(&parameter) {
                                    Ok(real_param) => {
                                        match real_param.get_value() {
                                            Ok(value) => {
                                                parameter_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                            },
                                            Err(_) => {
                                                parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining value"))?;
                                            }
                                        };
                                    },
                                    Err(_) => {
                                        parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining ICapeRealParameter"))?;
                                    }
                                }
                            },
                            cape_open_1_2::CapeParamType::CapeParameterInteger => {
                                match cape_open_1_2::CapeIntegerParameter::from_object(&parameter) {
                                    Ok(int_param) => {
                                        match int_param.get_value() {
                                            Ok(value) => {
                                                parameter_node.write_text_content(quick_xml::events::BytesText::new(&value.to_string()))?;
                                            },
                                            Err(_) => {
                                                parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining value"))?;
                                            }
                                        };
                                    },
                                    Err(_) => {
                                        parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining ICapeIntegerParameter"))?;
                                    }
                                }
                            },
                            cape_open_1_2::CapeParamType::CapeParameterBoolean => {
                                 match cape_open_1_2::CapeBooleanParameter::from_object(&parameter) {
                                    Ok(bool_param) => {
                                        match bool_param.get_value() {
                                            Ok(value) => {
                                                parameter_node.write_text_content(quick_xml::events::BytesText::new(if value!=0 { "true" } else { "false" }))?;
                                            },
                                            Err(_) => {
                                                parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining value"))?;
                                            }
                                        };
                                    },
                                    Err(_) => {
                                        parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining ICapeBooleanParameter"))?;
                                    }
                                }
                            },
                            cape_open_1_2::CapeParamType::CapeParameterString => {
                                match cape_open_1_2::CapeStringParameter::from_object(&parameter) {
                                    Ok(string_param) => {
                                        match string_param.get_value(&mut string_buf) {
                                            Ok(()) => {
                                                parameter_node.write_text_content(quick_xml::events::BytesText::new(&string_buf.to_string()))?;
                                            },
                                            Err(_) => {
                                                parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining value"))?;
                                            }
                                        };
                                    },
                                    Err(_) => {
                                        parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining ICapeStringParameter"))?;
                                    }
                                }
                            },
                            cape_open_1_2::CapeParamType::CapeParameterArrayReal
                            | cape_open_1_2::CapeParamType::CapeParameterArrayInteger
                            | cape_open_1_2::CapeParamType::CapeParameterArrayString
                            | cape_open_1_2::CapeParamType::CapeParameterArrayBoolean => {
                                //array
                                if let Ok(array_parameter) = cape_open_1_2::CapeArrayParameter::from_object(&parameter) {
                                    const MAX_ARRAY_ELEMENTS: usize = 500; //to prevent enormous file
                                    let mut sizes = CapeArrayIntegerVec::new();
                                    if array_parameter.get_size(&mut sizes).is_ok() {
                                        let mut element_count = 1;
                                        for size in sizes.as_vec() {
                                            element_count *= *size as usize;
                                        }
                                        let displayed_element_count =
                                            min(MAX_ARRAY_ELEMENTS, element_count);
                                        let mut values =
                                            Vec::with_capacity(displayed_element_count);
                                        values.resize(
                                            displayed_element_count,
                                            String::new(),
                                        );
                                        match parameter_type {
                                            cape_open_1_2::CapeParamType::CapeParameterArrayReal => {
                                                match cape_open_1_2::CapeArrayRealParameter::from_object(&parameter) {
                                                    Ok(real_array_param) => {
                                                        let mut flat_values=CapeArrayRealVec::new();
                                                        if real_array_param.get_value(&mut flat_values).is_ok() {
                                                            for (i,value) in flat_values.as_vec().iter().enumerate() {
                                                                if i<MAX_ARRAY_ELEMENTS {
                                                                    values[i]=value.to_string();
                                                                } else {
                                                                    break;
                                                                }
                                                            }
                                                        };
                                                    },
                                                    Err(_) => {
                                                        parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining array parameter values"))?;
                                                        continue;
                                                    }
                                                }
                                            },
                                            cape_open_1_2::CapeParamType::CapeParameterArrayInteger => {
                                                match cape_open_1_2::CapeArrayIntegerParameter::from_object(&parameter) {
                                                    Ok(int_array_param) => {
                                                        let mut flat_values=CapeArrayIntegerVec::new();
                                                        if int_array_param.get_value(&mut flat_values).is_ok() {
                                                            for (i,value) in flat_values.as_vec().iter().enumerate() {
                                                                if i<MAX_ARRAY_ELEMENTS {
                                                                    values[i]=value.to_string();
                                                                } else {
                                                                    break;
                                                                }
                                                            }
                                                        };
                                                    },
                                                    Err(_) => {
                                                        parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining array parameter values"))?;
                                                        continue;
                                                    }
                                                }
                                            },
                                            cape_open_1_2::CapeParamType::CapeParameterArrayString => {
                                                if is_input {
                                                    match cape_open_1_2::CapeArrayStringParameter::from_object(&parameter) {
                                                        Ok(string_array_param) => {
                                                            let mut flat_values=CapeArrayStringVec::new();
                                                            if string_array_param.get_value(&mut flat_values).is_ok() {
                                                                for (i,value) in flat_values.iter().enumerate() {
                                                                    if i<MAX_ARRAY_ELEMENTS {
                                                                        values[i]=value.as_string();
                                                                    } else {
                                                                        break;
                                                                    }
                                                                }
                                                            };
                                                        },
                                                        Err(_) => {
                                                            parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining array parameter values"))?;
                                                            continue;
                                                        }
                                                    }
                                                }
                                            },
                                            cape_open_1_2::CapeParamType::CapeParameterArrayBoolean => {
                                                match cape_open_1_2::CapeArrayBooleanParameter::from_object(&parameter) {
                                                    Ok(bool_array_param) => {
                                                        let mut flat_values=CapeArrayBooleanVec::new();
                                                        if bool_array_param.get_value(&mut flat_values).is_ok() {
                                                            for (i,value) in flat_values.as_vec().iter().enumerate() {
                                                                if i<MAX_ARRAY_ELEMENTS {
                                                                    values[i]=if *value!=0 {"true".to_string()} else {"false".to_string()};
                                                                } else {
                                                                    break;
                                                                }
                                                            }
                                                        };
                                                    },
                                                    Err(_) => {
                                                        parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining array parameter values"))?;
                                                        continue;
                                                    }
                                                }
                                            },
                                            _ => {unreachable!()},
                                        };
                                        let mut indices = vec![0; sizes.size()];
                                        parameter_node.write_inner_content(|parameter_writer| {
                                            for str_value in values {
                                                let mut disp_index = String::with_capacity(indices.len() * 4);
                                                for (i, index) in indices.iter().enumerate() {
                                                    if i > 0 {
                                                        disp_index.push_str(",");
                                                    }
                                                    disp_index.push_str(&(index + 1).to_string());
                                                }
                                                let element_node=parameter_writer.create_element("Element").with_attribute(("index", disp_index.as_str()));
                                                element_node.write_text_content(quick_xml::events::BytesText::new(&str_value))?;
                                                //increment indices, column-major order
                                                for (i, index) in indices.iter_mut().enumerate() {
                                                    *index += 1;
                                                    if *index < sizes.as_vec()[i] as usize {
                                                        break;
                                                    }
                                                    *index = 0;
                                                }
                                            }
                                            if displayed_element_count < element_count {
                                                let info_node=parameter_writer.create_element("Info");
                                                info_node.write_text_content(quick_xml::events::BytesText::new(&format!("(showing {} of {} elements)...",displayed_element_count, element_count)))?;
                                            }
                                            Ok(())
                                        })?;
                                    } else {
                                        parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining array parameter size"))?;
                                    }
                                } else {
                                    parameter_node.write_text_content(quick_xml::events::BytesText::new("Error obtaining ICapeArrayParameter"))?;
                                }
                            }
                        };
                    }
                    Ok(())
                })?;
            }
        }
        Ok(())
    }



    /// Sets the value of a parameter based on an edit ID and value string.
    ///
    /// The edit ID is expected to be in the format "unit_parameter\t{parameter_name}\t{index1}\t{index2}..." for array parameters,
    /// or "unit_parameter\t{parameter_name}" for scalar parameters.
    ///
    /// The value string should be parseable to the appropriate type based on the parameter's type.
    ///
    /// # Parameters
    /// - `parameter_id`: The edit ID identifying the parameter to set, including indices for array parameters.
    /// - `value`: The value to set, as a string. This will be parsed to the appropriate type based on the parameter's type.
    /// - `status`: A mutable reference to a `StatusNotification` that will be updated to indicate what kind of change occurred (e.g. parameter changed, status changed).
    pub fn set_parameter_value(
        &mut self,
        parameter_id: &str,
        value: &str,
        status: &mut StatusNotification,
    ) {
        let parameter_collection = match self.utilities.get_parameters() {
            Ok(c) => c,
            Err(_) => {
                return;
            }
        };
        //split rest at second tab to get parameter name
        let parts = parameter_id.split('\t').collect::<Vec<&str>>();
        if parts.is_empty() {
            return;
        }
        let str_name = CapeStringImpl::from_string(parts[0]);
        let parameter = match parameter_collection.item_by_name(&str_name) {
            Ok(p) => p,
            Err(_) => {
                return;
            }
        };
        let parameter_type = match parameter.get_type() {
            Ok(t) => t,
            Err(_) => {
                return;
            }
        };
        *status |= StatusNotification::ParametersChanged;
        match parameter_type {
            cape_open_1_2::CapeParamType::CapeParameterReal => {
                if let Ok(real_param) = cape_open_1_2::CapeRealParameter::from_object(&parameter) && let Ok(v) = value.parse()
                && real_param.set_value(v).is_ok() {
                    *status |= StatusNotification::StatusChanged;
                }
            }
            cape_open_1_2::CapeParamType::CapeParameterInteger => {
                if let Ok(int_param) = cape_open_1_2::CapeIntegerParameter::from_object(&parameter) && let Ok(v) = value.parse()
                && int_param.set_value(v).is_ok() {
                    *status |= StatusNotification::StatusChanged;
                }
            }
            cape_open_1_2::CapeParamType::CapeParameterString => {
                if let Ok(string_param) = cape_open_1_2::CapeStringParameter::from_object(&parameter)
                    && string_param
                        .set_value(&CapeStringImpl::from_string(value))
                        .is_ok()
                    {
                        *status |= StatusNotification::StatusChanged;
                    }
            }
            cape_open_1_2::CapeParamType::CapeParameterBoolean => {
                if let Ok(bool_param) = cape_open_1_2::CapeBooleanParameter::from_object(&parameter) {
                    let val = match value.to_lowercase().as_str() {
                        "true" => true as CapeBoolean,
                        "false" => false as CapeBoolean,
                        _ => {
                            return;
                        }
                    };
                    if bool_param.set_value(val).is_ok() {
                        *status |= StatusNotification::StatusChanged;
                    }
                }
            }
            parameter_type => {
                //array parameter, create index array
                let mut indices = Vec::with_capacity(parts.len() - 1);
                for part in parts.iter().skip(1) {
                    match part.parse::<CapeInteger>() {
                        Ok(index) => indices.push(index),
                        Err(_) => {
                            return;
                        }
                    }
                }
                let indices = CapeArrayIntegerVec::from_vec(indices);
                match parameter_type {
                    cape_open_1_2::CapeParamType::CapeParameterArrayReal => {
                        if let Ok(real_array_param) = cape_open_1_2::CapeArrayRealParameter::from_object(&parameter) && let Ok(v) = value.parse()
                        && real_array_param.set_element_value(&indices, v).is_ok() {
                            *status |= StatusNotification::StatusChanged;
                        }
                    }
                    cape_open_1_2::CapeParamType::CapeParameterArrayInteger => {
                        if let Ok(int_array_param) = cape_open_1_2::CapeArrayIntegerParameter::from_object(&parameter) && let Ok(v) = value.parse()
                        && int_array_param.set_element_value(&indices, v).is_ok() {
                            *status |= StatusNotification::StatusChanged;
                        }
                    }
                    cape_open_1_2::CapeParamType::CapeParameterArrayString => {
                        if let Ok(string_array_param) = cape_open_1_2::CapeArrayStringParameter::from_object(&parameter)
                            && string_array_param
                                .set_element_value(
                                    &indices,
                                    &CapeStringImpl::from_string(value),
                                )
                                .is_ok()
                            {
                                *status |= StatusNotification::StatusChanged;
                            }
                    }
                    cape_open_1_2::CapeParamType::CapeParameterArrayBoolean => {
                        if let Ok(bool_array_param) = cape_open_1_2::CapeArrayBooleanParameter::from_object(&parameter) {
                            let val = match value.to_lowercase().as_str() {
                                "true" => true as CapeBoolean,
                                "false" => false as CapeBoolean,
                                _ => {
                                    return;
                                }
                            };
                            if bool_array_param.set_element_value(&indices, val).is_ok() {
                                *status |= StatusNotification::StatusChanged;
                            }
                        }
                    }
                    _ => unreachable!(),
                };
            }
        }
    }

    /// Executes the unit's calculation and captures any errors or log messages.
    ///
    /// # Returns
    /// - `true` if the calculation succeeded without errors.
    /// - `false` if the calculation failed with an error. In this case
    pub fn calculate(&mut self) -> bool {
        self.calculation_error = None;
        self.calculation_log.clear();
        //run calculation
        unsafe { SimulationContext::borrow_mut(&self.sim_context) }.clear_messages();
        match self.unit.calculate() {
            Ok(_) => {
                self.calculation_log =
                    unsafe { SimulationContext::borrow_mut(&self.sim_context) }.take_messages();
                true
            }
            Err(e) => {
                self.calculation_log =
                    unsafe { SimulationContext::borrow_mut(&self.sim_context) }.take_messages();
                self.calculation_error = Some(format!("Calculation failed: {}", e));
                self.calculation_log
                    .push(self.calculation_error.clone().unwrap());
                false
            }
        }
    }

    /// Retrieves a list of available report names and their associated MIME types from the unit.
    ///
    /// # Returns
    /// A vector of strings, where each string is formatted as "{mime_type}\t{report_name}".
    pub fn get_report_list(&self) -> Vec<String> {
        let mut report_list = Vec::new();
        if let Ok(report) = cape_open_1_2::CapeReport::from_object(&self.unit) {
            let mut report_names = CapeArrayStringVec::new();
            if report.get_report_names(&mut report_names).is_ok() {
                report_list.reserve(report_names.size());
                let mut report_mime_types = CapeArrayStringVec::new();
                for report_name in report_names.iter() {
                    if report.report_types(report_name, &mut report_mime_types).is_ok() {
                        //select only the first mime type for a report
                        if report_mime_types.size() > 0 {
                            report_list.push(format!(
                                "{}\t{}",
                                report_mime_types[0], report_name
                            ));
                        }
                    };
                }
            }
        }
        report_list
    }

    /// Generates raw report content bytes for a given report and MIME type.
    ///
    /// # Parameters
    /// - `mime_type`: Requested report MIME type.
    /// - `name`: Report name.
    /// - `image_size`: Desired image resolution `(width, height)` for image reports.
    ///
    /// # Returns
    /// - `Ok(Vec<u8>)` with report payload bytes.
    /// - `Err(String)` with user-facing error text.
    pub fn get_report_content(
        &self,
        mime_type: &str,
        name: &str,
        image_size: (i32, i32),
    ) -> Result<Vec<u8>, String> {
        let report_name = CapeStringImpl::from_string(name);
        let report_type = CapeStringImpl::from_string(mime_type);
        let report_language = CapeStringImpl::new(); //default
        match cape_open_1_2::CapeReport::from_object(&self.unit) {
            Ok(report) => {
                if mime_type.starts_with("text/") {
                    //get as text
                    let mut content = CapeStringImpl::new();
                    match report.generate_report(
                        &report_name,
                        &report_type,
                        &report_language,
                        &mut content,
                    ) {
                        Ok(_) => Ok(content.as_string().into_bytes()),
                        Err(e) => Err(format!("Error getting report content: {}", e)),
                    }
                } else {
                    //binary report, get as file
                    if mime_type.starts_with("image/") {
                        unsafe { SimulationContext::borrow_mut(&self.sim_context) }
                            .set_image_resolution(image_size.0, image_size.1);
                    }
                    //create a temp file name
                    let temp_dir = std::env::temp_dir();
                    let temp_file =
                        temp_dir.join(format!("report_{}_{}.tmp", name, uuid::Uuid::new_v4()));
                    let str_temp_file =
                        CapeStringImpl::from_string(temp_file.to_string_lossy());
                    match report.generate_report_file(
                        &report_name,
                        &report_type,
                        &report_language,
                        &str_temp_file,
                    ) {
                        Ok(_) => {
                            //read file content
                            match std::fs::read(&temp_file) {
                                Ok(content) => {
                                    //delete temp file
                                    let _ = std::fs::remove_file(&temp_file);
                                    Ok(content)
                                }
                                Err(e) => {
                                    let _ = std::fs::remove_file(&temp_file);
                                    Err(format!("Error reading report content from file: {}", e))
                                }
                            }
                        }
                        Err(e) => Err(format!("Error getting report content: {}", e)),
                    }
                }
            }
            Err(e) => Err(format!(
                "Error getting ICapeReport from unit operation: {}",
                e
            )),
        }
    }

    /// Saves the unit's private data to a `PersistWriter` object, which can be used for serialization.
    ///
    /// # Returns
    /// - `Ok(PersistWriter)`: A `PersistWriter` containing the saved unit data, or None in case the unit does not support persistence.
    /// - `Err(String)`: If there was an error during the saving process, with a user-facing error message.
    pub fn save(&self) -> Result<Option<cape_open_1_2::CapePersistWriter>, String> {
        match cape_open_1_2::CapePersist::from_object(&self.unit) {
            Ok(persist) => {
                let writer = PersistWriter::root_1_2();
                match persist.save(&writer, false as CapeBoolean) {
                    Ok(_) => Ok(Some(writer)),
                    Err(e) => Err(format!("Error saving unit private data: {}", e)),
                }
            }
            Err(_) => Ok(None),
        }
    }
}

impl Drop for Unit {
    fn drop(&mut self) {
        let _ = self.utilities.terminate();
    }
}
