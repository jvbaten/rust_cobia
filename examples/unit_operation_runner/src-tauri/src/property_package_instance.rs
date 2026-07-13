use crate::simulation_context::SimulationContext;
use crate::stream_specification::StreamSpecificationType;
use cobia::prelude::*;
use cobia::*;
use core::ffi::c_void;

#[cfg(not(target_os = "windows"))]
use crate::persistence::PersistReader;

/// Owns and caches the CAPE-OPEN interfaces of a single property package
/// instance.
///
/// After construction the struct pre-fetches compound data, phase data, and
/// supported flash conditions so that hot-path code can query them without
/// repeated COM/COBIA calls. The `current_context_material` raw pointer is
/// used as a stable identity token to avoid redundant `SetMaterial` calls.
///
/// `Send + Sync` are manually asserted because the raw pointer member prevents
/// the automatic derivation; callers must ensure all access is serialised through
/// the [`PropertyPackageInstanceRef`] read/write lock.
pub(crate) struct PropertyPackageInstance {
    /// `ICapeUtilities` interface, required
    utilities: cape_open_1_2::CapeUtilities,
    /// `ICapeThermoCompounds` interface for compound queries.
    pub(crate) pp_compounds: cape_open_1_2::CapeThermoCompounds,
    /// `ICapeThermoPhases` interface for phase queries.
    pub(crate) pp_phases: cape_open_1_2::CapeThermoPhases,
    /// `ICapeThermoPropertyRoutine` interface for single- and two-phase property calculations.
    pub(crate) pp_property_routine: cape_open_1_2::CapeThermoPropertyRoutine,
    /// `ICapeThermoEquilibriumRoutine` interface for flash calculations.
    pub(crate) pp_equilibrium_routine: cape_open_1_2::CapeThermoEquilibriumRoutine,
    /// `ICapeThermoMaterialContext` interface used to set the context material.
    pub(crate) pp_material_context: cape_open_1_2::CapeThermoMaterialContext,
    /// Optional `ICapeThermoUniversalConstant` interface; `None` when not supported by the property package.
    pub(crate) pp_universal_constant: Option<cape_open_1_2::CapeThermoUniversalConstant>,
    /// Cached molecular weights for each compound in kg/mol (converted from the g/mol values returned by the property package).
    pub(crate) molecular_weights: Vec<f64>,
    /// Ordered list of phase labels as returned by the property package.
    pub(crate) phase_labels: CapeArrayStringVec,
    /// Maps a phase label string to its zero-based index in `phase_labels`.
    pub(crate) phase_index_map: CapeOpenMap<usize>,
    /// Number of compounds in the property package (cached for fast access).
    pub(crate) num_compounds: usize,
    /// Cached compound names in property-package order.
    pub(crate) compound_names: Vec<String>,
    /// Maps compound ID string to its zero-based compound index.
    pub(crate) compound_map: std::collections::HashMap<String, usize>,
    /// Zero-based index of the vapour phase in `phase_labels`, or `None` when the property package does not expose a vapour phase.
    pub(crate) vapor_phase_index: Option<usize>,
    /// Set of [`StreamSpecificationType`] values the property package accepts as flash specification inputs.
    pub(crate) flash_conditions: std::collections::HashSet<StreamSpecificationType>,
    /// `true` when the property package exposes fugacity calculations via `ICapeThermoPropertyRoutine`.
    pub(crate) supports_fugacity: bool,
    /// Identity token of the material stream currently set as the context material. Stored as a raw pointer to avoid repeated `SetMaterial` calls when the same stream calls the property package multiple times.
    current_context_material: *mut c_void,
    /// Log messages produced while creating/configuring thermodynamics.
    creation_log: Vec<String>,
    /// Shared simulation context object passed to CAPE-OPEN components.
    sim_context: CapeObject,
}

unsafe impl Send for PropertyPackageInstance {}
unsafe impl Sync for PropertyPackageInstance {}

impl PropertyPackageInstance {
    /// Creates a new [`PropertyPackageInstance`] from a CAPE-OPEN property package provider.
    ///
    /// # Parameters
    /// - `provider`: CAPE-OPEN property package provider details.
    /// - `persisted_data_reader`: Optional CAPE-OPEN object containing persisted property package data to load after creation.
    ///
    /// # Returns
    /// - `Ok(Self)`: Successfully created and initialized property package instance.
    /// - `Err(e)`: If any step of creation, initialization, or data loading fails, with `e` describing the error.
    pub fn new_standalone(
        provider: &CapePMCRegistrationDetails,
        persisted_data_reader: Option<CapeObject>,
        flags: CapePMCCreationFlags,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        //create property package
        let pp = match provider.create_instance(flags|CapePMCCreationFlags::AllowRestrictedThreading) {
            Ok(obj) => obj,
            Err(e) => {
                return Err(format!(
                    "Error creating instance of stand-alone property package {}: {}",
                    provider.get_name()?,
                    e
                )
                .into());
            }
        };
        let thermo_name = provider.get_name()?;
        Self::create(pp, thermo_name, persisted_data_reader)
    }

    /// Creates a new [`PropertyPackageInstance`] from a CAPE-OPEN property package manager.
    ///
    /// # Parameters
    /// - `provider`: CAPE-OPEN property package manager registration details.
    /// - `package`: Name of the property package to create from the manager.
    /// - `persisted_data_reader`: Optional CAPE-OPEN object containing persisted property package data to load after creation.
    ///
    /// # Returns
    /// - `Ok(Self)`: Successfully created and initialized property package instance.
    /// - `Err(e)`: If any step of creation, initialization, or data loading
    pub fn new_from_manager(
        provider: &CapePMCRegistrationDetails,
        package: &str,
        persisted_data_reader: Option<CapeObject>,
        flags: CapePMCCreationFlags,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        //create property package from manager
        let ppm = match provider.create_instance(flags|CapePMCCreationFlags::AllowRestrictedThreading) {
            Ok(obj) => obj,
            Err(e) => {
                return Err(format!(
                    "Error creating instance of provider {}: {}",
                    provider.get_name()?,
                    e
                )
                .into());
            }
        };
        let ppm_utils = match cape_open_1_2::CapeUtilities::from_object(&ppm) {
            Ok(u) => u,
            Err(e) => {
                return Err(format!(
                    "Error getting ICapeUtilities from provider {}: {}",
                    provider.get_name()?,
                    e
                )
                .into());
            }
        };
        match ppm_utils.initialize() {
            Ok(_) => {}
            Err(e) => {
                return Err(format!(
                    "Error initializing provider {}: {}",
                    provider.get_name()?,
                    e
                )
                .into());
            }
        };
        //create the package from the PPM
        let ppm = match cape_open_1_2::CapeThermoPropertyPackageManager::from_object(&ppm) {
            Ok(ppm) => ppm,
            Err(_) => {
                let _ = ppm_utils.terminate();
                return Err(format!(
                    "Provider {} is not a property package manager",
                    provider.get_name()?
                )
                .into());
            }
        };
        let package_name = CapeStringImpl::from_string(package);
        match ppm.get_property_package(&package_name) {
            Ok(pp) => {
                let _ = ppm_utils.terminate();
                let mut thermo_name = provider.get_name()?;
                thermo_name = thermo_name + " / " + package;
                Self::create(pp, thermo_name, persisted_data_reader)
            }
            Err(e) => {
                let _ = ppm_utils.terminate();
                Err(format!(
                    "Error creating property package {} from provider {}: {}",
                    package,
                    provider.get_name()?,
                    e
                )
                .into())
            }
        }
    }

    /// Creates a new [`PropertyPackageInstance`] for pre- or post-editing in an isolated process.
    ///
    /// # Parameters
    /// - `provider`: CAPE-OPEN property package manager registration details, used to create the instance.
    /// - `private_data`: CAPE-OPEN object containing persisted property package data to load into the new instance.
    /// - `package`: Name of the property package to create from the manager, used for logging in case of errors.
    /// - `persisted_data_reader`: Optional CAPE-OPEN object containing persisted property package data to load after creation.
    ///
    /// # Returns
    /// - `Ok(Self)`: Successfully created and initialized property package instance.
    /// - `Err(e)`: If any step of creation, initialization, or data loading fails, with `e` describing the error.
    #[cfg(not(target_os = "windows"))]
    fn new_copy(
        provider: &CapePMCRegistrationDetails,
        package: &str,
        persisted_data_reader: Option<CapeObject>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        //create a new copy, for editing
        let is_stand_alone_pp =
            match provider.implements_cat_id(&cape_open::CATEGORYID_STANDALONEPROPERTYPACKAGE) {
                Ok(result) => result,
                Err(_) => false,
            };
        if is_stand_alone_pp {
            Self::new_standalone(provider, persisted_data_reader, CapePMCCreationFlags::IsolatedProcess)
        } else {
            Self::new_from_manager(provider, package, persisted_data_reader, CapePMCCreationFlags::IsolatedProcess)
        }
    }

    /// Creates and initialises a [`PropertyPackageInstance`] from a CAPE object.
    ///
    /// Queries the required CAPE interfaces from `property_package`, caches
    /// compound data, phase data, and flash conditions by calling
    /// `update_after_edit`.
    ///
    /// # Parameters
    /// - `property_package`: A CAPE object that must expose at minimum
    ///   ICapeThermoCompounds, ICapeThermoPhases,
    ///   ICapeThermoPropertyRoutine, ICapeThermoEquilibriumRoutine, and
    ///   ICapeThermoMaterialContext.
    ///
    /// # Returns
    /// - `Ok(Self)`: Fully initialised instance.
    /// - `Err(e)`: If any required interface cannot be obtained or if the initial cache update fails.
    fn create(
        property_package: CapeObject,
        thermo_name: String,
        persisted_data_reader: Option<CapeObject>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        //get required interfaces
        let utilities = match cape_open_1_2::CapeUtilities::from_object(&property_package) {
            Ok(utilities) => utilities,
            Err(e) => {
                return Err(format!(
                    "Error getting ICapeUtilities from property package {}: {}",
                    thermo_name, e
                )
                .into());
            }
        };
        let pp_compounds = match cape_open_1_2::CapeThermoCompounds::from_object(&property_package)
        {
            Ok(compounds) => compounds,
            Err(e) => {
                return Err(format!(
                    "Error getting ICapeThermoCompounds from property package {}: {}",
                    thermo_name, e
                )
                .into());
            }
        };
        let pp_phases = match cape_open_1_2::CapeThermoPhases::from_object(&property_package) {
            Ok(phases) => phases,
            Err(e) => {
                return Err(format!(
                    "Error getting ICapeThermoPhases from property package {}: {}",
                    thermo_name, e
                )
                .into());
            }
        };
        let pp_property_routine =
            match cape_open_1_2::CapeThermoPropertyRoutine::from_object(&property_package) {
                Ok(prop_routine) => prop_routine,
                Err(e) => {
                    return Err(format!(
                        "Error getting ICapeThermoPropertyRoutine from property package {}: {}",
                        thermo_name, e
                    )
                    .into());
                }
            };
        let pp_equilibrium_routine =
            match cape_open_1_2::CapeThermoEquilibriumRoutine::from_object(&property_package) {
                Ok(equil_routine) => equil_routine,
                Err(e) => {
                    return Err(format!(
                        "Error getting ICapeThermoEquilibriumRoutine from property package {}: {}",
                        thermo_name, e
                    )
                    .into());
                }
            };
        let pp_material_context =
            match cape_open_1_2::CapeThermoMaterialContext::from_object(&property_package) {
                Ok(mat_context) => mat_context,
                Err(e) => {
                    return Err(format!(
                        "Error getting ICapeThermoMaterialContext from property package {}: {}",
                        thermo_name, e
                    )
                    .into());
                }
            };
        let pp_universal_constant =cape_open_1_2::CapeThermoUniversalConstant::from_object(&property_package).ok();
        //put simulation context
        let sim_context = SimulationContext::create();
        unsafe { SimulationContext::borrow_mut(&sim_context) }.clear_messages();
        let context = cape_open_1_2::CapeSimulationContext::from_object(&sim_context)
            .expect("Unable to get ICapeSimulationContext from unit operation sim context");
        let _ = utilities.set_simulation_context(&context);
        //load from persistence
        if let Some(reader) = persisted_data_reader {
            let persist = match cape_open_1_2::CapePersist::from_object(&property_package) {
                Ok(p) => p,
                Err(e) => {
                    return Err(format!(
                        "Cannot depersist property package {}: cannot obtain IPersist: {}",
                        thermo_name, e
                    )
                    .into());
                }
            };
            let reader = cape_open_1_2::CapePersistReader::from_object(&reader).unwrap();
            persist.load(&reader)?;
        };
        //initialize
        match utilities.initialize() {
            Ok(_) => {}
            Err(e) => {
                return Err(
                    format!("Error initializing property package {}: {}", thermo_name, e).into(),
                );
            }
        };
        let mut res = PropertyPackageInstance {
            utilities,
            pp_compounds,
            pp_phases,
            pp_property_routine,
            pp_equilibrium_routine,
            pp_material_context,
            pp_universal_constant,
            molecular_weights: Vec::new(),
            current_context_material: std::ptr::null_mut(),
            phase_labels: CapeArrayStringVec::new(),
            phase_index_map: CapeOpenMap::new(),
            num_compounds: 0,
            compound_names: Vec::new(),
            compound_map: std::collections::HashMap::new(),
            vapor_phase_index: None,
            flash_conditions: std::collections::HashSet::new(),
            supports_fugacity: false,
            creation_log: unsafe { SimulationContext::borrow_mut(&sim_context) }.take_messages(),
            sim_context,
        };
        res.update_after_edit()?;
        Ok(res)
    }

    /// Refreshes all cached compound/phase data and flash-condition support.
    ///
    /// Should be called after the property package is reconfigured (e.g. after
    /// a user edits the thermo settings). Rebuilds `molecular_weights`,
    /// `compound_names`, `compound_map`, `phase_labels`,
    /// `phase_index_map`, `vapor_phase_index`, `flash_conditions`, and
    /// `supports_fugacity`.
    ///
    /// # Returns
    /// - `Ok(())`: Cache updated successfully.
    /// - `Err(e)`: If any CAPE call to retrieve compound or phase data fails.
    pub fn update_after_edit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        //get the compound list
        let mut comp_ids = CapeArrayStringVec::new();
        let mut formulae = CapeArrayStringVec::new();
        let mut names = CapeArrayStringVec::new();
        let mut boil_temps = CapeArrayRealVec::new();
        let mut molwts = CapeArrayRealVec::new();
        let mut casnos = CapeArrayStringVec::new();
        match self.pp_compounds.get_compound_list(
            &mut comp_ids,
            &mut formulae,
            &mut names,
            &mut boil_temps,
            &mut molwts,
            &mut casnos,
        ) {
            Ok(_) => {
                //store compound names and map, for quick lookup
                self.compound_names = names.as_vec().iter().map(|s| s.to_string()).collect();
                self.molecular_weights = molwts.as_vec().iter().map(|mw| *mw * 1e-3).collect(); //convert from g/mol to kg/mol
                self.compound_map.clear();
                for (i, comp_id) in comp_ids.as_vec().iter().enumerate() {
                    self.compound_map.insert(comp_id.to_string(), i);
                }
            }
            Err(e) => {
                return Err(
                    format!("Error getting compound list from property package: {}", e).into(),
                );
            }
        };
        //store number of compounds
        self.num_compounds = self.compound_names.len();
        //get the phase list
        let mut state_of_aggregation = CapeArrayStringVec::new();
        let mut key_compound_id = CapeArrayStringVec::new();
        match self.pp_phases.get_phase_list(
            &mut self.phase_labels,
            &mut state_of_aggregation,
            &mut key_compound_id,
        ) {
            Ok(_) => {}
            Err(e) => {
                return Err(
                    format!("Error getting phase list from property package: {}", e).into(),
                );
            }
        };
        //set up a phase index map an determine vapor phase index
        self.vapor_phase_index = None;
        self.phase_index_map.clear();
        let aggregation_state = state_of_aggregation.as_vec();
        let str_vapor = CapeStringConstNoCase::from_string("vapor");
        for (i, label) in self.phase_labels.as_vec().iter().enumerate() {
            self.phase_index_map.insert_from_cape_string(
                &CapeStringInFromProvider::from(label).as_cape_string_in(),
                i,
            );
            if self.vapor_phase_index.is_none() && str_vapor == aggregation_state[i] {
                self.vapor_phase_index = Some(i);
            }
        }
        //check valid flash conditions
        let mut single_phase_prop_list = CapeArrayStringVec::new();
        self.flash_conditions.clear();
        self.flash_conditions
            .insert(StreamSpecificationType::Temperature);
        self.flash_conditions
            .insert(StreamSpecificationType::Pressure);
        if self.vapor_phase_index.is_some() {
            self.flash_conditions
                .insert(StreamSpecificationType::MassVaporFraction);
            self.flash_conditions
                .insert(StreamSpecificationType::MolarVaporFraction);
        }
        match self
            .pp_property_routine
            .get_single_phase_prop_list(&mut single_phase_prop_list)
        {
            Ok(_) => {
                let str_enthalpy = CapeStringConstNoCase::from_string("enthalpy");
                let str_entropy = CapeStringConstNoCase::from_string("entropy");
                let str_volume = CapeStringConstNoCase::from_string("volume");
                let str_density = CapeStringConstNoCase::from_string("density");
                let str_fugacity = CapeStringConstNoCase::from_string("fugacity");
                for prop in single_phase_prop_list.as_vec() {
                    if prop == &str_enthalpy {
                        self.flash_conditions
                            .insert(StreamSpecificationType::MolarEnthalpy);
                        self.flash_conditions
                            .insert(StreamSpecificationType::SpecificEnthalpy);
                    } else if prop == &str_entropy {
                        self.flash_conditions
                            .insert(StreamSpecificationType::MolarEntropy);
                        self.flash_conditions
                            .insert(StreamSpecificationType::SpecificEntropy);
                    } else if prop == &str_volume {
                        self.flash_conditions
                            .insert(StreamSpecificationType::MolarVolume);
                        self.flash_conditions
                            .insert(StreamSpecificationType::SpecificVolume);
                    } else if prop == &str_density {
                        self.flash_conditions
                            .insert(StreamSpecificationType::MolarDensity);
                        self.flash_conditions
                            .insert(StreamSpecificationType::MassDensity);
                    } else if prop == &str_fugacity {
                        self.supports_fugacity = true;
                    }
                }
            }
            Err(_e) => {}
        };
        Ok(())
    }

    /// Opens the property package's edit dialog and updates the cache if modified.
    ///
    /// # Parameters
    /// - `parent`: Handle of the parent window for the edit dialog.
    /// - `provider`: The CAPE-OPEN property package manager registration details, used for logging in case of errors.
    /// - `package`: The name of the property package, used for logging in case of errors.
    ///
    /// # Returns
    /// - `Ok(true)`: The property package was modified and the cache was updated.
    /// - `Ok(false)`: The property package was not modified; cache is unchanged.
    /// - `Err(e)`: An error occurred while opening the edit dialog or updating the cache.
    #[cfg(target_os = "windows")]
    pub fn edit(
        &mut self,
        parent: CapeWindowId,
        _provider: &CapePMCRegistrationDetails,
        _package: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match self.utilities.edit(parent) {
            Ok(result) => {
                match result {
                    cape_open_1_2::CapeEditResult::CapeModified => {
                        //refresh cached data after edit
                        self.update_after_edit()?;
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            Err(e) => Err(format!("Error editing property package: {}", e).into()),
        }
    }

    /// Opens the property package edit dialog using an isolated edit instance.
    ///
    /// On linux and other platforms, chances are that windowing toolkits
    /// PMCs use GTL-4 and as this process uses GTL-3, we cannot pop up the
    /// PMCs GUI inside this process. To work around this, we create an isolated
    /// instance of the property package, in an isolated process.
    ///
    /// The route is to persist the property package to be edited, create an isolated
    /// instance of the same property package, depersist into the isolated instance,
    /// and then call edit on the isolated instance. If the edit is successful,
    /// we persist the edited property package and depersist back into the main
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
        package: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        //create an isolated property package for editing
        let private_data = PersistReader::reader_from_pmc_persistence(&self.utilities)?;
        let thermo2 = Self::new_copy(provider, package, private_data)?;
        let mode = match thermo2.utilities.edit(parent) {
            Ok(mode) => mode,
            Err(e) => {
                return Err(format!("Error editing property package: {}", e).into());
            }
        };
        if mode == cape_open_1_2::CapeEditResult::CapeModified {
            //save the edited property package
            let private_data = PersistReader::reader_from_pmc_persistence(&thermo2.utilities)?;
            //create a new instance
            let mut thermo3 = Self::new_copy(provider, package, private_data)?;
            //swap edited instance into the current instance
            std::mem::swap(self, &mut thermo3);
            self.update_after_edit()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns `true` if the given identity token matches the currently set
    /// context material.
    ///
    /// # Parameters
    /// - `material_id`: Raw pointer used as a stable stream identity (matches
    ///   the value passed to `ensure_context_material`).
    ///
    /// # Returns
    /// `true` when `material_id` equals the cached context pointer.
    pub fn is_context_material(&self, material_id: *mut c_void) -> bool {
        self.current_context_material == material_id
    }

    /// Sets the context material on the property package if it has changed.
    ///
    /// Compares `material_id` with the cached pointer; skips the CAPE call when
    /// they match to avoid redundant round-trips.
    ///
    /// # Parameters
    /// - `material_stream`: Smart pointer to the material to set as context.
    /// - `material_id`: Opaque identity token to cache for subsequent `is_context_material` checks.
    ///
    /// # Returns
    /// - `Ok(())`: Context material is now set to `material_stream`.
    /// - `Err(COBIAError)`: If the underlying `SetMaterial` CAPE call fails.
    pub fn ensure_context_material(
        &mut self,
        material_stream: &cape_open_1_2::CapeThermoMaterial,
        material_id: *mut c_void,
    ) -> Result<(), COBIAError> {
        //set context material
        if self.current_context_material != material_id {
            match self.pp_material_context.set_material(material_stream) {
                Ok(_) => {
                    self.current_context_material = material_id;
                    Ok(())
                }
                Err(e) => {
                    Err(e)
                }
            }
        } else {
            //do nothing, already set
            Ok(())
        }
    }

    /// Clears the context material from the property package.
    ///
    /// Calls `UnsetMaterial` on the property package (ignoring errors) and
    /// resets the cached context pointer to null.
    pub fn unset_context_material(&mut self) {
        //remove context material reference
        let _ = self.pp_material_context.unset_material();
        self.current_context_material = std::ptr::null_mut();
    }

    /// Looks up the zero-based index of a compound by its compound ID.
    ///
    /// # Parameters
    /// - `compound_id`: The compound identifier string as returned by
    ///   ICapeThermoCompounds::GetCompoundList.
    ///
    /// # Returns
    /// - `Some(&index)`: Zero-based position in the compound list.
    /// - `None`: Compound ID not found.
    pub fn get_compound_index(&self, compound_id: &str) -> Option<&usize> {
        self.compound_map.get(compound_id)
    }

    /// Returns the total number of phases exposed by the property package.
    ///
    /// # Returns
    /// Count of entries in `phase_labels`.
    pub fn get_number_of_phases(&self) -> usize {
        self.phase_labels.size()
    }

    /// Clears the log from any previously logged messages
    pub fn clear_log(&mut self) {
        unsafe { SimulationContext::borrow_mut(&self.sim_context) }.clear_messages();
    }

    /// Takes the messages in the log since last call to `clear_log` and clears the log
    ///
    /// # Returns
    /// Vector of accumulated log messages
    pub fn take_log(&mut self) -> Vec<String> {
        unsafe { SimulationContext::borrow_mut(&self.sim_context) }.take_messages()
    }

    /// Gets the log messages produced during property package creation and initialization.
    ///
    /// # Returns
    /// - `&Vec<String>` containing log messages from property package creation, loading and initialization.
    pub fn get_creation_log(&self) -> &Vec<String> {
        &self.creation_log
    }
}

impl Drop for PropertyPackageInstance {
    fn drop(&mut self) {
        //call terminate
        let _ = self.utilities.terminate(); //ignore errors
    }
}

/// Shared, thread-safe reference to a [`PropertyPackageInstance`].
///
/// All access must go through the read/write lock to satisfy the safety
/// contract required by the manually-asserted `Send + Sync` on the struct.
pub(crate) type PropertyPackageInstanceRef =
    std::sync::Arc<std::sync::RwLock<PropertyPackageInstance>>;
