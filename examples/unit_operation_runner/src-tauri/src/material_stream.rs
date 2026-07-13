use crate::{
    property_package_instance::{PropertyPackageInstance, PropertyPackageInstanceRef},
    stream_specification::{
        CompoundSpecUsed, FlowRateSpecUsed, StreamSpecification, StreamSpecificationType,
    },
};
use arrayvec::ArrayVec;
use cobia::*;
use std::{sync::LazyLock, vec};

enum PropertyInfo {
    Temperature,
    Pressure,
    Fraction,
    PhaseFraction,
    Flow,
    TotalFlow,
    SinglePhase(i32, i32, bool), //conversion order, rank, extensive
    TwoPhase(i32),               //rank
    TemperatureDerivative(CapeStringConstNoCase), //property name
    PressureDerivative(CapeStringConstNoCase), //property name
    MolFractionDerivative(CapeStringConstNoCase), //property name
    MoleNumberDerivative(CapeStringConstNoCase), //property name
}

/// Pre-built CAPE string constants and lookup tables used by material streams.
///
/// This groups frequently-used identifiers (property names, basis names,
/// flash specifications) to avoid repeated allocations and to centralize
/// property metadata lookup.
pub(crate) struct MaterialConstants {
    /// String constant for "fraction" property, used for composition representation.
    pub(crate) fraction: CapeStringConstNoCase,
    /// String constant for "temperature" property, used for temperature specification and retrieval.
    pub(crate) temperature: CapeStringConstNoCase,
    /// String constant for "pressure" property, used for pressure specification and retrieval.
    pub(crate) pressure: CapeStringConstNoCase,
    /// String constant for "enthalpy" property, used for enthalpy specification and retrieval.
    pub(crate) enthalpy: CapeStringConstNoCase,
    /// String constant for "entropy" property, used for entropy specification and retrieval.
    pub(crate) entropy: CapeStringConstNoCase,
    /// String constant for "volume" property, used for volume specification and retrieval.
    pub(crate) volume: CapeStringConstNoCase,
    /// String constant for "density" property, used for density specification and retrieval.
    pub(crate) density: CapeStringConstNoCase,
    /// String constant for "fugacity" property, used for fugacity specification and retrieval.
    pub(crate) fugacity: CapeStringConstNoCase,
    /// String constant for "phaseFraction" property, used for phase fraction specification and retrieval.
    pub(crate) phase_fraction: CapeStringConstNoCase,
    /// String constant for "totalFlow" property, used for total flow specification and retrieval.
    pub(crate) total_flow: CapeStringConstNoCase,
    /// String constant for "flow" property, used for flow specification and retrieval.
    pub(crate) flow: CapeStringConstNoCase,
    /// String constant for "mole" basis, used for basis specification and retrieval.
    pub(crate) mole_basis: CapeStringConstNoCase,
    /// String constant for "mass" basis, used for basis specification and retrieval.
    pub(crate) mass_basis: CapeStringConstNoCase,
    /// String constant for no basis, used for basis specification and retrieval.
    pub(crate) no_basis: CapeStringConstNoCase,
    /// String constant for unspecified specification, used to mark missing flash specifications.
    unspecified: CapeStringConstNoCase,
    /// String constants for temperature flash specification: Temperature/\<no basis\>/Overall, used for flash specification parsing and validation.
    flash_spec_temperature: CapeArrayStringVec,
    /// String constants for pressure flash specification: Pressure/\<no basis\>/Overall, used for flash specification parsing and validation.
    flash_spec_pressure: CapeArrayStringVec,
    /// String constants for enthalpy flash specification: Enthalpy/\<no basis\>/Overall, used for flash specification parsing and validation.
    flash_spec_enthalpy: CapeArrayStringVec,
    /// String constants for entropy flash specification: Entropy/\<no basis\>/Overall, used for flash specification parsing and validation.
    flash_spec_entropy: CapeArrayStringVec,
    /// String constants for volume flash specification: Volume/\<no basis\>/Overall, used for flash specification parsing and validation.
    flash_spec_volume: CapeArrayStringVec,
    /// String constants for density flash specification: Density/\<no basis\>/Overall, used for flash specification parsing and validation.  
    flash_spec_density: CapeArrayStringVec,
    /// Lookup map for property metadata, used for specification parsing, validation, and unit handling.
    property_info_map: CapeOpenMap<PropertyInfo>,
}

impl MaterialConstants {
    /// Constructs string constants and the property metadata map.
    ///
    /// # Returns
    /// A fully initialized [`MaterialConstants`] instance.
    fn new() -> Self {
        //construct property lookup table
        let mut property_info_map = CapeOpenMap::<PropertyInfo>::new();
        property_info_map.insert("temperature", PropertyInfo::Temperature);
        property_info_map.insert("pressure", PropertyInfo::Pressure);
        property_info_map.insert("fraction", PropertyInfo::Fraction);
        property_info_map.insert("phaseFraction", PropertyInfo::PhaseFraction);
        property_info_map.insert("flow", PropertyInfo::Flow);
        property_info_map.insert("totalFlow", PropertyInfo::TotalFlow);
        let single_phase_properties = vec![
            ("activity", PropertyInfo::SinglePhase(0, 1, false)),
            (
                "activityCoefficient",
                PropertyInfo::SinglePhase(0, 1, false),
            ),
            ("compressibility", PropertyInfo::SinglePhase(0, 0, false)),
            (
                "compressibilityFactor",
                PropertyInfo::SinglePhase(0, 0, false),
            ),
            ("density", PropertyInfo::SinglePhase(-1, 0, false)),
            (
                "diffusionCoefficient",
                PropertyInfo::SinglePhase(0, 2, false),
            ),
            (
                "dissociationConstant",
                PropertyInfo::SinglePhase(0, 0, false),
            ),
            ("enthalpy", PropertyInfo::SinglePhase(1, 0, true)),
            ("enthalpyF", PropertyInfo::SinglePhase(1, 0, true)),
            ("enthalpyNF", PropertyInfo::SinglePhase(1, 0, true)),
            ("entropy", PropertyInfo::SinglePhase(1, 0, true)),
            ("entropyF", PropertyInfo::SinglePhase(1, 0, true)),
            ("entropyNF", PropertyInfo::SinglePhase(1, 0, true)),
            ("excessEnthalpy", PropertyInfo::SinglePhase(1, 0, true)),
            ("excessEntropy", PropertyInfo::SinglePhase(1, 0, true)),
            ("excessGibbsEnergy", PropertyInfo::SinglePhase(1, 0, true)),
            (
                "excessHelmholtzEnergy",
                PropertyInfo::SinglePhase(1, 0, true),
            ),
            (
                "excessInternalEnergy",
                PropertyInfo::SinglePhase(1, 0, true),
            ),
            ("excessVolume", PropertyInfo::SinglePhase(1, 0, true)),
            ("fugacity", PropertyInfo::SinglePhase(0, 1, false)),
            (
                "fugacityCoefficient",
                PropertyInfo::SinglePhase(0, 1, false),
            ),
            ("gibbsEnergy", PropertyInfo::SinglePhase(1, 0, true)),
            ("heatCapacityCp", PropertyInfo::SinglePhase(1, 0, true)),
            ("heatCapacityCv", PropertyInfo::SinglePhase(1, 0, true)),
            ("helmholtzEnergy", PropertyInfo::SinglePhase(1, 0, true)),
            ("internalEnergy", PropertyInfo::SinglePhase(1, 0, true)),
            (
                "jouleThomsonCoefficient",
                PropertyInfo::SinglePhase(0, 0, false),
            ),
            ("logFugacity", PropertyInfo::SinglePhase(0, 1, false)),
            (
                "logFugacityCoefficient",
                PropertyInfo::SinglePhase(0, 1, false),
            ),
            (
                "meanActivityCoefficient",
                PropertyInfo::SinglePhase(0, 0, false),
            ),
            ("osmoticCoefficient", PropertyInfo::SinglePhase(0, 0, false)),
            ("pH", PropertyInfo::SinglePhase(0, 0, false)),
            ("pOH", PropertyInfo::SinglePhase(0, 0, false)),
            ("speedOfSound", PropertyInfo::SinglePhase(0, 0, false)),
            (
                "thermalConductivity",
                PropertyInfo::SinglePhase(0, 0, false),
            ),
            ("viscosity", PropertyInfo::SinglePhase(0, 0, false)),
            ("volume", PropertyInfo::SinglePhase(1, 0, true)),
        ];
        let two_phase_properties = vec![
            ("kvalue", PropertyInfo::TwoPhase(1)),
            ("logKvalue", PropertyInfo::TwoPhase(1)),
            ("surfaceTension", PropertyInfo::TwoPhase(0)),
        ];
        //insert single phase properties and derivatives
        for (prop_name, prop_info) in single_phase_properties.into_iter() {
            property_info_map.insert(prop_name, prop_info);
            //insert derivatives
            property_info_map.insert(
                &format!("{}.Dtemperature", prop_name),
                PropertyInfo::TemperatureDerivative(prop_name.into()),
            );
            property_info_map.insert(
                &format!("{}.Dpressure", prop_name),
                PropertyInfo::PressureDerivative(prop_name.into()),
            );
            property_info_map.insert(
                &format!("{}.DmolFraction", prop_name),
                PropertyInfo::MolFractionDerivative(prop_name.into()),
            );
            property_info_map.insert(
                &format!("{}.Dmoles", prop_name),
                PropertyInfo::MoleNumberDerivative(prop_name.into()),
            );
        }
        //insert two phase properties
        for (prop_name, prop_info) in two_phase_properties.into_iter() {
            property_info_map.insert(prop_name, prop_info);
            //insert derivatives
            property_info_map.insert(
                &format!("{}.Dtemperature", prop_name),
                PropertyInfo::TemperatureDerivative(prop_name.into()),
            );
            property_info_map.insert(
                &format!("{}.Dpressure", prop_name),
                PropertyInfo::PressureDerivative(prop_name.into()),
            );
            property_info_map.insert(
                &format!("{}.DmolFraction", prop_name),
                PropertyInfo::MolFractionDerivative(prop_name.into()),
            );
            property_info_map.insert(
                &format!("{}.Dmoles", prop_name),
                PropertyInfo::MoleNumberDerivative(prop_name.into()),
            );
        }
        Self {
            fraction: CapeStringConstNoCase::from_string("fraction"),
            temperature: CapeStringConstNoCase::from_string("temperature"),
            pressure: CapeStringConstNoCase::from_string("pressure"),
            enthalpy: CapeStringConstNoCase::from_string("enthalpy"),
            entropy: CapeStringConstNoCase::from_string("entropy"),
            volume: CapeStringConstNoCase::from_string("volume"),
            density: CapeStringConstNoCase::from_string("density"),
            fugacity: CapeStringConstNoCase::from_string("fugacity"),
            phase_fraction: CapeStringConstNoCase::from_string("phaseFraction"),
            total_flow: CapeStringConstNoCase::from_string("totalFlow"),
            flow: CapeStringConstNoCase::from_string("flow"),
            mole_basis: CapeStringConstNoCase::from_string("mole"),
            mass_basis: CapeStringConstNoCase::from_string("mass"),
            unspecified: CapeStringConstNoCase::from_string("unspecified"),
            no_basis: CapeStringConstNoCase::from(None),
            flash_spec_temperature: CapeArrayStringVec::from_slice(&["temperature", "", "Overall"]),
            flash_spec_pressure: CapeArrayStringVec::from_slice(&["pressure", "", "Overall"]),
            flash_spec_enthalpy: CapeArrayStringVec::from_slice(&["enthalpy", "", "Overall"]),
            flash_spec_entropy: CapeArrayStringVec::from_slice(&["entropy", "", "Overall"]),
            flash_spec_volume: CapeArrayStringVec::from_slice(&["volume", "", "Overall"]),
            flash_spec_density: CapeArrayStringVec::from_slice(&["density", "", "Overall"]),
            property_info_map,
        }
    }
}

pub(crate) static MATERIAL_CONSTANTS: LazyLock<MaterialConstants> =
    LazyLock::new(MaterialConstants::new);

#[derive(Clone, PartialEq, Eq)]
enum PropertyBasis {
    /// No basis
    None,
    /// Mass basis
    Mass,
    /// Mole basis
    Mole,
}

impl PropertyBasis {
    /// Returns a human-readable basis label.
    ///
    /// # Returns
    /// One of `"None"`, `"Mass"`, or `"Mole"`.
    fn as_str(&self) -> &str {
        match self {
            PropertyBasis::None => "None",
            PropertyBasis::Mass => "Mass",
            PropertyBasis::Mole => "Mole",
        }
    }

    /// Parses a CAPE basis string into [`PropertyBasis`].
    ///
    /// # Parameters
    /// - `basis`: CAPE basis token (mole, mass, or empty/no-basis).
    ///
    /// # Returns
    /// - `Ok(PropertyBasis)`: Parsed basis.
    /// - `Err(COBIAError)`: If the basis token is unsupported.
    fn from_string(basis: &CapeStringIn) -> Result<Self, COBIAError> {
        if &MATERIAL_CONSTANTS.mass_basis == basis {
            Ok(PropertyBasis::Mass)
        } else if &MATERIAL_CONSTANTS.mole_basis == basis {
            Ok(PropertyBasis::Mole)
        } else if &MATERIAL_CONSTANTS.no_basis == basis {
            Ok(PropertyBasis::None)
        } else {
            Err(COBIAError::Message(format!("Invalid basis: {}", basis)))
        }
    }
}

#[derive(Clone)]
enum PropertyValueData {
    Scalar(f64),
    Vector(Vec<f64>),
}

#[derive(Clone)]
struct PropertyValue {
    value: PropertyValueData,
    basis: PropertyBasis,
}

#[derive(Clone)]
struct Phase {
    /// Phase ID
    label: String,
    /// Map of property values for this phase, keyed by CAPE property name.
    value_map: CapeOpenMap<PropertyValue>,
    /// Flag indicating whether this phase is present in the current state of the stream.
    present: bool,
    /// Phase status, used to track whether a phase is at phase equilibrium
    status: cape_open_1_2::CapePhaseStatus,
}

impl Phase {
    /// Creates an empty phase container.
    ///
    /// # Parameters
    /// - `label`: Display/lookup label of the phase.
    ///
    /// # Returns
    /// A new [`Phase`] with no properties and unknown phase status.
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            value_map: CapeOpenMap::new(),
            present: false,
            status: cape_open_1_2::CapePhaseStatus::CapeUnknownphasestatus,
        }
    }

    /// Removes all stored properties and resets phase presence/status flags.
    fn clear(&mut self) {
        //clear all properties
        self.value_map.clear();
        self.present = false;
        self.status = cape_open_1_2::CapePhaseStatus::CapeUnknownphasestatus;
    }
}

/// Material stream implementation backing CAPE-OPEN thermodynamic interfaces.
///
/// The stream stores overall and per-phase properties, supports stream
/// specification handling, and delegates thermodynamic calculations to the
/// configured property package.
#[cape_object_implementation(
		interfaces = {
			cape_open_1_2::ICapeIdentification,
			cape_open_1_2::ICapeThermoMaterial,
			cape_open_1_2::ICapeThermoCompounds,
			cape_open_1_2::ICapeThermoPhases,
			cape_open_1_2::ICapeThermoPropertyRoutine,
			cape_open_1_2::ICapeThermoEquilibriumRoutine,
			cape_open_1_2::ICapeThermoUniversalConstant
		},
        create_arguments = {property_package},
        default_values ={
            overall=Phase::new("Overall"),
        }
  )]
pub(crate) struct MaterialStream {
    /// Name of the stream, copied from the port it is connected to
    pub(crate) name: String,
    /// Description of the stream, copied from the port it is connected to
    pub(crate) description: String,
    /// Reference to the currently assigned property package, used for delegating thermodynamic calculations and accessing property metadata.
    property_package: PropertyPackageInstanceRef,
    /// Stored overall properties
    overall: Phase,
    /// Storage for all phases, allocated in one go
    phases: Vec<Phase>,
    /// Phase pairs are allocated on demand; index is stored here
    two_phase_index: std::collections::HashMap<(usize, usize), usize>,
    /// Storage for two-phase combinations
    two_phases: Vec<Phase>,
    /// Remember the last call to set_present_phases during calc_equilibrium
    equilibrium_phases: Vec<usize>,
    /// Flag indicating whether the stream is fully specified
    pub(crate) is_specified: bool,
    /// If specification fails, the reason for it
    pub(crate) specification_error: Option<String>,
    /// Currently used flash specifications for the stream
    flash_specs: ArrayVec<StreamSpecificationType, 2>,
    /// Currently used flash specification values for the stream
    compound_spec: CompoundSpecUsed,
    /// Currently used flow rate specification type for the stream
    flow_rate_spec: FlowRateSpecUsed,
    /// List of user specified stream specifications
    specifications: Vec<StreamSpecification>,
    /// Flag indicating whether the stream is a feed
    pub(crate) is_feed: bool,
    /// Index used to create names of duplicate streams
    copy_index: usize,
    /// Log messages from property package routines
    log: Vec<String>,
}

impl MaterialStream {
    /// Creates a material stream object bound to a property package.
    ///
    /// # Parameters
    /// - `pp`: Shared reference to the property package wrapper.
    ///
    /// # Returns
    /// A CAPE object exposing [`MaterialStream`] interfaces.
    pub fn with_property_package(pp: PropertyPackageInstanceRef) -> CapeObject {
        //get the required interfaces from the property package
        let res = Self::create(pp);
        let mo = unsafe { Self::borrow_mut(&res) };
        mo.property_package_changed();
        res
    }

    /// Update the material stream in case of change of the connected property package.
    fn property_package_changed(&mut self) {
        //clear all phases, they will be re-populated on demand
        self.overall.clear();
        //re-create the phases
        let pp = self.property_package.read().unwrap();
        self.phases.clear();
        for phase in pp.phase_labels.iter() {
            self.phases.push(Phase::new(&phase.as_string()));
        }
        self.two_phases.clear();
        self.log.clear();
    }

    /// Rebinds this stream to a new property package and resets phase caches.
    ///
    /// # Parameters
    /// - `pp`: New property package reference to use.
    pub fn update_property_package(&mut self, pp: PropertyPackageInstanceRef) {
        self.property_package = pp;
        self.property_package_changed();
    }

    /// Returns accumulated backend log messages.
    ///
    /// # Returns
    /// Reference to the mutable stream log storage.
    pub fn get_log(&mut self) -> &Vec<String> {
        &self.log
    }

    /// Appends additional log messages.
    ///
    /// # Parameters
    /// - `log`: Messages to append in order.
    pub fn add_log(&mut self, log: Vec<String>) {
        self.log.extend(log);
    }

    /// Calculate the mixture molecular weight of the stream based on its composition and the molecular weights of the compounds in the property package.
    ///
    /// # Parameters
    /// - `pp`: Reference to the property package instance.
    /// - `phase`: Reference to the phase for which to calculate the mixture molecular weight.
    ///
    /// # Returns
    /// Result containing the mixture molecular weight or an error if calculation fails.
    fn mix_molecular_weight(
        &self,
        pp: &PropertyPackageInstance,
        phase: &Phase,
    ) -> Result<f64, COBIAError> {
        let fraction_value = phase.value_map.get(&MATERIAL_CONSTANTS.fraction);
        match fraction_value {
            Some(fraction_value) => {
                match fraction_value.value {
                    PropertyValueData::Scalar(_) => {
                        Err(COBIAError::Message(format!(
                            "Unexpected number of values for fraction of phase {}: vector expected",
                            phase.label
                        )))
                    }
                    PropertyValueData::Vector(ref fractions) => {
                        if fractions.len() != pp.molecular_weights.len() {
                            return Err(COBIAError::Message(format!(
                                "Unable to provide basis conversion; number of fractions ({}) does not match number of compounds ({})",
                                fractions.len(),
                                pp.molecular_weights.len()
                            )));
                        }
                        match fraction_value.basis {
                            PropertyBasis::Mass => {
                                //mix mw from mass fractions
                                let mut inv_mix_mw = 0.0;
                                for (i, fraction) in fractions.iter().enumerate() {
                                    let mw = pp.molecular_weights[i];
                                    if f64::is_nan(mw) {
                                        return Err(COBIAError::Message("Unable to provide basis conversion; one or more missing molecular weights".into()));
                                    }
                                    if f64::is_nan(*fraction) {
                                        return Err(COBIAError::Message("Unable to provide basis conversion; one or more missing mass fractions".into()));
                                    }
                                    inv_mix_mw += fraction / mw;
                                }
                                if inv_mix_mw == 0.0 {
                                    return Err(COBIAError::Message("Unable to provide basis conversion; sum of mass fractions is zero".into()));
                                }
                                Ok(1.0 / inv_mix_mw)
                            }
                            PropertyBasis::Mole => {
                                //mix mw from mole fractions
                                let mut mix_mw = 0.0;
                                for (i, fraction) in fractions.iter().enumerate() {
                                    let mw = pp.molecular_weights[i];
                                    if f64::is_nan(mw) {
                                        return Err(COBIAError::Message("Unable to provide basis conversion; one or more missing molecular weights".into()));
                                    }
                                    if f64::is_nan(*fraction) {
                                        return Err(COBIAError::Message("Unable to provide basis conversion; one or more missing mole fractions".into()));
                                    }
                                    mix_mw += fraction * mw;
                                }
                                Ok(mix_mw)
                            }
                            _ => Err(COBIAError::Message(format!(
                                "Invalid basis {} for fraction of phase {}",
                                fraction_value.basis.as_str(),
                                phase.label
                            ))),
                        }
                    }
                }
            }
            None => Err(COBIAError::Message(format!(
                "Unable to provide basis conversion; unknown composition of phase {}",
                phase.label
            ))),
        }
    }

    /// Ensures this material object is the property package context material.
    ///
    /// # Returns
    /// - `Ok(())`: Context is already correct or successfully updated.
    /// - `Err(COBIAError)`: Locking or property package context update failed.
    pub fn ensure_context_material(&mut self) -> Result<(), COBIAError> {
        //set context material
        let i_cape_material = &mut self.cobia_object_data.cape_open_1_2_i_cape_thermo_material
            as *mut C::ICapeInterface
            as *mut C::CAPEOPEN_1_2_ICapeThermoMaterial; //raw pointer to ICapeThermoMaterial
        let my_id = unsafe { (*i_cape_material).me }; //raw pointer to myself, serves as MO id
        let mo = unsafe {cape_open_1_2::CapeThermoMaterial::from_interface_pointer(i_cape_material)}; //smart pointer to ICapeThermoMaterial
        let is_current = self
            .property_package
            .read()
            .unwrap()
            .is_context_material(my_id);
        if !is_current {
            match self.property_package.write() {
                Ok(mut pp) => match pp.ensure_context_material(&mo, my_id) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(COBIAError::Message(format!(
                        "Failed to set context material for property package: {}",
                        e
                    ))),
                },
                Err(_) => Err(COBIAError::Message(
                    "Failed to acquire write lock on property package".into(),
                )),
            }
        } else {
            Ok(())
        }
    }

    /// Retrieves a reference to a phase by its label.
    ///
    /// # Parameters
    /// - `pp`: Reference to the property package instance.
    /// - `label`: Label of the phase to retrieve.
    ///
    /// # Returns
    /// Result containing a reference to the phase or an error if the phase is not found.
    fn get_phase(
        &self,
        pp: &PropertyPackageInstance,
        label: &CapeStringIn,
    ) -> Result<&Phase, COBIAError> {
        match pp.phase_index_map.get(label) {
            Some(phase_index) => Ok(&self.phases[*phase_index]),
            None => Err(COBIAError::Message(format!(
                "Invalid/unsupported phase '{}'",
                label
            ))),
        }
    }

    /// Retrieves a mutable reference to a phase by its label.
    ///
    /// # Parameters
    /// - `label`: Label of the phase to retrieve.
    ///
    /// # Returns
    /// Result containing a mutable reference to the phase or an error if the phase is not found
    fn get_phase_mut(&mut self, label: &CapeStringIn) -> Result<&mut Phase, COBIAError> {
        let pp = self.property_package.read().unwrap();
        match pp.phase_index_map.get(label) {
            Some(phase_index) => Ok(&mut self.phases[*phase_index]),
            None => Err(COBIAError::Message(format!(
                "Invalid/unsupported phase '{}'",
                label
            ))),
        }
    }

    /// Retrieves a reference to a two-phase property storage
    ///
    /// # Parameters
    /// - `pp`: Reference to the property package instance, used to resolve phase labels to indices.
    /// - `labels`: CAPE array of phase labels specifying the two phases in the format [phase1, phase2].
    ///
    /// # Returns
    /// Result containing a reference to the two-phase property storage or an error if the phase pair
    fn get_two_phase(
        &self,
        pp: &PropertyPackageInstance,
        labels: &CapeArrayStringIn,
    ) -> Result<&Phase, COBIAError> {
        if labels.size() != 2 {
            return Err(COBIAError::Message(format!(
                "Invalid number of phases specified for two phase property: expected 2, got {}",
                labels.size()
            )));
        }
        let phase_label_1 = match labels.at(0) {
            Ok(label) => label,
            Err(_) => {
                return Err(COBIAError::Message("Invalid phase label".into()));
            }
        };
        let phase_label_2 = match labels.at(1) {
            Ok(label) => label,
            Err(_) => {
                return Err(COBIAError::Message(String::from("Invalid phase label")));
            }
        };
        let index1 = match pp.phase_index_map.get(&phase_label_1) {
            Some(phase_index) => *phase_index,
            None => {
                return Err(COBIAError::Message(format!(
                    "Invalid/unsupported phase '{}'",
                    phase_label_1
                )));
            }
        };
        let index2 = match pp.phase_index_map.get(&phase_label_2) {
            Some(phase_index) => *phase_index,
            None => {
                return Err(COBIAError::Message(format!(
                    "Invalid/unsupported phase '{}'",
                    phase_label_2
                )));
            }
        };
        if index1 == index2 {
            return Err(COBIAError::Message(format!(
                "Identical phases specified for two phase property: '{}'",
                phase_label_1
            )));
        }
        match self.two_phase_index.get(&(index1, index2)) {
            Some(two_phase_index) => Ok(&self.two_phases[*two_phase_index]),
            None => Err(COBIAError::Message(format!(
                "Phase pair {}/{} not found in material stream",
                phase_label_1, phase_label_2
            ))),
        }
    }

    /// Retrieves a mutable reference to a two-phase property storage, allocating it if it does not exist.
    ///
    /// # Parameters
    /// - `labels`: CAPE array of phase labels specifying the two phases in the format [phase1, phase2].
    ///
    /// # Returns
    /// Result containing a mutable reference to the two-phase property storage or an error if the phase pair is not found.
    fn get_two_phase_mut(&mut self, labels: &CapeArrayStringIn) -> Result<&mut Phase, COBIAError> {
        if labels.size() != 2 {
            return Err(COBIAError::Message(format!(
                "Invalid number of phases specified for two phase property: expected 2, got {}",
                labels.size()
            )));
        }
        let pp = self.property_package.read().unwrap();
        let phase_label_1 = match labels.at(0) {
            Ok(label) => label,
            Err(_) => {
                return Err(COBIAError::Message("Invalid phase label".into()));
            }
        };
        let phase_label_2 = match labels.at(1) {
            Ok(label) => label,
            Err(_) => {
                return Err(COBIAError::Message(String::from("Invalid phase label")));
            }
        };
        let index1 = match pp.phase_index_map.get(&phase_label_1) {
            Some(phase_index) => *phase_index,
            None => {
                return Err(COBIAError::Message(format!(
                    "Invalid/unsupported phase '{}'",
                    phase_label_1
                )));
            }
        };
        let index2 = match pp.phase_index_map.get(&phase_label_2) {
            Some(phase_index) => *phase_index,
            None => {
                return Err(COBIAError::Message(format!(
                    "Invalid/unsupported phase '{}'",
                    phase_label_2
                )));
            }
        };
        if index1 == index2 {
            return Err(COBIAError::Message(format!(
                "Identical phases specified for two phase property: '{}'",
                phase_label_1
            )));
        }
        match self.two_phase_index.get(&(index1, index2)) {
            Some(two_phase_index) => Ok(&mut self.two_phases[*two_phase_index]),
            None => {
                //allocate new two phase
                let new_two_phase = Phase::new(&format!("{}/{}", phase_label_1, phase_label_2)); //label may be useful for debugging, it is never exposed
                let new_two_phase_index = self.two_phases.len();
                self.two_phases.push(new_two_phase);
                self.two_phase_index
                    .insert((index1, index2), new_two_phase_index);
                Ok(&mut self.two_phases[new_two_phase_index])
            }
        }
    }

    /// Retrieves a property value for a given phase and property name, performing basis conversion if necessary.
    ///     
    /// # Parameters
    /// - `pp`: Reference to the property package instance, used for basis conversion and error messages.
    /// - `phase`: Reference to the phase from which to retrieve the property.
    /// - `property`: CAPE property name to retrieve.
    /// - `basis`: Desired basis for the property value, used for validation and conversion.
    /// - `results`: CAPE array to store the retrieved property value(s).
    ///
    /// # Returns
    /// - `Ok(())`: Property value successfully retrieved and stored in `results`.
    fn get_phase_prop(
        &self,
        pp: &PropertyPackageInstance,
        phase: &Phase,
        property: &CapeStringIn,
        basis: PropertyBasis,
        results: &mut CapeArrayRealOut,
    ) -> Result<(), COBIAError> {
        match phase.value_map.get(property) {
            Some(prop_value) => {
                let prop_info: Option<PropertyInfo>;
                match MATERIAL_CONSTANTS.property_info_map.get(property) {
                    Some(PropertyInfo::Fraction) => {
                        if basis == PropertyBasis::None {
                            return Err(COBIAError::Message(format!(
                                "Invalid basis {} for property {} of phase {}",
                                basis.as_str(),
                                property,
                                phase.label
                            )));
                        }
                        let data = match &prop_value.value {
                            PropertyValueData::Scalar(_val) => {
                                return Err(COBIAError::Message(format!(
                                    "Unexpected number of values for {} of phase {}: vector expected",
                                    property, phase.label
                                )));
                            }
                            PropertyValueData::Vector(vec) => vec,
                        };
                        if basis == prop_value.basis {
                            //as is
                            results.put_array(data)?;
                        } else {
                            //require basis conversion
                            let mut converted = Vec::<f64>::with_capacity(data.len());
                            match prop_value.basis {
                                PropertyBasis::Mass => {
                                    //mass to mole
                                    for (i, val) in data.iter().enumerate() {
                                        let mw = pp.molecular_weights[i];
                                        if f64::is_nan(mw) {
                                            return Err(COBIAError::Message("Unable to provide basis conversion; one or more missing molecular weights".into()));
                                        }
                                        if f64::is_nan(*val) {
                                            return Err(COBIAError::Message("Unable to provide basis conversion; one or more missing mass fractions".into()));
                                        }
                                        converted.push(val / mw);
                                    }
                                }
                                PropertyBasis::Mole => {
                                    //mole to mass
                                    for (i, val) in data.iter().enumerate() {
                                        let mw = pp.molecular_weights[i];
                                        if f64::is_nan(mw) {
                                            return Err(COBIAError::Message("Unable to provide basis conversion; one or more missing molecular weights".into()));
                                        }
                                        if f64::is_nan(*val) {
                                            return Err(COBIAError::Message("Unable to provide basis conversion; one or more missing mole fractions".into()));
                                        }
                                        converted.push(val * mw);
                                    }
                                }
                                _ => {
                                    return Err(COBIAError::Message(format!(
                                        "Invalid stored basis {} for property {} of phase {}",
                                        prop_value.basis.as_str(),
                                        property,
                                        phase.label
                                    )));
                                }
                            }
                            let sum = converted.iter().sum::<f64>();
                            if sum == 0.0 {
                                return Err(COBIAError::Message(format!(
                                    "Unable to provide basis conversion; sum of fractions is zero for phase {}",
                                    phase.label
                                )));
                            }
                            let norm = 1.0 / sum;
                            for val in converted.iter_mut() {
                                *val *= norm;
                            }
                            results.put_array(&converted)?;
                        }
                        return Ok(());
                    }
                    Some(PropertyInfo::PhaseFraction) => {
                        if phase.label == "Overall" {
                            return Err(COBIAError::Message(format!(
                                "Property {} is not defined overall phase",
                                property
                            )));
                        }
                        if basis == PropertyBasis::None {
                            return Err(COBIAError::Message(format!(
                                "Invalid basis {} for property {} of phase {}",
                                basis.as_str(),
                                property,
                                phase.label
                            )));
                        }
                        let data = match &prop_value.value {
                            PropertyValueData::Scalar(val) => val,
                            PropertyValueData::Vector(_vec) => {
                                return Err(COBIAError::Message(format!(
                                    "Unexpected number of values for {} of phase {}: scalar expected",
                                    property, phase.label
                                )));
                            }
                        };
                        if basis == prop_value.basis {
                            //as is
                            results.put_scalar(*data)?;
                        } else {
                            //require basis conversion, on phase fraction; this requires composition (or rather mixture mw) of the overall phase
                            let overall_mw = self.mix_molecular_weight(pp, &self.overall)?; //kg/mol
                            let phase_mw = self.mix_molecular_weight(pp, phase)?; //kg/mol
                            let converted = match prop_value.basis {
                                PropertyBasis::Mass => {
                                    //mass to mole
                                    data * (overall_mw / phase_mw)
                                }
                                PropertyBasis::Mole => {
                                    //mole to mass
                                    data * (phase_mw / overall_mw)
                                }
                                _ => {
                                    return Err(COBIAError::Message(format!(
                                        "Invalid stored basis {} for property {} of phase {}",
                                        prop_value.basis.as_str(),
                                        property,
                                        phase.label
                                    )));
                                }
                            };
                            results.put_scalar(converted)?;
                        }
                        return Ok(());
                    }
                    Some(PropertyInfo::TotalFlow) => {
                        prop_info = Some(PropertyInfo::SinglePhase(-1, 0, true)); //mol/s or kg/s, same conversion order as density
                    }
                    Some(PropertyInfo::Flow) => {
                        //this value is never set - always set as fraction + flow
                        return Err(COBIAError::Message("internal error".into()));
                    }
                    Some(PropertyInfo::Temperature) | Some(PropertyInfo::Pressure) => {
                        prop_info = Some(PropertyInfo::SinglePhase(0, 0, false));
                    }
                    Some(PropertyInfo::SinglePhase(conversion_order, rank, extensive)) => {
                        prop_info = Some(PropertyInfo::SinglePhase(
                            *conversion_order,
                            *rank,
                            *extensive,
                        ));
                    }
                    Some(PropertyInfo::TwoPhase(_)) => {
                        //not a single phase property
                        return Err(COBIAError::Message(format!(
                            "Property {} is not a single phase property",
                            property
                        )));
                    }
                    Some(PropertyInfo::TemperatureDerivative(base_prop))
                    | Some(PropertyInfo::PressureDerivative(base_prop)) => {
                        //look up base property
                        match MATERIAL_CONSTANTS.property_info_map.get(base_prop) {
                            Some(PropertyInfo::SinglePhase(conversion_order, rank, extensive)) => {
                                prop_info = Some(PropertyInfo::SinglePhase(
                                    *conversion_order,
                                    *rank,
                                    *extensive,
                                ));
                            }
                            _ => {
                                return Err(COBIAError::Message(format!(
                                    "Property {} is not a single phase property",
                                    property
                                )));
                            }
                        }
                    }
                    Some(PropertyInfo::MolFractionDerivative(base_prop)) => {
                        //mole fraction derivative
                        if phase.label == "Overall" {
                            return Err(COBIAError::Message(format!(
                                "Property {} is not defined overall phase",
                                property
                            )));
                        }
                        match prop_value.value {
                            PropertyValueData::Scalar(_) => {
                                return Err(COBIAError::Message(format!(
                                    "Unexpected number of values for {} of phase {}",
                                    property, phase.label
                                )));
                            }
                            PropertyValueData::Vector(ref vec) => {
                                if basis == prop_value.basis {
                                    //as is
                                    results.put_array(vec)?;
                                    return Ok(());
                                }
                                if basis == PropertyBasis::None
                                    || prop_value.basis == PropertyBasis::None
                                {
                                    return Err(COBIAError::Message(format!(
                                        "Invalid for property {} of phase {}",
                                        property, phase.label
                                    )));
                                }
                                //find the base property
                                match MATERIAL_CONSTANTS.property_info_map.get(base_prop) {
                                    Some(PropertyInfo::SinglePhase(
                                        conversion_order,
                                        rank,
                                        _extensive,
                                    )) => {
                                        //ok, but basis conversion is required; more over, this basis conversion requires
                                        if *rank != 0 {
                                            //there are no known vector or matrix properties that require basis conversion
                                            return Err(COBIAError::Message(format!(
                                                "Unexpected/unsupported property rank in basis conversion for mole fraction derivative property {} of phase {}",
                                                property, phase.label
                                            )));
                                        }
                                        //the base property is required for this conversion, due to the derivative of mixture molecular weight w.r.t. mole fraction
                                        // this particular implementation does not check whether the property is already there and whether it is up to date. Instead
                                        // it creates a temporary MO to do the calculation
                                        //create temporary material
                                        let temp_material: CapeObject =
                                            Self::with_property_package(
                                                self.property_package.clone(),
                                            );
                                        //set t, p, x for current phase
                                        let phase_label_cape_string =
                                            CapeStringImpl::from_string(&phase.label);
                                        {
                                            let temp_stream = unsafe {
                                                MaterialStream::borrow_mut(&temp_material)
                                            };
                                            temp_stream.name =
                                                "internal calculation material".into();
                                            temp_stream.description="Material stream used for internal calculation of basis conversion for derivative property".into();
                                            let target_phase = match temp_stream.get_phase_mut(
                                                &CapeStringInFromProvider::from(
                                                    &phase_label_cape_string,
                                                )
                                                .as_cape_string_in(),
                                            ) {
                                                Ok(ph) => ph,
                                                Err(e) => {
                                                    return Err(COBIAError::Message(format!(
                                                        "Unable to provide basis conversion; unable to get phase {} from temporary material for derivative property {}: {}",
                                                        phase.label, property, e
                                                    )));
                                                }
                                            };
                                            let mut t = 0.0;
                                            let mut p = 0.0;
                                            let mut x = CapeArrayRealVec::new();
                                            match self.get_phase_tpfraction(
                                                pp,
                                                phase,
                                                &mut t,
                                                &mut p,
                                                &mut CapeArrayRealOutFromProvider::from(&mut x)
                                                    .as_cape_array_real_out(),
                                            ) {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    return Err(COBIAError::Message(format!(
                                                        "Unable to provide basis conversion; unable to get temperature, pressure, and mole fractions for phase {} from material stream for derivative property {}: {}",
                                                        phase.label, property, e
                                                    )));
                                                }
                                            };
                                            target_phase.value_map.insert_from_cape_string(
                                                &MATERIAL_CONSTANTS.temperature,
                                                PropertyValue {
                                                    value: PropertyValueData::Scalar(t),
                                                    basis: PropertyBasis::None,
                                                },
                                            );
                                            target_phase.value_map.insert_from_cape_string(
                                                &MATERIAL_CONSTANTS.pressure,
                                                PropertyValue {
                                                    value: PropertyValueData::Scalar(p),
                                                    basis: PropertyBasis::None,
                                                },
                                            );
                                            target_phase.value_map.insert_from_cape_string(
                                                &MATERIAL_CONSTANTS.fraction,
                                                PropertyValue {
                                                    value: PropertyValueData::Vector(x.into_vec()),
                                                    basis: PropertyBasis::Mole,
                                                },
                                            );
                                        }
                                        //calculate the property on the temporary material
                                        let temp_property_routine =
                                            cape_open_1_2::CapeThermoPropertyRoutine::from_object(
                                                &temp_material,
                                            )?;
                                        let prop_list =
                                            CapeArrayStringVec::from_cape_open_string_slice(&[
                                                base_prop,
                                            ]);
                                        let phase_label = CapeStringImpl::from_string(&phase.label);
                                        match temp_property_routine
                                            .calc_single_phase_prop(&prop_list, &phase_label)
                                        {
                                            Ok(_) => {}
                                            Err(e) => {
                                                return Err(COBIAError::Message(format!(
                                                    "Unable to provide basis conversion; unable to calculate base property {} for derivative property {} of phase {}: {}",
                                                    base_prop, property, phase.label, e
                                                )));
                                            }
                                        };
                                        //get the base property from the temporary material in the same basis as the stored value
                                        let mut base_prop_value = CapeArrayRealScalar::new();
                                        let temp_stream =
                                            unsafe { MaterialStream::borrow(&temp_material) };
                                        let phase = match temp_stream.get_phase(
                                            pp,
                                            &CapeStringInFromProvider::from(
                                                &phase_label_cape_string,
                                            )
                                            .as_cape_string_in(),
                                        ) {
                                            Ok(ph) => ph,
                                            Err(e) => {
                                                return Err(COBIAError::Message(format!(
                                                    "Unable to provide basis conversion; unable to get phase {} from temporary material for derivative property {} of phase {}: {}",
                                                    phase.label, property, phase.label, e
                                                )));
                                            }
                                        };
                                        match temp_stream.get_phase_prop(
                                            pp,
                                            phase,
                                            &CapeStringInFromProvider::from(base_prop)
                                                .as_cape_string_in(),
                                            prop_value.basis.clone(),
                                            &mut CapeArrayRealOutFromProvider::from(
                                                &mut base_prop_value,
                                            )
                                            .as_cape_array_real_out(),
                                        ) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                return Err(COBIAError::Message(format!(
                                                    "Unable to provide basis conversion; unable to get base property {} from temporary material for derivative property {} of phase {}: {}",
                                                    base_prop, property, phase.label, e
                                                )));
                                            }
                                        };
                                        let base_prop_value = base_prop_value.value();
                                        //finally, perform the basis conversion for the derivative
                                        let mix_mw = self.mix_molecular_weight(pp, phase)?; //kg/mol
                                        //get phase composition in mole fractions
                                        let fraction_value =
                                            phase.value_map.get(&MATERIAL_CONSTANTS.fraction);
                                        let fraction_value = match fraction_value {
                                            Some(fraction_value) => {
                                                match fraction_value.value {
                                                    PropertyValueData::Scalar(_) => {
                                                        return Err(COBIAError::Message(format!(
                                                            "Unable to provide basis conversion; unexpected number of values for fraction of phase {}: vector expected",
                                                            phase.label
                                                        )));
                                                    }
                                                    PropertyValueData::Vector(ref fractions) => {
                                                        if fractions.len()
                                                            != pp.molecular_weights.len()
                                                        {
                                                            return Err(COBIAError::Message(
                                                                format!(
                                                                    "Unable to provide basis conversion; number of fractions ({}) does not match number of compounds ({})",
                                                                    fractions.len(),
                                                                    pp.molecular_weights.len()
                                                                ),
                                                            ));
                                                        }
                                                        match fraction_value.basis {
                                                            PropertyBasis::Mass => {
                                                                //convert mass fractions to mole fractions
                                                                let mut mole_fractions =
                                                                    Vec::<f64>::with_capacity(
                                                                        fractions.len(),
                                                                    );
                                                                for (i, fraction) in
                                                                    fractions.iter().enumerate()
                                                                {
                                                                    let mw =
                                                                        pp.molecular_weights[i];
                                                                    if f64::is_nan(mw) {
                                                                        return Err(COBIAError::Message("Unable to provide basis conversion; one or more missing molecular weights".into()));
                                                                    }
                                                                    if f64::is_nan(*fraction) {
                                                                        return Err(COBIAError::Message("Unable to provide basis conversion; one or more missing mass fractions".into()));
                                                                    }
                                                                    mole_fractions
                                                                        .push(fraction / mw);
                                                                }
                                                                let sum = mole_fractions
                                                                    .iter()
                                                                    .sum::<f64>();
                                                                let norm = 1.0 / sum;
                                                                for val in mole_fractions.iter_mut()
                                                                {
                                                                    *val *= norm;
                                                                }
                                                                mole_fractions
                                                            }
                                                            PropertyBasis::Mole => {
                                                                //already in mole fractions
                                                                fractions.clone()
                                                            }
                                                            _ => {
                                                                return Err(COBIAError::Message(
                                                                    format!(
                                                                        "Invalid stored basis {} for fraction of phase {}",
                                                                        fraction_value
                                                                            .basis
                                                                            .as_str(),
                                                                        phase.label
                                                                    ),
                                                                ));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            None => {
                                                return Err(COBIAError::Message(format!(
                                                    "Unable to provide basis conversion; unknown composition of phase {}",
                                                    phase.label
                                                )));
                                            }
                                        };
                                        let mut conversion_power = match prop_value.basis {
                                            PropertyBasis::Mass => {
                                                1 //multply by kg/mol to go from e.g. J/kg to J/mol
                                            }
                                            PropertyBasis::Mole => {
                                                -1 //divide by kg/mol to go from e.g. J/mol to J/kg
                                            }
                                            _ => {
                                                return Err(COBIAError::Message(format!(
                                                    "Invalid stored basis {} for property {} of phase {}",
                                                    prop_value.basis.as_str(),
                                                    property,
                                                    phase.label
                                                )));
                                            }
                                        };
                                        conversion_power *= conversion_order; //negative for flow rate and density, positive for specific/molar properties
                                        match prop_value.value {
                                            PropertyValueData::Scalar(_) => {
                                                return Err(COBIAError::Message(format!(
                                                    "Unable to provide basis conversion; unexpected number of values for {} of phase {}: vector expected",
                                                    property, phase.label
                                                )));
                                            }
                                            PropertyValueData::Vector(ref vec) => {
                                                if vec.len() != fraction_value.len() {
                                                    return Err(COBIAError::Message(format!(
                                                        "Unable to provide basis conversion; number of values ({}) does not match number of compounds ({}) for property {} of phase {}",
                                                        vec.len(),
                                                        fraction_value.len(),
                                                        property,
                                                        phase.label
                                                    )));
                                                }
                                                let mw = &pp.molecular_weights;
                                                results.resize(vec.len())?;
                                                match conversion_power {
                                                    1 => {
                                                        //derivative * mix_mw + base_prop * comp_mw
                                                        for (i, val) in vec.iter().enumerate() {
                                                            let converted_value = val * mix_mw
                                                                + base_prop_value
                                                                    * fraction_value[i]
                                                                    * mw[i];
                                                            results[i] = converted_value;
                                                        }
                                                    }
                                                    -1 => {
                                                        //derivative / mix_mw - base_prop * comp_mw / mix_mw^2
                                                        let inv_mix_mw = 1.0 / mix_mw;
                                                        for (i, val) in vec.iter().enumerate() {
                                                            let converted_value = inv_mix_mw
                                                                * (val
                                                                    - base_prop_value
                                                                        * fraction_value[i]
                                                                        * mw[i]
                                                                        * inv_mix_mw);
                                                            results[i] = converted_value;
                                                        }
                                                    }
                                                    _ => {
                                                        //no other flavour exists
                                                        unreachable!()
                                                    }
                                                }
                                            }
                                        };
                                        return Ok(());
                                    }
                                    _ => {
                                        return Err(COBIAError::Message(format!(
                                            "Property {} is not a single phase property",
                                            property
                                        )));
                                    }
                                }
                            }
                        };
                    }
                    Some(PropertyInfo::MoleNumberDerivative(base_prop)) => {
                        //mole number derivative
                        // for mole number derivative, basis conversion is not supported. If the original
                        // property is mole basis, mole basis must be specified. Otherwise accept
                        // either mole or no basis
                        match MATERIAL_CONSTANTS.property_info_map.get(base_prop) {
                            Some(PropertyInfo::SinglePhase(conversion_order, rank, extensive)) => {
                                prop_info = Some(PropertyInfo::SinglePhase(
                                    *conversion_order,
                                    *rank,
                                    *extensive,
                                ));
                            }
                            None => {
                                //undefined
                                if basis == PropertyBasis::Mass {
                                    return Err(COBIAError::Message(format!(
                                        "Unsupported basis for mole number derivative of unknown/undefined property {} of phase {}",
                                        base_prop, phase.label
                                    )));
                                }
                                match &prop_value.value {
                                    PropertyValueData::Scalar(val) => {
                                        results.put_scalar(*val)?;
                                    }
                                    PropertyValueData::Vector(vec) => {
                                        results.put_array(vec)?;
                                    }
                                };
                                return Ok(());
                            }
                            _ => {
                                return Err(COBIAError::Message(format!(
                                    "Property {} is not a single phase property",
                                    property
                                )));
                            }
                        }
                    }
                    None => {
                        //unknown property; pass on, but without basis conversion
                        if basis != prop_value.basis {
                            return Err(COBIAError::Message(format!(
                                "Cannot convert basis for unknown/undefined property {} of phase {}",
                                property, phase.label
                            )));
                        }
                        match &prop_value.value {
                            PropertyValueData::Scalar(val) => {
                                results.put_scalar(*val)?;
                            }
                            PropertyValueData::Vector(vec) => {
                                results.put_array(vec)?;
                            }
                        };
                        return Ok(());
                    }
                }
                match prop_info {
                    Some(PropertyInfo::SinglePhase(conversion_order, rank, _)) => {
                        if prop_value.basis == PropertyBasis::None {
                            if conversion_order != 0 {
                                return Err(COBIAError::Message(format!(
                                    "Invalid basis None for property {} of phase {}",
                                    property, phase.label
                                )));
                            }
                        } else {
                            if conversion_order == 0 {
                                return Err(COBIAError::Message(format!(
                                    "Invalid basis {} for property {} of phase {}",
                                    basis.as_str(),
                                    property,
                                    phase.label
                                )));
                            }
                        }
                        //single phase property
                        if rank == 0 {
                            match prop_value.value {
                                PropertyValueData::Scalar(val) => {
                                    //scalar property
                                    let mut val = val;
                                    if basis != prop_value.basis {
                                        //require basis conversion
                                        let mut conversion_power = match prop_value.basis {
                                            PropertyBasis::Mass => {
                                                1 //multply by kg/mol to go from e.g. J/kg to J/mol
                                            }
                                            PropertyBasis::Mole => {
                                                -1 //divide by kg/mol to go from e.g. J/mol to J/kg
                                            }
                                            _ => {
                                                return Err(COBIAError::Message(format!(
                                                    "Invalid stored basis {} for property {} of phase {}",
                                                    prop_value.basis.as_str(),
                                                    property,
                                                    phase.label
                                                )));
                                            }
                                        };
                                        conversion_power *= conversion_order; //negative for flow rate and density, positive for specific/molar properties
                                        let mix_mw = self.mix_molecular_weight(pp, phase)?; //kg/mol
                                        match conversion_power {
                                            1 => {
                                                val *= mix_mw;
                                            }
                                            -1 => {
                                                val /= mix_mw;
                                            }
                                            _ => {
                                                //no other flavour exists
                                                unreachable!()
                                            }
                                        }
                                        results.put_scalar(val)?;
                                    } else {
                                        //as is
                                        results.put_scalar(val)?;
                                    }
                                    Ok(())
                                }
                                PropertyValueData::Vector(ref _vec) => {
                                    Err(COBIAError::Message(format!(
                                        "Unexpected number of values for {} of phase {}: scalar expected",
                                        property, phase.label
                                    )))
                                }
                            }
                        } else {
                            match prop_value.value {
                                PropertyValueData::Scalar(_) => Err(COBIAError::Message(format!(
                                    "Unexpected number of values for {} of phase {}.",
                                    property, phase.label
                                ))),
                                PropertyValueData::Vector(ref vec) => {
                                    //there are no known vector properties that require basis conversion, except composition derivatives, which are handled above
                                    if basis == prop_value.basis {
                                        results.put_array(vec)?;
                                        Ok(())
                                    } else {
                                        Err(COBIAError::Message(format!(
                                            "Basis conversion for vector property {} of phase {} is not supported",
                                            property, phase.label
                                        )))
                                    }
                                }
                            }
                        }
                    }
                    _ => Err(COBIAError::Message(format!(
                        "Property {} is not a single phase property",
                        property
                    ))),
                }
            }
            None => {
                //no stored value, auto inter-conversions of totalflow+fraction to flow
                if let Some(PropertyInfo::Flow) = MATERIAL_CONSTANTS.property_info_map.get(property) {
                    //get total flow and fraction in the correct basis
                    let mut total_flow_value = CapeArrayRealScalar::new();
                    return match self.get_phase_prop(
                        pp,
                        phase,
                        &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.total_flow)
                            .as_cape_string_in(),
                        basis.clone(),
                        &mut CapeArrayRealOutFromProvider::from(&mut total_flow_value)
                            .as_cape_array_real_out(),
                    ) {
                        Ok(()) => {
                            let mut mole_fraction_value = CapeArrayRealVec::new();
                            match self.get_phase_prop(
                                pp,
                                phase,
                                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.fraction)
                                    .as_cape_string_in(),
                                basis.clone(),
                                &mut CapeArrayRealOutFromProvider::from(
                                    &mut mole_fraction_value,
                                )
                                .as_cape_array_real_out(),
                            ) {
                                Ok(()) => {
                                    //calculate flow as total flow * fraction
                                    let total_flow = total_flow_value.value();
                                    let mut fraction = mole_fraction_value.into_vec();
                                    //multiply fraction with total flow
                                    for f in fraction.iter_mut() {
                                        *f *= total_flow;
                                    }
                                    results.put_array(&fraction)?;
                                    Ok(())
                                }
                                Err(e) => Err(COBIAError::Message(format!(
                                    "Unable to calculate flow for phase {} due to missing fraction: {}",
                                    phase.label, e
                                ))),
                            }
                        }
                        Err(e) => Err(COBIAError::Message(format!(
                            "Unable to calculate flow for phase {} due to missing total flow: {}",
                            phase.label, e
                        ))),
                    };
                }
                Err(COBIAError::Message(format!(
                    "Property {} not found for phase {}",
                    property, phase.label
                )))
            }
        }
    }

    /// Get temperature, pressure, and composition of a phase for use in property calculations, with appropriate error handling and unit conversions as needed.
    ///
    /// # Parameters
    /// - `pp`: Reference to the property package instance, used for unit conversions and accessing compound information.
    /// - `phase`: Reference to the phase for which to retrieve the properties.
    /// - `temperature`: Mutable reference to store the retrieved temperature value.
    /// - `pressure`: Mutable reference to store the retrieved pressure value.
    /// - `composition`: Mutable reference to a CAPE array to store the retrieved composition values in mole fractions.
    ///
    /// # Returns
    /// Result indicating success or containing an error if any of the required properties are missing or invalid.
    fn get_phase_tpfraction(
        &self,
        pp: &PropertyPackageInstance,
        phase: &Phase,
        temperature: &mut CapeReal,
        pressure: &mut CapeReal,
        composition: &mut CapeArrayRealOut,
    ) -> Result<(), COBIAError> {
        match phase.value_map.get(&MATERIAL_CONSTANTS.temperature) {
            Some(temp_value) => match temp_value.value {
                PropertyValueData::Scalar(val) => {
                    *temperature = val;
                }
                PropertyValueData::Vector(_) => {
                    return Err(COBIAError::Message(format!(
                        "Unexpected number of values for temperature of phase {}.",
                        phase.label
                    )));
                }
            },
            None => {
                return Err(COBIAError::Message(format!(
                    "Temperature not found for phase {}.",
                    phase.label
                )));
            }
        };
        match phase.value_map.get(&MATERIAL_CONSTANTS.pressure) {
            Some(pressure_value) => match pressure_value.value {
                PropertyValueData::Scalar(val) => {
                    *pressure = val;
                }
                PropertyValueData::Vector(_) => {
                    return Err(COBIAError::Message(format!(
                        "Unexpected number of values for pressure of phase {}.",
                        phase.label
                    )));
                }
            },
            None => {
                return Err(COBIAError::Message(format!(
                    "Pressure not found for phase {}.",
                    phase.label
                )));
            }
        };
        match phase.value_map.get(&MATERIAL_CONSTANTS.fraction) {
            Some(fraction_value) => {
                match fraction_value.value {
                    PropertyValueData::Scalar(_) => {
                        return Err(COBIAError::Message(format!(
                            "Unexpected number of values for fraction of phase {}.",
                            phase.label
                        )));
                    }
                    PropertyValueData::Vector(ref vec) => {
                        match fraction_value.basis {
                            PropertyBasis::Mass => {
                                //convert mass fractions to mole fractions
                                let mut mole_fractions = Vec::<f64>::with_capacity(vec.len());
                                for (i, fraction) in vec.iter().enumerate() {
                                    let mw = pp.molecular_weights[i];
                                    if f64::is_nan(mw) {
                                        return Err(COBIAError::Message("Unable to convert mass composition to mole composition; one or more missing molecular weights".into()));
                                    }
                                    if f64::is_nan(*fraction) {
                                        return Err(COBIAError::Message("Unable to convert mass composition to mole composition; one or more missing mass fractions".into()));
                                    }
                                    mole_fractions.push(fraction / mw);
                                }
                                let sum = mole_fractions.iter().sum::<f64>();
                                let norm = 1.0 / sum;
                                for val in mole_fractions.iter_mut() {
                                    *val *= norm;
                                }
                                composition.put_array(&mole_fractions)?;
                            }
                            PropertyBasis::Mole => {
                                //already in mole fractions
                                composition.put_array(vec)?;
                            }
                            _ => {
                                return Err(COBIAError::Message(format!(
                                    "Invalid basis {} for fraction of phase {}",
                                    fraction_value.basis.as_str(),
                                    phase.label
                                )));
                            }
                        };
                    }
                }
            }
            None => {
                return Err(COBIAError::Message(format!(
                    "Fraction not found for phase {}.",
                    phase.label
                )));
            }
        };
        Ok(())
    }

    /// Set a property value for a phase, with appropriate error handling
    ///
    /// # Parameters
    /// - `num_compounds`: Number of compounds in the system, used for validating input array sizes.
    /// - `phase`: Mutable reference to the phase for which to set the property.
    /// - `property`: CAPE string specifying the property to set.
    /// - `basis`: The basis of the provided property value (e.g., mass, mole, none).
    /// - `values`: CAPE array containing the property values to set.
    ///
    /// # Returns
    /// Result indicating success or containing an error if the property cannot be set due to validation failures
    fn put_phase_prop(
        num_compounds: usize,
        phase: &mut Phase,
        property: &CapeStringIn,
        basis: PropertyBasis,
        values: &CapeArrayRealIn,
    ) -> Result<(), COBIAError> {
        let (conversion_order, rank) = match MATERIAL_CONSTANTS.property_info_map.get(property) {
            Some(PropertyInfo::SinglePhase(conversion_order, rank, _extensive)) => {
                (*conversion_order, *rank)
            }
            Some(PropertyInfo::TwoPhase(_)) => {
                //two phase property; not supported for setting
                return Err(COBIAError::Message(format!(
                    "Property {} is a two phase property and cannot be set on a single phase",
                    property
                )));
            }
            Some(PropertyInfo::Flow) => {
                if phase.label != "Overall" {
                    //do not allow on real phase
                    return Err(COBIAError::Message(format!(
                        "Total flow property can only be set on the overall phase, not on phase {}",
                        phase.label
                    )));
                }
                {
                    if values.size() != num_compounds {
                        return Err(COBIAError::Message(format!(
                            "Incorrect number of values ({}) for flow: must match number of compounds ({})",
                            values.size(),
                            num_compounds
                        )));
                    }
                }
                //special rules: set fraction and total flow
                let flow_values = values.as_slice();
                let total_flow_value = flow_values.iter().sum();
                match phase.value_map.get_mut(&MATERIAL_CONSTANTS.total_flow) {
                    Some(ref mut prop_val) => {
                        //total flow is already there - reuse
                        prop_val.value = PropertyValueData::Scalar(total_flow_value);
                        prop_val.basis = basis.clone();
                    }
                    _ => {
                        //total flow is not there, add it
                        phase.value_map.insert_from_cape_string(
                            &MATERIAL_CONSTANTS.total_flow,
                            PropertyValue {
                                value: PropertyValueData::Scalar(total_flow_value),
                                basis: basis.clone(),
                            },
                        );
                    }
                };
                let mut set_fraction = true;
                let normalization_factor = 1.0 / total_flow_value;
                if let Some(ref mut prop_val) = phase.value_map.get_mut(&MATERIAL_CONSTANTS.fraction)
                    && let PropertyValueData::Vector(ref mut fraction_values) = prop_val.value {
                        //fraction is already there - reuse
                        fraction_values.resize(flow_values.len(), 0.0);
                        for (i, val) in flow_values.iter().enumerate() {
                            fraction_values[i] = val * normalization_factor;
                        }
                        prop_val.basis = basis.clone();
                        set_fraction = false;
                    };
                if set_fraction {
                    //set
                    let mut fraction_values = Vec::<f64>::with_capacity(flow_values.len());
                    for val in flow_values.iter() {
                        fraction_values.push(val * normalization_factor);
                    }
                    phase.value_map.insert_from_cape_string(
                        &MATERIAL_CONSTANTS.fraction,
                        PropertyValue {
                            value: PropertyValueData::Vector(fraction_values),
                            basis,
                        },
                    );
                }
                return Ok(());
            }
            Some(PropertyInfo::TotalFlow) => {
                if phase.label != "Overall" {
                    //do not allow on real phase
                    return Err(COBIAError::Message(format!(
                        "Total flow property can only be set on the overall phase, not on phase {}",
                        phase.label
                    )));
                }
                (-1, 0)
            }
            Some(PropertyInfo::Temperature) | Some(PropertyInfo::Pressure) => (0, 0),
            Some(PropertyInfo::Fraction) => {
                (2, 1) //return a non-zero value to accept basis conversion, but -1 and 1 do not apply
            }
            Some(PropertyInfo::PhaseFraction) => {
                if phase.label == "Overall" {
                    //cannot set
                    return Err(COBIAError::Message("Overall phase cannot have fraction property".to_string()));
                }
                (2, 0) //return a non-zero value to accept basis conversion, but -1 and 1 do not apply
            }
            Some(PropertyInfo::TemperatureDerivative(base_prop))
            | Some(PropertyInfo::PressureDerivative(base_prop)) => {
                match MATERIAL_CONSTANTS.property_info_map.get(base_prop) {
                    Some(PropertyInfo::SinglePhase(conversion_order, rank, _extensive)) => {
                        (*conversion_order, *rank)
                    }
                    _ => {
                        //can only be two phase property
                        return Err(COBIAError::Message(format!(
                            "Property {} is not a single phase property",
                            property
                        )));
                    }
                }
            }
            Some(PropertyInfo::MoleNumberDerivative(base_prop)) => {
                match MATERIAL_CONSTANTS.property_info_map.get(base_prop) {
                    Some(PropertyInfo::SinglePhase(conversion_order, rank, _extensive)) => {
                        (100 + *conversion_order, *rank + 1) //99, 100 and 101 have special rules for accepting basis, see below
                    }
                    _ => {
                        //can only be two phase property
                        return Err(COBIAError::Message(format!(
                            "Property {} is not a single phase property",
                            property
                        )));
                    }
                }
            }
            Some(PropertyInfo::MolFractionDerivative(base_prop)) => {
                match MATERIAL_CONSTANTS.property_info_map.get(base_prop) {
                    Some(PropertyInfo::SinglePhase(conversion_order, rank, _extensive)) => {
                        (*conversion_order, *rank + 1) //99, 100 and 101 have special rules for accepting basis
                    }
                    _ => {
                        //can only be two phase property
                        return Err(COBIAError::Message(format!(
                            "Property {} is not a single phase property",
                            property
                        )));
                    }
                }
            }
            None => {
                //unknown property; allow setting as is
                match phase.value_map.get_mut(property) {
                    Some(prop_value) => {
                        //property is already there; re-use storage
                        prop_value.basis = basis;
                        match prop_value.value {
                            PropertyValueData::Scalar(_) => {
                                if values.size() != 1 {
                                    prop_value.value = PropertyValueData::Vector(values.as_vec());
                                } else {
                                    prop_value.value =
                                        PropertyValueData::Scalar(values.as_slice()[0]);
                                }
                            }
                            PropertyValueData::Vector(ref mut vec) => {
                                vec.resize(values.size(), 0.0);
                                for (i, val) in values.as_slice().iter().enumerate() {
                                    vec[i] = *val;
                                }
                            }
                        }
                    }
                    None => {
                        //property is not there; add it
                        if values.size() == 1 {
                            phase.value_map.insert_from_cape_string(
                                property,
                                PropertyValue {
                                    value: PropertyValueData::Scalar(values.as_slice()[0]),
                                    basis,
                                },
                            );
                        } else {
                            phase.value_map.insert_from_cape_string(
                                property,
                                PropertyValue {
                                    value: PropertyValueData::Vector(values.as_vec()),
                                    basis,
                                },
                            );
                        }
                    }
                }
                return Ok(());
            }
        };
        match conversion_order {
            0 => {
                //require no bases
                if basis != PropertyBasis::None {
                    return Err(COBIAError::Message(format!(
                        "Invalid basis {} for property {}",
                        basis.as_str(),
                        property
                    )));
                }
            }
            99 | 101 => {
                //mole number derivatives of a property that requires basis must be mole
                if basis != PropertyBasis::Mole {
                    return Err(COBIAError::Message(format!(
                        "Invalid basis {} for property {}; this property is suppored only for Mole basis",
                        basis.as_str(),
                        property
                    )));
                }
            }
            100 => {
                //mole number derivative property that does not require basis, accept mole or none, but not mass
                if basis == PropertyBasis::Mass {
                    return Err(COBIAError::Message(format!(
                        "Invalid basis {} for property {}; this property is not suppored Mass basis",
                        basis.as_str(),
                        property
                    )));
                }
            }
            _ => {
                //require a basis
                if basis == PropertyBasis::None {
                    return Err(COBIAError::Message(format!(
                        "Invalid basis {} for property {}",
                        basis.as_str(),
                        property
                    )));
                }
            }
        }
        if rank == 0 {
            //scalar, check size
            if values.size() != 1 {
                return Err(COBIAError::Message(format!(
                    "Unexpected number of values for {} of phase {}: scalar expected",
                    property, phase.label
                )));
            }
            match phase.value_map.get_mut(property) {
                Some(prop_value) => {
                    //property is already there; re-use storage
                    prop_value.value = PropertyValueData::Scalar(values.as_slice()[0]);
                    prop_value.basis = basis;
                }
                None => {
                    //property is not there; add it
                    phase.value_map.insert_from_cape_string(
                        property,
                        PropertyValue {
                            value: PropertyValueData::Scalar(values.as_slice()[0]),
                            basis,
                        },
                    );
                }
            }
        } else {
            if rank == 1
                && values.size() != num_compounds {
                    return Err(COBIAError::Message(format!(
                        "Incorrect number of values ({}) for {}: must match number of compounds ({})",
                        values.size(),
                        property,
                        num_compounds
                    )));
                }
            match phase.value_map.get_mut(property) {
                Some(prop_value) => {
                    //property is already there; re-use storage
                    match prop_value.value {
                        PropertyValueData::Scalar(_) => {
                            //should not happen, reset
                            prop_value.value = PropertyValueData::Vector(values.as_vec());
                        }
                        PropertyValueData::Vector(ref mut vec) => {
                            //reuse
                            vec.resize(values.size(), 0.0);
                            for (i, val) in values.as_slice().iter().enumerate() {
                                vec[i] = *val;
                            }
                        }
                    };
                    prop_value.basis = basis;
                }
                None => {
                    //property is not there; add it
                    phase.value_map.insert_from_cape_string(
                        property,
                        PropertyValue {
                            value: PropertyValueData::Vector(values.as_vec()),
                            basis,
                        },
                    );
                }
            };
        }
        //seting any property sets the phase present and unknown
        phase.status = cape_open_1_2::CapePhaseStatus::CapeUnknownphasestatus;
        phase.present = true;
        Ok(())
    }

    /// Returns which specification families are currently active.
    ///
    /// # Returns
    /// Tuple of:
    /// - active flash specifications (up to two),
    /// - active compound specification mode,
    /// - active flow-rate specification mode.
    pub fn get_specification_usage(
        &self,
    ) -> (
        ArrayVec<StreamSpecificationType, 2>,
        CompoundSpecUsed,
        FlowRateSpecUsed,
    ) {
        (
            self.flash_specs.clone(),
            self.compound_spec.clone(),
            self.flow_rate_spec.clone(),
        )
    }

    /// Adds, updates, or removes one stream specification.
    ///
    /// # Parameters
    /// - `spec_type`: Specification kind to modify.
    /// - `value`: Numeric value text; empty/whitespace removes the spec.
    ///
    /// # Returns
    /// `true` if the specification set changed, `false` otherwise.
    pub fn specify(&mut self, spec_type: StreamSpecificationType, value: &str) -> bool {
        //check if empty value - if so, remove specification
        //find if the specification is already there
        let mut need_update = false;
        let spec_index = self
            .specifications
            .iter()
            .position(|s| s.specification_type == spec_type);
        if value.trim().is_empty() {
            match spec_index {
                Some(i) => {
                    //remove specification
                    self.specifications.remove(i);
                    need_update = true;
                }
                None => {
                    //nothing to remove
                }
            };
        } else {
            //parse the value
            let value: f64 = match value.parse() {
                Ok(v) => v,
                Err(_) => {
                    return false;
                }
            };
            match spec_index {
                Some(i) => {
                    //update existing specification
                    if self.specifications[i].value != value {
                        need_update = true;
                        self.specifications[i].value = value;
                    }
                }
                None => {
                    //insert new specification
                    self.specifications.push(StreamSpecification {
                        specification_type: spec_type,
                        value,
                    });
                    need_update = true;
                }
            }
        }
        if need_update {
            self.apply_specifications();
        }
        need_update
    }

    /// Rebuilds stream state from current specification entries.
    ///
    /// Result status is reflected in:
    /// - [`Self::is_specified`]
    /// - [`Self::specification_error`]
    pub fn apply_specifications(&mut self) {
        self.specification_error = None;
        self.is_specified = false;
        self.clear();
        //process spec and set (even if incomplete, as this allows basis conversion etc)
        let mut use_molar_vapor_fraction = false;
        let mut use_mass_vapor_fraction = false;
        let mut compound_values = Vec::new(); //to hold mole or mass fractions for compound spec, if needed for basis conversion
        {
            let pp = self.property_package.read().unwrap();
            compound_values.resize(pp.num_compounds, f64::NAN);
        }
        let mut vapor_fraction_spec = CapeArrayStringVec::new();
        let mut flash_specs = ArrayVec::new();
        let mut flash_conditions = ArrayVec::<&CapeArrayStringVec, 2>::new();
        let mut compound_spec = if compound_values.len() == 1 {
            CompoundSpecUsed::MoleFraction
        } else {
            CompoundSpecUsed::None
        };
        let mut flow_rate_spec = FlowRateSpecUsed::None;
        self.specifications.retain(|spec| {
            match &spec.specification_type {
                StreamSpecificationType::Temperature
                    if flash_specs.len() < 2 => {
                        flash_specs.push(StreamSpecificationType::Temperature);
                        self.overall.value_map.insert_from_cape_string(
                            &MATERIAL_CONSTANTS.temperature,
                            PropertyValue {
                                value: PropertyValueData::Scalar(spec.value),
                                basis: PropertyBasis::None,
                            },
                        );
                        flash_conditions.push(&MATERIAL_CONSTANTS.flash_spec_temperature);
                        true
                    }
                StreamSpecificationType::Pressure
                    if flash_specs.len() < 2 => {
                        flash_specs.push(StreamSpecificationType::Pressure);
                        self.overall.value_map.insert_from_cape_string(
                            &MATERIAL_CONSTANTS.pressure,
                            PropertyValue {
                                value: PropertyValueData::Scalar(spec.value),
                                basis: PropertyBasis::None,
                            },
                        );
                        flash_conditions.push(&MATERIAL_CONSTANTS.flash_spec_pressure);
                        true
                    }
                StreamSpecificationType::MolarEnthalpy
                    if flash_specs.len() < 2
                        && (flash_specs.is_empty()
                            || flash_specs[0] != StreamSpecificationType::SpecificEnthalpy)
                        => {
                            flash_specs.push(StreamSpecificationType::MolarEnthalpy);
                            self.overall.value_map.insert_from_cape_string(
                                &MATERIAL_CONSTANTS.enthalpy,
                                PropertyValue {
                                    value: PropertyValueData::Scalar(spec.value),
                                    basis: PropertyBasis::Mole,
                                },
                            );
                            flash_conditions.push(&MATERIAL_CONSTANTS.flash_spec_enthalpy);
                            true
                        }
                StreamSpecificationType::SpecificEnthalpy
                    if flash_specs.len() < 2
                        && (flash_specs.is_empty()
                            || flash_specs[0] != StreamSpecificationType::MolarEnthalpy)
                        => {
                            flash_specs.push(StreamSpecificationType::SpecificEnthalpy);
                            self.overall.value_map.insert_from_cape_string(
                                &MATERIAL_CONSTANTS.enthalpy,
                                PropertyValue {
                                    value: PropertyValueData::Scalar(spec.value),
                                    basis: PropertyBasis::Mass,
                                },
                            );
                            flash_conditions.push(&MATERIAL_CONSTANTS.flash_spec_enthalpy);
                            true
                        }
                StreamSpecificationType::SpecificEntropy
                    if flash_specs.len() < 2
                        && (flash_specs.is_empty()
                            || flash_specs[0] != StreamSpecificationType::MolarEntropy)
                        => {
                            flash_specs.push(StreamSpecificationType::SpecificEntropy);
                            self.overall.value_map.insert_from_cape_string(
                                &MATERIAL_CONSTANTS.entropy,
                                PropertyValue {
                                    value: PropertyValueData::Scalar(spec.value),
                                    basis: PropertyBasis::Mass,
                                },
                            );
                            flash_conditions.push(&MATERIAL_CONSTANTS.flash_spec_entropy);
                            true
                        }
                StreamSpecificationType::MolarEntropy
                    if flash_specs.len() < 2
                        && (flash_specs.is_empty()
                            || flash_specs[0] != StreamSpecificationType::SpecificEntropy)
                        => {
                            flash_specs.push(StreamSpecificationType::MolarEntropy);
                            self.overall.value_map.insert_from_cape_string(
                                &MATERIAL_CONSTANTS.entropy,
                                PropertyValue {
                                    value: PropertyValueData::Scalar(spec.value),
                                    basis: PropertyBasis::Mole,
                                },
                            );
                            flash_conditions.push(&MATERIAL_CONSTANTS.flash_spec_entropy);
                            true
                        }
                StreamSpecificationType::SpecificVolume
                    if flash_specs.len() < 2
                        && (flash_specs.is_empty()
                            || flash_specs[0] != StreamSpecificationType::MolarVolume)
                        => {
                            flash_specs.push(StreamSpecificationType::SpecificVolume);
                            self.overall.value_map.insert_from_cape_string(
                                &MATERIAL_CONSTANTS.volume,
                                PropertyValue {
                                    value: PropertyValueData::Scalar(spec.value),
                                    basis: PropertyBasis::Mass,
                                },
                            );
                            flash_conditions.push(&MATERIAL_CONSTANTS.flash_spec_volume);
                            true
                        }
                StreamSpecificationType::MolarVolume
                    if flash_specs.len() < 2
                        && (flash_specs.is_empty()
                            || flash_specs[0] != StreamSpecificationType::SpecificVolume)
                        => {
                            flash_specs.push(StreamSpecificationType::MolarVolume);
                            self.overall.value_map.insert_from_cape_string(
                                &MATERIAL_CONSTANTS.volume,
                                PropertyValue {
                                    value: PropertyValueData::Scalar(spec.value),
                                    basis: PropertyBasis::Mole,
                                },
                            );
                            flash_conditions.push(&MATERIAL_CONSTANTS.flash_spec_volume);
                            true
                        }
                StreamSpecificationType::MassDensity
                    if flash_specs.len() < 2
                        && (flash_specs.is_empty()
                            || flash_specs[0] != StreamSpecificationType::MolarDensity)
                        => {
                            flash_specs.push(StreamSpecificationType::MassDensity);
                            self.overall.value_map.insert_from_cape_string(
                                &MATERIAL_CONSTANTS.density,
                                PropertyValue {
                                    value: PropertyValueData::Scalar(spec.value),
                                    basis: PropertyBasis::Mass,
                                },
                            );
                            flash_conditions.push(&MATERIAL_CONSTANTS.flash_spec_density);
                            true
                        }
                StreamSpecificationType::MolarDensity
                    if flash_specs.len() < 2
                        && (flash_specs.is_empty()
                            || flash_specs[0] != StreamSpecificationType::MassDensity)
                        => {
                            flash_specs.push(StreamSpecificationType::MolarDensity);
                            self.overall.value_map.insert_from_cape_string(
                                &MATERIAL_CONSTANTS.density,
                                PropertyValue {
                                    value: PropertyValueData::Scalar(spec.value),
                                    basis: PropertyBasis::Mole,
                                },
                            );
                            flash_conditions.push(&MATERIAL_CONSTANTS.flash_spec_density);
                            true
                        }
                StreamSpecificationType::MolarVaporFraction
                    if flash_specs.len() < 2
                        && (flash_specs.is_empty()
                            || flash_specs[0] != StreamSpecificationType::MassVaporFraction)
                        => {
                            flash_specs.push(StreamSpecificationType::MolarVaporFraction);
                            let pp = self.property_package.read().unwrap();
                            let vapor = &mut self.phases[pp.vapor_phase_index.unwrap()];
                            vapor.value_map.insert_from_cape_string(
                                &MATERIAL_CONSTANTS.phase_fraction,
                                PropertyValue {
                                    value: PropertyValueData::Scalar(spec.value),
                                    basis: PropertyBasis::Mole,
                                },
                            );
                            use_molar_vapor_fraction = true;
                            true
                        }
                StreamSpecificationType::MassVaporFraction
                    if flash_specs.len() < 2
                        && (flash_specs.is_empty()
                            || flash_specs[0] != StreamSpecificationType::MolarVaporFraction)
                        => {
                            flash_specs.push(StreamSpecificationType::MassVaporFraction);
                            let pp = self.property_package.read().unwrap();
                            let vapor = &mut self.phases[pp.vapor_phase_index.unwrap()];
                            vapor.value_map.insert_from_cape_string(
                                &MATERIAL_CONSTANTS.phase_fraction,
                                PropertyValue {
                                    value: PropertyValueData::Scalar(spec.value),
                                    basis: PropertyBasis::Mass,
                                },
                            );
                            use_mass_vapor_fraction = true;
                            true
                        }
                StreamSpecificationType::MolarFraction(compound)
                    if (compound_spec == CompoundSpecUsed::MoleFraction
                        || compound_spec == CompoundSpecUsed::None)
                    => {
                        let pp = self.property_package.read().unwrap();
                        if let Some(compound_index) = pp.get_compound_index(compound) {
                            compound_values[*compound_index ] = spec.value;
                            compound_spec = CompoundSpecUsed::MoleFraction;
                            true
                        } else {
                            //invalid compound id; ignore
                            false
                        }
                    }
                StreamSpecificationType::MassFraction(compound)
                    if (compound_spec == CompoundSpecUsed::MassFraction
                        || compound_spec == CompoundSpecUsed::None)
                    => {
                        let pp = self.property_package.read().unwrap();
                        if let Some(compound_index) = pp.get_compound_index(compound) {
                            compound_values[*compound_index ] = spec.value;
                            compound_spec = CompoundSpecUsed::MassFraction;
                            true
                        } else {
                            //invalid compound id; ignore
                            false
                        }
                    }
                StreamSpecificationType::MassFlowRate
                    if (flow_rate_spec == FlowRateSpecUsed::MassFlow
                        || flow_rate_spec == FlowRateSpecUsed::None)
                    => {
                        flow_rate_spec = FlowRateSpecUsed::MassFlow;
                        self.overall.value_map.insert_from_cape_string(
                            &MATERIAL_CONSTANTS.total_flow,
                            PropertyValue {
                                value: PropertyValueData::Scalar(spec.value),
                                basis: PropertyBasis::Mass,
                            },
                        );
                        true
                    }
                StreamSpecificationType::MolarFlowRate
                    if (flow_rate_spec == FlowRateSpecUsed::MolarFlow
                        || flow_rate_spec == FlowRateSpecUsed::None)
                    => {
                        flow_rate_spec = FlowRateSpecUsed::MolarFlow;
                        self.overall.value_map.insert_from_cape_string(
                            &MATERIAL_CONSTANTS.total_flow,
                            PropertyValue {
                                value: PropertyValueData::Scalar(spec.value),
                                basis: PropertyBasis::Mole,
                            },
                        );
                        true
                    }
                StreamSpecificationType::MassCompoundFlowRate(compound)
                    if (flow_rate_spec == FlowRateSpecUsed::MassCompoundFlow
                        || flow_rate_spec == FlowRateSpecUsed::None)
                        && (compound_spec == CompoundSpecUsed::MassCompoundFlow
                            || compound_spec == CompoundSpecUsed::None)
                    => {
                        let pp = self.property_package.read().unwrap();
                        if let Some(compound_index) = pp.get_compound_index(compound) {
                            compound_values[*compound_index ] = spec.value;
                            flow_rate_spec = FlowRateSpecUsed::MassCompoundFlow;
                            compound_spec = CompoundSpecUsed::MassCompoundFlow;
                            true
                        } else {
                            //invalid compound id; ignore
                            false
                        }
                    }
                StreamSpecificationType::MolarCompoundFlowRate(compound)
                    if (flow_rate_spec == FlowRateSpecUsed::MolarCompoundFlow
                        || flow_rate_spec == FlowRateSpecUsed::None)
                        && (compound_spec == CompoundSpecUsed::MolarCompoundFlow
                            || compound_spec == CompoundSpecUsed::None)
                    => {
                        let pp = self.property_package.read().unwrap();
                        if let Some(compound_index) = pp.get_compound_index(compound) {
                            compound_values[*compound_index ] = spec.value;
                            flow_rate_spec = FlowRateSpecUsed::MolarCompoundFlow;
                            compound_spec = CompoundSpecUsed::MolarCompoundFlow;
                            true
                        } else {
                            //invalid compound id; ignore
                            false
                        }
                    }
                _ => {
                    //not a stream specification
                    false
                }
            }
        });
        //update specification usage
        self.flash_specs = flash_specs;
        self.compound_spec = compound_spec;
        self.flow_rate_spec = flow_rate_spec;
        //apply values
        if use_mass_vapor_fraction {
            vapor_fraction_spec.resize(3);
            vapor_fraction_spec
                .get_mut(0)
                .unwrap()
                .set(&MATERIAL_CONSTANTS.phase_fraction);
            vapor_fraction_spec
                .get_mut(1)
                .unwrap()
                .set(&MATERIAL_CONSTANTS.mass_basis);
            let pp = self.property_package.read().unwrap();
            vapor_fraction_spec
                .get_mut(2)
                .unwrap()
                .set(&pp.phase_labels[pp.vapor_phase_index.unwrap()]);
            flash_conditions.push(&vapor_fraction_spec);
        } else if use_molar_vapor_fraction {
            vapor_fraction_spec.resize(3);
            vapor_fraction_spec
                .get_mut(0)
                .unwrap()
                .set(&MATERIAL_CONSTANTS.phase_fraction);
            vapor_fraction_spec
                .get_mut(1)
                .unwrap()
                .set(&MATERIAL_CONSTANTS.mole_basis);
            let pp = self.property_package.read().unwrap();
            vapor_fraction_spec
                .get_mut(2)
                .unwrap()
                .set(&pp.phase_labels[pp.vapor_phase_index.unwrap()]);
            flash_conditions.push(&vapor_fraction_spec);
        }
        let mut is_complete = self.flash_specs.len() == 2
            && self.compound_spec != CompoundSpecUsed::None
            && self.flow_rate_spec != FlowRateSpecUsed::None;
        match self.compound_spec {
            CompoundSpecUsed::MoleFraction | CompoundSpecUsed::MassFraction => {
                //count the number of missing values
                let mut missing_count = 0;
                let mut sum = 0.0;
                for val in compound_values.iter() {
                    if val.is_nan() {
                        missing_count += 1;
                    } else {
                        sum += *val;
                    }
                }
                match missing_count {
                    0 => {
                        //normalize values to sum to 1
                        let normalization_factor = 1.0 / sum;
                        for val in compound_values.iter_mut() {
                            *val *= normalization_factor;
                        }
                        //also normalize the specs themselves
                        for flash_spec in self.specifications.iter_mut() {
                            match flash_spec.specification_type {
                                StreamSpecificationType::MolarFraction(_)
                                | StreamSpecificationType::MassFraction(_) => {
                                    flash_spec.value *= normalization_factor;
                                }
                                _ => {}
                            }
                        }
                    }
                    1 => {
                        //can calculate missing value
                        let mut missing_value = 1.0 - sum;
                        if missing_value >= -1e-12 {
                            if missing_value < 0.0 {
                                missing_value = 0.0;
                            }
                            for val in compound_values.iter_mut() {
                                if val.is_nan() {
                                    *val = missing_value;
                                    break;
                                }
                            }
                        } else {
                            //invalid values, cannot calculate
                            is_complete = false;
                        }
                    }
                    _ => {
                        //too many missing values to calculate
                        is_complete = false;
                    }
                }
                self.overall.value_map.insert_from_cape_string(
                    &MATERIAL_CONSTANTS.fraction,
                    PropertyValue {
                        value: PropertyValueData::Vector(compound_values),
                        basis: if self.compound_spec == CompoundSpecUsed::MoleFraction {
                            PropertyBasis::Mole
                        } else {
                            PropertyBasis::Mass
                        },
                    },
                );
            }
            CompoundSpecUsed::MassCompoundFlow | CompoundSpecUsed::MolarCompoundFlow => {
                //treat missing as zero
                let mut total_flow_value = 0.0;
                for val in compound_values.iter_mut() {
                    if val.is_nan() {
                        *val = 0.0;
                    }
                    total_flow_value += *val;
                }
                //set total flow
                self.overall.value_map.insert_from_cape_string(
                    &MATERIAL_CONSTANTS.total_flow,
                    PropertyValue {
                        value: PropertyValueData::Scalar(total_flow_value),
                        basis: if self.compound_spec == CompoundSpecUsed::MolarCompoundFlow {
                            PropertyBasis::Mole
                        } else {
                            PropertyBasis::Mass
                        },
                    },
                );
                //normalize composition
                let normalization_factor = 1.0 / total_flow_value;
                for val in compound_values.iter_mut() {
                    *val *= normalization_factor;
                }
                self.overall.value_map.insert_from_cape_string(
                    &MATERIAL_CONSTANTS.fraction,
                    PropertyValue {
                        value: PropertyValueData::Vector(compound_values),
                        basis: if self.compound_spec == CompoundSpecUsed::MolarCompoundFlow {
                            PropertyBasis::Mole
                        } else {
                            PropertyBasis::Mass
                        },
                    },
                );
            }
            CompoundSpecUsed::None => {
                //no compound spec, do nothing
            }
        }
        //if complete, flash
        if is_complete {
            //mark all phases as present
            for phase in self.phases.iter_mut() {
                phase.present = true;
            }
            //perform flash calculation to determine phase split and properties
            match self.ensure_context_material() {
                Ok(_) => {}
                Err(e) => {
                    self.specification_error = Some(format!("Context failure: {}", e));
                    return;
                }
            };
            let is_feed = self.is_feed;
            self.is_feed = false; //temporarily set to false to avoid feed flash spec validation, will reset after flash
            match self.calc_equilibrium_internal(
                &CapeArrayStringInFromProvider::from(flash_conditions[0]).as_cape_array_string_in(),
                &CapeArrayStringInFromProvider::from(flash_conditions[1]).as_cape_array_string_in(),
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.unspecified)
                    .as_cape_string_in(),
            ) {
                Ok(_) => {
                    //flash calculation successful, continue
                    self.is_specified = true;
                    self.calc_properties_after_flash();
                }
                Err(e) => {
                    self.specification_error = Some(format!("Flash failure: {}", e));
                    self.is_feed = is_feed; //reset feed status
                    return;
                }
            }
            self.is_feed = is_feed; //reset feed status
        }
    }

    /// Returns all raw specification entries currently stored on the stream.
    ///
    /// # Returns
    /// Borrowed vector of specifications.
    pub fn get_specifications(&self) -> &Vec<StreamSpecification> {
        &self.specifications
    }

    /// Replaces all specification entries and re-applies them.
    ///
    /// # Parameters
    /// - `specs`: New full specification list.
    pub fn set_specifications(&mut self, specs: Vec<StreamSpecification>) {
        self.specifications = specs;
        self.apply_specifications();
    }

    /// Gets the current value and availability of one flash-like specification.
    ///
    /// # Parameters
    /// - `pp`: Property package metadata/cache for phase and compound context.
    /// - `spec_type`: Specification to query.
    /// - `flash_specifications`: Already selected flash specifications used to
    ///   determine whether this specification can still be selected.
    ///
    /// # Returns
    /// Tuple `(value, allowed_as_spec)` where:
    /// - `value` is the current value if derivable,
    /// - `allowed_as_spec` indicates whether the UI may offer this spec.
    pub fn get_specification_value(
        &self,
        pp: &PropertyPackageInstance,
        spec_type: StreamSpecificationType,
        flash_specifications: &ArrayVec<StreamSpecificationType, 2>,
    ) -> (Option<f64>, bool) {
        match spec_type {
            StreamSpecificationType::Temperature
            | StreamSpecificationType::Pressure
            | StreamSpecificationType::MolarEnthalpy
            | StreamSpecificationType::MolarEntropy
            | StreamSpecificationType::MolarVolume
            | StreamSpecificationType::SpecificEnthalpy
            | StreamSpecificationType::SpecificEntropy
            | StreamSpecificationType::SpecificVolume
            | StreamSpecificationType::MolarDensity
            | StreamSpecificationType::MassDensity
            | StreamSpecificationType::MolarVaporFraction
            | StreamSpecificationType::MassVaporFraction => {
                //this is a flash spec; check if present
                for spec in flash_specifications.iter() {
                    if spec == &spec_type {
                        //spec is present, return value
                        for s in self.specifications.iter() {
                            if s.specification_type == spec_type {
                                return (Some(s.value), true);
                            }
                        }
                        unreachable!();
                    }
                }
                let mut excluded_specs: ArrayVec<StreamSpecificationType, 3> = ArrayVec::new();
                let (prop_name, basis) = match spec_type {
                    StreamSpecificationType::Temperature => {
                        (&MATERIAL_CONSTANTS.temperature, PropertyBasis::None)
                    }
                    StreamSpecificationType::Pressure => {
                        (&MATERIAL_CONSTANTS.pressure, PropertyBasis::None)
                    }
                    StreamSpecificationType::MolarEnthalpy => {
                        excluded_specs.push(StreamSpecificationType::SpecificEnthalpy);
                        (&MATERIAL_CONSTANTS.enthalpy, PropertyBasis::Mole)
                    }
                    StreamSpecificationType::MolarEntropy => {
                        excluded_specs.push(StreamSpecificationType::SpecificEntropy);
                        (&MATERIAL_CONSTANTS.entropy, PropertyBasis::Mole)
                    }
                    StreamSpecificationType::MolarVolume => {
                        excluded_specs.push(StreamSpecificationType::SpecificVolume);
                        excluded_specs.push(StreamSpecificationType::MassDensity);
                        excluded_specs.push(StreamSpecificationType::MolarDensity);
                        (&MATERIAL_CONSTANTS.volume, PropertyBasis::Mole)
                    }
                    StreamSpecificationType::SpecificEnthalpy => {
                        excluded_specs.push(StreamSpecificationType::MolarEnthalpy);
                        (&MATERIAL_CONSTANTS.enthalpy, PropertyBasis::Mass)
                    }
                    StreamSpecificationType::SpecificEntropy => {
                        excluded_specs.push(StreamSpecificationType::MolarEntropy);
                        (&MATERIAL_CONSTANTS.entropy, PropertyBasis::Mass)
                    }
                    StreamSpecificationType::SpecificVolume => {
                        excluded_specs.push(StreamSpecificationType::MolarVolume);
                        excluded_specs.push(StreamSpecificationType::MassDensity);
                        excluded_specs.push(StreamSpecificationType::MolarDensity);
                        (&MATERIAL_CONSTANTS.volume, PropertyBasis::Mass)
                    }
                    StreamSpecificationType::MolarDensity => {
                        excluded_specs.push(StreamSpecificationType::MassDensity);
                        excluded_specs.push(StreamSpecificationType::MolarVolume);
                        excluded_specs.push(StreamSpecificationType::SpecificVolume);
                        (&MATERIAL_CONSTANTS.density, PropertyBasis::Mole)
                    }
                    StreamSpecificationType::MassDensity => {
                        excluded_specs.push(StreamSpecificationType::MolarDensity);
                        excluded_specs.push(StreamSpecificationType::MolarVolume);
                        excluded_specs.push(StreamSpecificationType::SpecificVolume);
                        (&MATERIAL_CONSTANTS.density, PropertyBasis::Mass)
                    }
                    StreamSpecificationType::MolarVaporFraction
                    | StreamSpecificationType::MassVaporFraction => {
                        let vapor_fraction = if self.is_specified {
                            //get the vapor phase
                            match pp.vapor_phase_index {
                                Some(vapor_index) => {
                                    let mut value = CapeArrayRealScalar::new();
                                    match self.get_phase_prop(
                                        pp,
                                        &self.phases[vapor_index],
                                        &CapeStringInFromProvider::from(
                                            &MATERIAL_CONSTANTS.phase_fraction,
                                        )
                                        .as_cape_string_in(),
                                        if spec_type == StreamSpecificationType::MolarVaporFraction
                                        {
                                            PropertyBasis::Mole
                                        } else {
                                            PropertyBasis::Mass
                                        },
                                        &mut CapeArrayRealOutFromProvider::from(&mut value)
                                            .as_cape_array_real_out(),
                                    ) {
                                        Ok(()) => Some(value.value()),
                                        Err(_) => Some(0.0), //zero vapor fraction if no vapor
                                    }
                                }
                                None => None,
                            }
                        } else {
                            //no value
                            None
                        };
                        //allow spec only in case the no other vapor fraction flash spec is used
                        return (
                            vapor_fraction,
                            flash_specifications.len() < 2
                                && !flash_specifications
                                    .iter()
                                    .any(|s| excluded_specs.contains(s)),
                        );
                    }
                    _ => unreachable!(),
                };
                let allow_as_spec = flash_specifications.len() < 2
                    && !flash_specifications
                        .iter()
                        .any(|s| excluded_specs.contains(s));
                let mut value = CapeArrayRealScalar::new();
                let prop_value = match self.get_phase_prop(
                    pp,
                    &self.overall,
                    &CapeStringInFromProvider::from(prop_name).as_cape_string_in(),
                    basis,
                    &mut CapeArrayRealOutFromProvider::from(&mut value).as_cape_array_real_out(),
                ) {
                    Ok(()) => Some(value.value()),
                    Err(_) => None,
                };
                (prop_value, allow_as_spec)
            }
            _ => unreachable!(),
        }
    }

    /// Builds per-compound and total-flow based on currently used specifications
    ///
    /// # Parameters
    /// - `pp`: Property package metadata and compound indexing.
    /// - `compound_spec`: Currently active compound specification family.
    /// - `flow_rate_spec`: Currently active flow-rate specification family.
    ///
    /// # Returns
    /// Tuple containing:
    /// - mole-fraction values and allowed flags,
    /// - mass-fraction values and allowed flags,
    /// - molar component flow values and allowed flags,
    /// - mass component flow values and allowed flags,
    /// - total molar flow `(value, allowed)`,
    /// - total mass flow `(value, allowed)`.
    pub fn get_compound_specification_values(
        &self,
        pp: &PropertyPackageInstance,
        compound_spec: &CompoundSpecUsed,
        flow_rate_spec: &FlowRateSpecUsed,
    ) -> (
        Vec<(Option<f64>, bool)>, //mole fraction
        Vec<(Option<f64>, bool)>, //mass fraction
        Vec<(Option<f64>, bool)>, //molar component flow rate
        Vec<(Option<f64>, bool)>, //mass component flow rate
        (Option<f64>, bool),      //total molar flow rate
        (Option<f64>, bool),      //total mass flow rate
    ) {
        let mut mole_fractions = Vec::new();
        let mut mass_fractions = Vec::new();
        let mut molar_component_flow_rates = Vec::new();
        let mut mass_component_flow_rates = Vec::new();
        let mut total_molar_flow_rate = (None, false);
        let mut total_mass_flow_rate = (None, false);
        //default: no values, not allowed as spec
        mole_fractions.resize(pp.num_compounds, (None, false));
        mass_fractions.resize(pp.num_compounds, (None, false));
        molar_component_flow_rates.resize(pp.num_compounds, (None, false));
        mass_component_flow_rates.resize(pp.num_compounds, (None, false));
        //see what is specified
        let mut get_mole_fractions = true;
        let mut get_mass_fractions = true;
        let mut get_molar_component_flow_rates = true;
        let mut get_mass_component_flow_rates = true;
        let mut get_total_molar_flow_rate = true;
        let mut get_total_mass_flow_rate = true;
        match compound_spec {
            CompoundSpecUsed::None => {
                //all are specs
                for (_, is_spec) in mole_fractions.iter_mut() {
                    *is_spec = true;
                }
                for (_, is_spec) in mass_fractions.iter_mut() {
                    *is_spec = true;
                }
                //unless we already have a flow rate spec
                let can_have_flow_rate_spec = *flow_rate_spec == FlowRateSpecUsed::None;
                for (_, is_spec) in molar_component_flow_rates.iter_mut() {
                    *is_spec = can_have_flow_rate_spec;
                }
                for (_, is_spec) in mass_component_flow_rates.iter_mut() {
                    *is_spec = can_have_flow_rate_spec;
                }
            }
            CompoundSpecUsed::MoleFraction => {
                get_mole_fractions = false;
                for spec in self.specifications.iter() {
                    if let StreamSpecificationType::MolarFraction(compound_id) =
                        &spec.specification_type
                        && let Some(compound_index) = pp.get_compound_index(compound_id) {
                            mole_fractions[*compound_index ] = (Some(spec.value), true);
                        }
                }
                for (_, is_spec) in mole_fractions.iter_mut() {
                    *is_spec = true;
                }
            }
            CompoundSpecUsed::MassFraction => {
                get_mass_fractions = false;
                for spec in self.specifications.iter() {
                    if let StreamSpecificationType::MassFraction(compound_id) =
                        &spec.specification_type
                        && let Some(compound_index) = pp.get_compound_index(compound_id) {
                            mass_fractions[*compound_index ] = (Some(spec.value), true);
                        }
                }
                for (_, is_spec) in mass_fractions.iter_mut() {
                    *is_spec = true;
                }
            }
            CompoundSpecUsed::MolarCompoundFlow => {
                get_molar_component_flow_rates = false;
                for spec in self.specifications.iter() {
                    if let StreamSpecificationType::MolarCompoundFlowRate(compound_id) =
                        &spec.specification_type
                        && let Some(compound_index) = pp.get_compound_index(compound_id) {
                            molar_component_flow_rates[*compound_index ] =
                                (Some(spec.value), true);
                        }
                }
                for (_, is_spec) in molar_component_flow_rates.iter_mut() {
                    *is_spec = true;
                }
            }
            CompoundSpecUsed::MassCompoundFlow => {
                get_mass_component_flow_rates = false;
                for spec in self.specifications.iter() {
                    if let StreamSpecificationType::MassCompoundFlowRate(compound_id) =
                        &spec.specification_type
                        && let Some(compound_index) = pp.get_compound_index(compound_id) {
                            mass_component_flow_rates[*compound_index ] =
                                (Some(spec.value), true);
                        }
                }
            }
        };
        match flow_rate_spec {
            FlowRateSpecUsed::MassFlow => {
                get_total_mass_flow_rate = false;
                for spec in self.specifications.iter() {
                    if let StreamSpecificationType::MassFlowRate = spec.specification_type {
                        total_mass_flow_rate = (Some(spec.value), true);
                    }
                }
            }
            FlowRateSpecUsed::MolarFlow => {
                get_total_molar_flow_rate = false;
                for spec in self.specifications.iter() {
                    if let StreamSpecificationType::MolarFlowRate = spec.specification_type {
                        total_molar_flow_rate = (Some(spec.value), true);
                    }
                }
            }
            FlowRateSpecUsed::MassCompoundFlow | FlowRateSpecUsed::MolarCompoundFlow => {}
            FlowRateSpecUsed::None => {
                //both are specs
                total_mass_flow_rate = (None, true);
                total_molar_flow_rate = (None, true);
                //neither can be calculated
                get_total_mass_flow_rate = false;
                get_total_molar_flow_rate = false;
            }
        };
        if get_mole_fractions {
            let mut mole_fraction_values = CapeArrayRealVec::new();
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.fraction).as_cape_string_in(),
                PropertyBasis::Mole,
                &mut CapeArrayRealOutFromProvider::from(&mut mole_fraction_values)
                    .as_cape_array_real_out(),
            ) {
                for (index, (val, _)) in mole_fractions.iter_mut().enumerate() {
                    *val = if f64::is_nan(mole_fraction_values[index]) {
                        None
                    } else {
                        Some(mole_fraction_values[index])
                    };
                }
            }
        }
        if get_mass_fractions {
            let mut mass_fraction_values = CapeArrayRealVec::new();
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.fraction).as_cape_string_in(),
                PropertyBasis::Mass,
                &mut CapeArrayRealOutFromProvider::from(&mut mass_fraction_values)
                    .as_cape_array_real_out(),
            ) {
                for (index, (val, _)) in mass_fractions.iter_mut().enumerate() {
                    *val = if f64::is_nan(mass_fraction_values[index]) {
                        None
                    } else {
                        Some(mass_fraction_values[index])
                    };
                }
            }
        }
        if get_molar_component_flow_rates {
            let mut molar_component_flow_values = CapeArrayRealVec::new();
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.flow).as_cape_string_in(),
                PropertyBasis::Mole,
                &mut CapeArrayRealOutFromProvider::from(&mut molar_component_flow_values)
                    .as_cape_array_real_out(),
            ) {
                for (index, (val, _)) in molar_component_flow_rates.iter_mut().enumerate() {
                    *val = if f64::is_nan(molar_component_flow_values[index]) {
                        None
                    } else {
                        Some(molar_component_flow_values[index])
                    };
                }
            }
        }
        if get_mass_component_flow_rates {
            let mut mass_component_flow_values = CapeArrayRealVec::new();
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.flow).as_cape_string_in(),
                PropertyBasis::Mass,
                &mut CapeArrayRealOutFromProvider::from(&mut mass_component_flow_values)
                    .as_cape_array_real_out(),
            ) {
                for (index, (val, _)) in mass_component_flow_rates.iter_mut().enumerate() {
                    *val = if f64::is_nan(mass_component_flow_values[index]) {
                        None
                    } else {
                        Some(mass_component_flow_values[index])
                    };
                }
            }
        }
        if get_total_molar_flow_rate {
            let mut total_molar_flow_value = CapeArrayRealScalar::new();
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.total_flow).as_cape_string_in(),
                PropertyBasis::Mole,
                &mut CapeArrayRealOutFromProvider::from(&mut total_molar_flow_value)
                    .as_cape_array_real_out(),
            ) {
                total_molar_flow_rate.0 = Some(total_molar_flow_value.value());
            }
        }
        if get_total_mass_flow_rate {
            let mut total_mass_flow_value = CapeArrayRealScalar::new();
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.total_flow).as_cape_string_in(),
                PropertyBasis::Mass,
                &mut CapeArrayRealOutFromProvider::from(&mut total_mass_flow_value)
                    .as_cape_array_real_out(),
            ) {
                total_mass_flow_rate.0 = Some(total_mass_flow_value.value());
            }
        }
        //return all in one go
        (
            mole_fractions,
            mass_fractions,
            molar_component_flow_rates,
            mass_component_flow_rates,
            total_molar_flow_rate,
            total_mass_flow_rate,
        )
    }

    /// Calculates post-flash display properties from phase results.
    ///
    /// Updates phase and overall cached properties in-place.
    pub fn calc_properties_after_flash(&mut self) {
        //calculate values for enthalpy, entropy, volume, density and fugacity based on phase split and values for stream display; if exposed by the property package
        // here we make the additional assumption that density always equals 1/volume
        let pp = self.property_package.read().unwrap();
        let mut enthalpy_index = None;
        let mut entropy_index = None;
        let mut volume_index = None;
        let mut fugacity_index = None;
        let mut count = 0;
        if pp
            .flash_conditions
            .contains(&StreamSpecificationType::MolarEnthalpy)
        {
            enthalpy_index = Some(count);
            count += 1;
        }
        if pp
            .flash_conditions
            .contains(&StreamSpecificationType::MolarEntropy)
        {
            entropy_index = Some(count);
            count += 1;
        }
        if pp
            .flash_conditions
            .contains(&StreamSpecificationType::MolarVolume)
        {
            volume_index = Some(count);
            count += 1;
        }
        if pp.supports_fugacity {
            fugacity_index = Some(count);
            count += 1;
        }
        if count > 0 {
            let pp = self.property_package.read().unwrap();
            let mut prop_list = cobia::CapeArrayStringVec::new();
            prop_list.resize(count);
            if let Some(enthalpy_index) = enthalpy_index {
                prop_list
                    .get_mut(enthalpy_index)
                    .unwrap()
                    .set(&MATERIAL_CONSTANTS.enthalpy);
            }
            if let Some(entropy_index) = entropy_index {
                prop_list
                    .get_mut(entropy_index)
                    .unwrap()
                    .set(&MATERIAL_CONSTANTS.entropy);
            }
            if let Some(volume_index) = volume_index {
                prop_list
                    .get_mut(volume_index)
                    .unwrap()
                    .set(&MATERIAL_CONSTANTS.volume);
            }
            if let Some(fugacity_index) = fugacity_index {
                prop_list
                    .get_mut(fugacity_index)
                    .unwrap()
                    .set(&MATERIAL_CONSTANTS.fugacity);
            }
            let mut overall_values = vec![0.0; count];
            //use the fact that we are still the context material after the CalcEquilibrium call
            for phase_index in 0..self.phases.len() {
                let phase_fraction;
                {
                    let phase = &self.phases[phase_index];
                    if !phase.present {
                        continue;
                    }
                    //get molar phase fraction
                    let (mut phase_frac, basis) =
                        match phase.value_map.get(&MATERIAL_CONSTANTS.phase_fraction) {
                            Some(prop_value) => {
                                match &prop_value.value {
                                    PropertyValueData::Scalar(val) => {
                                        (*val, prop_value.basis.clone())
                                    }
                                    PropertyValueData::Vector(_vec) => {
                                        assert!(false);
                                        (f64::NAN, PropertyBasis::None) //should not happen, continue with NaN values
                                    }
                                }
                            }
                            None => {
                                //no phase fraction, skip
                                assert!(false);
                                (f64::NAN, PropertyBasis::None) //should not happen, continue with NaN values
                            }
                        };
                    if basis == PropertyBasis::Mass {
                        //basis conversion
                        let overall_mw = self.mix_molecular_weight(&pp, &self.overall).unwrap_or(f64::NAN);
                        let phase_mw = self.mix_molecular_weight(&pp, phase).unwrap_or(f64::NAN);
                        phase_frac *= overall_mw / phase_mw ;
                    }
                    phase_fraction = phase_frac;
                }
                match pp
                    .pp_property_routine
                    .calc_single_phase_prop(&prop_list, &pp.phase_labels[phase_index])
                {
                    Ok(_) => {
                        let phase = &self.phases[phase_index];
                        for (i,overall_value) in overall_values.iter_mut().enumerate() {
                            match phase.value_map.get(&prop_list[i]) {
                                Some(prop_value) => {
                                    let val = match &prop_value.value {
                                        PropertyValueData::Scalar(val) => {
                                            match prop_value.basis {
                                                PropertyBasis::Mole => *val,
                                                PropertyBasis::Mass => {
                                                    //basis conversion
                                                    let mw = self
                                                        .mix_molecular_weight(&pp, phase).unwrap_or(f64::NAN);
                                                    val / mw
                                                }
                                                PropertyBasis::None => {
                                                    //should not happen
                                                    f64::NAN
                                                }
                                            }
                                        }
                                        PropertyValueData::Vector(_) => f64::NAN,
                                    };
                                    *overall_value += val * phase_fraction;
                                }
                                None => {
                                    //property not found for phase, skip
                                    *overall_value = f64::NAN;
                                }
                            }
                        }
                    }
                    Err(_) => {
                        //ignore
                        for overall_value in overall_values.iter_mut() {
                            *overall_value = f64::NAN;
                        }
                    }
                }
            }
            //bring phases back at equilibrium
            for phase in self.phases.iter_mut() {
                if phase.present {
                    phase.status = cape_open_1_2::CapePhaseStatus::CapeAtequilibrium; //mark phase as at least equilibrium, as we have calculated properties based on flash conditions; this is used for stream display purposes
                }
            }
            //store overall values
            if let Some(enthalpy_index) = enthalpy_index {
                self.overall.value_map.insert_from_cape_string(
                    &MATERIAL_CONSTANTS.enthalpy,
                    PropertyValue {
                        value: PropertyValueData::Scalar(overall_values[enthalpy_index]),
                        basis: PropertyBasis::Mole,
                    },
                );
            }
            if let Some(entropy_index) = entropy_index {
                self.overall.value_map.insert_from_cape_string(
                    &MATERIAL_CONSTANTS.entropy,
                    PropertyValue {
                        value: PropertyValueData::Scalar(overall_values[entropy_index]),
                        basis: PropertyBasis::Mole,
                    },
                );
            }
            if let Some(volume_index) = volume_index {
                self.overall.value_map.insert_from_cape_string(
                    &MATERIAL_CONSTANTS.volume,
                    PropertyValue {
                        value: PropertyValueData::Scalar(overall_values[volume_index]),
                        basis: PropertyBasis::Mole,
                    },
                );
                //also set density as 1/volume
                self.overall.value_map.insert_from_cape_string(
                    &MATERIAL_CONSTANTS.density,
                    PropertyValue {
                        value: PropertyValueData::Scalar(1.0 / overall_values[volume_index]),
                        basis: PropertyBasis::Mole,
                    },
                );
            }
        }
    }

    /// Indicates whether a phase is currently marked as present.
    ///
    /// # Parameters
    /// - `phase_index`: Index into the property-package phase list.
    ///
    /// # Returns
    /// `true` if the phase is present in the current material state.
    pub fn phase_is_present(&self, phase_index: usize) -> bool {
        self.phases[phase_index].present
    }

    /// Returns an overall scalar value for a requested stream specification type.
    ///
    /// # Parameters
    /// - `pp`: Property package metadata/cache.
    /// - `spec_type`: Target value to extract.
    ///
    /// # Returns
    /// - `Some(value)` when available.
    /// - `None` when the stream is incomplete or the value is unavailable.
    pub fn get_overall_value(
        &self,
        pp: &PropertyPackageInstance,
        spec_type: StreamSpecificationType,
    ) -> Option<f64> {
        if !self.is_specified {
            return None;
        }
        match spec_type {
            StreamSpecificationType::Temperature
            | StreamSpecificationType::Pressure
            | StreamSpecificationType::MolarEnthalpy
            | StreamSpecificationType::MolarEntropy
            | StreamSpecificationType::MolarVolume
            | StreamSpecificationType::SpecificEnthalpy
            | StreamSpecificationType::SpecificEntropy
            | StreamSpecificationType::SpecificVolume
            | StreamSpecificationType::MolarDensity
            | StreamSpecificationType::MassDensity
            | StreamSpecificationType::MolarVaporFraction
            | StreamSpecificationType::MassVaporFraction => {
                let (prop_name, basis) = match spec_type {
                    StreamSpecificationType::Temperature => {
                        (&MATERIAL_CONSTANTS.temperature, PropertyBasis::None)
                    }
                    StreamSpecificationType::Pressure => {
                        (&MATERIAL_CONSTANTS.pressure, PropertyBasis::None)
                    }
                    StreamSpecificationType::MolarEnthalpy => {
                        (&MATERIAL_CONSTANTS.enthalpy, PropertyBasis::Mole)
                    }
                    StreamSpecificationType::MolarEntropy => {
                        (&MATERIAL_CONSTANTS.entropy, PropertyBasis::Mole)
                    }
                    StreamSpecificationType::MolarVolume => {
                        (&MATERIAL_CONSTANTS.volume, PropertyBasis::Mole)
                    }
                    StreamSpecificationType::SpecificEnthalpy => {
                        (&MATERIAL_CONSTANTS.enthalpy, PropertyBasis::Mass)
                    }
                    StreamSpecificationType::SpecificEntropy => {
                        (&MATERIAL_CONSTANTS.entropy, PropertyBasis::Mass)
                    }
                    StreamSpecificationType::SpecificVolume => {
                        (&MATERIAL_CONSTANTS.volume, PropertyBasis::Mass)
                    }
                    StreamSpecificationType::MolarDensity => {
                        (&MATERIAL_CONSTANTS.density, PropertyBasis::Mole)
                    }
                    StreamSpecificationType::MassDensity => {
                        (&MATERIAL_CONSTANTS.density, PropertyBasis::Mass)
                    }
                    StreamSpecificationType::MolarVaporFraction
                    | StreamSpecificationType::MassVaporFraction => {
                        //get the vapor phase
                        return match pp.vapor_phase_index {
                            Some(vapor_index) => {
                                let mut value = CapeArrayRealScalar::new();
                                match self.get_phase_prop(
                                    pp,
                                    &self.phases[vapor_index],
                                    &CapeStringInFromProvider::from(
                                        &MATERIAL_CONSTANTS.phase_fraction,
                                    )
                                    .as_cape_string_in(),
                                    if spec_type == StreamSpecificationType::MolarVaporFraction {
                                        PropertyBasis::Mole
                                    } else {
                                        PropertyBasis::Mass
                                    },
                                    &mut CapeArrayRealOutFromProvider::from(&mut value)
                                        .as_cape_array_real_out(),
                                ) {
                                    Ok(()) => Some(value.value()),
                                    Err(_) => Some(0.0), //no vapor means zero
                                }
                            }
                            None => None, //should not ask
                        };
                    }
                    _ => unreachable!(),
                };
                let mut value = CapeArrayRealScalar::new();
                match self.get_phase_prop(
                    pp,
                    &self.overall,
                    &CapeStringInFromProvider::from(prop_name).as_cape_string_in(),
                    basis,
                    &mut CapeArrayRealOutFromProvider::from(&mut value).as_cape_array_real_out(),
                ) {
                    Ok(()) => Some(value.value()),
                    Err(_) => None,
                }
            }
            _ => unreachable!(),
        }
    }

    /// Returns overall composition and flow values in both mole and mass bases.
    ///
    /// # Parameters
    /// - `pp`: Property package metadata (compound count and conversions).
    ///
    /// # Returns
    /// Tuple of:
    /// - mole fractions,
    /// - mass fractions,
    /// - molar component flow rates,
    /// - mass component flow rates,
    /// - total molar flow,
    /// - total mass flow.
    pub fn get_overall_compound_values(
        &self,
        pp: &PropertyPackageInstance,
    ) -> (
        Vec<Option<f64>>, //mole fraction
        Vec<Option<f64>>, //mass fraction
        Vec<Option<f64>>, //molar component flow rate
        Vec<Option<f64>>, //mass component flow rate
        Option<f64>,      //total molar flow rate
        Option<f64>,      //total mass flow rate
    ) {
        let mut mole_fractions = Vec::new();
        let mut mass_fractions = Vec::new();
        let mut molar_component_flow_rates = Vec::new();
        let mut mass_component_flow_rates = Vec::new();
        let mut total_molar_flow_rate = None;
        let mut total_mass_flow_rate = None;
        if self.is_specified {
            let mut scalar_value = CapeArrayRealScalar::new();
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.total_flow).as_cape_string_in(),
                PropertyBasis::Mole,
                &mut CapeArrayRealOutFromProvider::from(&mut scalar_value).as_cape_array_real_out(),
            ) {
                total_molar_flow_rate = Some(scalar_value.value());
            };
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.total_flow).as_cape_string_in(),
                PropertyBasis::Mass,
                &mut CapeArrayRealOutFromProvider::from(&mut scalar_value).as_cape_array_real_out(),
            ) {
                total_mass_flow_rate = Some(scalar_value.value());
            };
            let mut vector_value = CapeArrayRealVec::new();
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.fraction).as_cape_string_in(),
                PropertyBasis::Mole,
                &mut CapeArrayRealOutFromProvider::from(&mut vector_value).as_cape_array_real_out(),
            ) {
                for index in 0..vector_value.size() {
                    let val = vector_value[index];
                    mole_fractions.push(if f64::is_nan(val) { None } else { Some(val) });
                }
            };
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.fraction).as_cape_string_in(),
                PropertyBasis::Mass,
                &mut CapeArrayRealOutFromProvider::from(&mut vector_value).as_cape_array_real_out(),
            ) {
                for index in 0..vector_value.size() {
                    let val = vector_value[index];
                    mass_fractions.push(if f64::is_nan(val) { None } else { Some(val) });
                }
            };
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.flow).as_cape_string_in(),
                PropertyBasis::Mole,
                &mut CapeArrayRealOutFromProvider::from(&mut vector_value).as_cape_array_real_out(),
            ) {
                for index in 0..vector_value.size() {
                    let val = vector_value[index];
                    molar_component_flow_rates.push(if f64::is_nan(val) {
                        None
                    } else {
                        Some(val)
                    });
                }
            };
            if let Ok(()) = self.get_phase_prop(
                pp,
                &self.overall,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.flow).as_cape_string_in(),
                PropertyBasis::Mass,
                &mut CapeArrayRealOutFromProvider::from(&mut vector_value).as_cape_array_real_out(),
            ) {
                for index in 0..vector_value.size() {
                    let val = vector_value[index];
                    mass_component_flow_rates.push(if f64::is_nan(val) {
                        None
                    } else {
                        Some(val)
                    });
                }
            }
        } else {
            //not specified, return empty results
            mole_fractions.resize(pp.num_compounds, None);
            mass_fractions.resize(pp.num_compounds, None);
            molar_component_flow_rates.resize(pp.num_compounds, None);
            mass_component_flow_rates.resize(pp.num_compounds, None);
        }
        (
            mole_fractions,
            mass_fractions,
            molar_component_flow_rates,
            mass_component_flow_rates,
            total_molar_flow_rate,
            total_mass_flow_rate,
        )
    }

    /// Returns phase-level values for one phase in both mole and mass bases.
    ///
    /// # Parameters
    /// - `pp`: Property package metadata.
    /// - `phase_index`: Phase index in property-package ordering.
    ///
    /// # Returns
    /// Tuple containing:
    /// - molar phase fraction,
    /// - mass phase fraction,
    /// - molar composition,
    /// - mass composition,
    /// - molar enthalpy,
    /// - specific enthalpy,
    /// - molar entropy,
    /// - specific entropy,
    /// - molar volume,
    /// - specific volume,
    /// - molar density,
    /// - mass density,
    /// - fugacity vector.
    pub fn get_phase_values(
        &self,
        pp: &PropertyPackageInstance,
        phase_index: usize,
    ) -> (
        Option<f64>,      //molar phase fraction
        Option<f64>,      //mass phase fraction
        Option<Vec<f64>>, //molar composition
        Option<Vec<f64>>, //mass composition
        Option<f64>,      //molar enthalpy
        Option<f64>,      //specific enthalpy
        Option<f64>,      //molar entropy
        Option<f64>,      //specific entropy
        Option<f64>,      //molar volume
        Option<f64>,      //specific volume
        Option<f64>,      //molar density
        Option<f64>,      //mass density
        Option<Vec<f64>>, //fugacity
    ) {
        let mut molar_phase_fraction = None;
        let mut mass_phase_fraction = None;
        let mut molar_composition = None;
        let mut mass_composition = None;
        let mut molar_enthalpy = None;
        let mut specific_enthalpy = None;
        let mut molar_entropy = None;
        let mut specific_entropy = None;
        let mut molar_volume = None;
        let mut specific_volume = None;
        let mut molar_density = None;
        let mut mass_density = None;
        let mut fugacity = None;
        if self.is_specified {
            let phase = &self.phases[phase_index];
            let mut scalar_value = CapeArrayRealScalar::new();
            let mut vector_value = CapeArrayRealVec::new();
            if let Ok(()) = self.get_phase_prop(
                pp,
                phase,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.phase_fraction)
                    .as_cape_string_in(),
                PropertyBasis::Mole,
                &mut CapeArrayRealOutFromProvider::from(&mut scalar_value).as_cape_array_real_out(),
            ) {
                molar_phase_fraction = Some(scalar_value.value());
            };
            if let Ok(()) = self.get_phase_prop(
                pp,
                phase,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.phase_fraction)
                    .as_cape_string_in(),
                PropertyBasis::Mass,
                &mut CapeArrayRealOutFromProvider::from(&mut scalar_value).as_cape_array_real_out(),
            ) {
                mass_phase_fraction = Some(scalar_value.value());
            };
            if let Ok(()) = self.get_phase_prop(
                pp,
                phase,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.fraction).as_cape_string_in(),
                PropertyBasis::Mole,
                &mut CapeArrayRealOutFromProvider::from(&mut vector_value).as_cape_array_real_out(),
            ) {
                molar_composition = Some(vector_value.as_vec().clone());
            };
            if let Ok(()) = self.get_phase_prop(
                pp,
                phase,
                &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.fraction).as_cape_string_in(),
                PropertyBasis::Mass,
                &mut CapeArrayRealOutFromProvider::from(&mut vector_value).as_cape_array_real_out(),
            ) {
                mass_composition = Some(vector_value.as_vec().clone());
            };
            if pp
                .flash_conditions
                .contains(&StreamSpecificationType::MolarEnthalpy)
            {
                if let Ok(()) = self.get_phase_prop(
                    pp,
                    phase,
                    &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.enthalpy)
                        .as_cape_string_in(),
                    PropertyBasis::Mole,
                    &mut CapeArrayRealOutFromProvider::from(&mut scalar_value)
                        .as_cape_array_real_out(),
                ) {
                    molar_enthalpy = Some(scalar_value.value());
                };
                if let Ok(()) = self.get_phase_prop(
                    pp,
                    phase,
                    &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.enthalpy)
                        .as_cape_string_in(),
                    PropertyBasis::Mass,
                    &mut CapeArrayRealOutFromProvider::from(&mut scalar_value)
                        .as_cape_array_real_out(),
                ) {
                    specific_enthalpy = Some(scalar_value.value());
                };
            }
            if pp
                .flash_conditions
                .contains(&StreamSpecificationType::MolarEntropy)
            {
                if let Ok(()) = self.get_phase_prop(
                    pp,
                    phase,
                    &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.entropy)
                        .as_cape_string_in(),
                    PropertyBasis::Mole,
                    &mut CapeArrayRealOutFromProvider::from(&mut scalar_value)
                        .as_cape_array_real_out(),
                ) {
                    molar_entropy = Some(scalar_value.value());
                };
                if let Ok(()) = self.get_phase_prop(
                    pp,
                    phase,
                    &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.entropy)
                        .as_cape_string_in(),
                    PropertyBasis::Mass,
                    &mut CapeArrayRealOutFromProvider::from(&mut scalar_value)
                        .as_cape_array_real_out(),
                ) {
                    specific_entropy = Some(scalar_value.value());
                };
            }
            if pp
                .flash_conditions
                .contains(&StreamSpecificationType::MolarVolume)
            {
                //here we assume that density is always equal to 1/volume
                if let Ok(()) = self.get_phase_prop(
                    pp,
                    phase,
                    &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.volume).as_cape_string_in(),
                    PropertyBasis::Mole,
                    &mut CapeArrayRealOutFromProvider::from(&mut scalar_value)
                        .as_cape_array_real_out(),
                ) {
                    molar_volume = Some(scalar_value.value());
                    molar_density = Some(1.0 / scalar_value.value());
                };
                if let Ok(()) = self.get_phase_prop(
                    pp,
                    phase,
                    &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.volume).as_cape_string_in(),
                    PropertyBasis::Mass,
                    &mut CapeArrayRealOutFromProvider::from(&mut scalar_value)
                        .as_cape_array_real_out(),
                ) {
                    specific_volume = Some(scalar_value.value());
                    mass_density = Some(1.0 / scalar_value.value());
                };
            }
            if pp.supports_fugacity
                && let Ok(()) = self.get_phase_prop(
                    pp,
                    phase,
                    &CapeStringInFromProvider::from(&MATERIAL_CONSTANTS.fugacity)
                        .as_cape_string_in(),
                    PropertyBasis::None,
                    &mut CapeArrayRealOutFromProvider::from(&mut vector_value)
                        .as_cape_array_real_out(),
                ) {
                    fugacity = Some(vector_value.into_vec());
                };
        }
        (
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
        )
    }

    /// Calculates phase equilibrium based on the provided specifications and updates phase presence/status.
    ///
    /// # Parameters
    /// - `specification1`: First specification for equilibrium calculation
    /// - `specification2`: Second specification for equilibrium calculation
    /// - `solution_type`: Type of solution to calculate (e.g., flash, dew, bubble).
    ///
    /// # Returns
    /// - `Ok(())` when equilibrium calculation and phase updates are successful.
    fn calc_equilibrium_internal(
        &mut self,
        specification1: &CapeArrayStringIn,
        specification2: &CapeArrayStringIn,
        solution_type: &CapeStringIn,
    ) -> Result<(), COBIAError> {
        self.equilibrium_phases.clear();
        {
            let pp = self.property_package.read().unwrap();
            match pp.pp_equilibrium_routine.calc_equilibrium(
                specification1,
                specification2,
                solution_type,
            ) {
                Ok(()) => {}
                Err(e) => {
                    return Err(e);
                }
            };
        }
        //fix equilibrium phases
        if self.equilibrium_phases.is_empty() {
            return Err(COBIAError::Message(
                "CalcEquilibrium did not specify phases at phase equilibrium".to_string(),
            ));
        }
        for phase in self.phases.iter_mut() {
            phase.present = false;
        }
        for phase_index in self.equilibrium_phases.iter() {
            let phase = &mut self.phases[*phase_index];
            phase.present = true;
            phase.status = cape_open_1_2::CapePhaseStatus::CapeAtequilibrium;
        }
        Ok(())
    }

    /// Returns indices of phases currently marked present.
    ///
    /// # Returns
    /// Vector of phase indices. Empty when the stream is not specified.
    pub fn get_present_phase_indices(&self) -> Vec<usize> {
        let mut indices = Vec::new();
        if self.is_specified {
            for (i, phase) in self.phases.iter().enumerate() {
                if phase.present {
                    indices.push(i);
                }
            }
        }
        indices
    }

    /// Marks the stream as fully specified without recalculation.
    pub fn mark_as_complete(&mut self) {
        self.is_specified = true;
    }

    /// Clears overall/phase properties and transient log entries.
    pub fn clear(&mut self) {
        self.overall.clear();
        self.is_specified = false;
        for phase in self.phases.iter_mut() {
            phase.clear();
        }
        for phase in self.two_phases.iter_mut() {
            phase.clear();
        }
        self.log.clear();
    }
}

impl std::fmt::Display for MaterialStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MaterialStream {}", self.name)
    }
}

impl cape_open_1_2::ICapeIdentification for MaterialStream {
    fn get_component_name(&mut self, name: &mut CapeStringOut) -> Result<(), COBIAError> {
        name.set_string(&self.name)
    }

    fn set_component_name(&mut self, _name: &CapeStringIn) -> Result<(), COBIAError> {
        Err(COBIAError::Code(COBIAERR_DENIED))
    }

    fn get_component_description(&mut self, desc: &mut CapeStringOut) -> Result<(), COBIAError> {
        desc.set_string(&self.description)
    }

    fn set_component_description(&mut self, _desc: &CapeStringIn) -> Result<(), COBIAError> {
        Err(COBIAError::Code(COBIAERR_DENIED))
    }
}

impl cape_open_1_2::ICapeThermoMaterial for MaterialStream {
    fn clear_all_props(&mut self) -> Result<(), COBIAError> {
        if self.is_feed {
            return Err(COBIAError::Message("Cannot modify feed stream".to_string()));
        }
        self.clear();
        Ok(())
    }

    fn copy_from_material(
        &mut self,
        source: cape_open_1_2::CapeThermoMaterial,
    ) -> Result<(), COBIAError> {
        if self.is_feed {
            return Err(COBIAError::Message("Cannot modify feed stream".to_string()));
        }
        //get other
        match unsafe { Self::check_borrow(&source) } {
            Some(source_material) => {
                //copy overall properties
                self.overall = source_material.overall.clone();
                //copy single phase properties
                self.phases = source_material.phases.clone();
                //copy two phase properties
                self.two_phase_index = source_material.two_phase_index.clone();
                self.two_phases = source_material.two_phases.clone();
                //done
                Ok(())
            }
            None => {
                //cannot copy - incompatible MO
                Err(COBIAError::Code(COBIAERR_INVALIDARGUMENT))
            }
        }
    }

    fn create_material(&mut self) -> Result<cape_open_1_2::CapeThermoMaterial, COBIAError> {
        //create and return a new material
        self.copy_index += 1;
        let mo: CapeObject = Self::with_property_package(self.property_package.clone());
        {
            let mo_material = unsafe { Self::borrow_mut(&mo) };
            mo_material.name = format!("Copy {} of {}", self.copy_index, self.name);
            mo_material.description = "Duplicate material object".to_string();
        }
        //must return as CapeThermoMaterial
        let mo: cape_open_1_2::CapeThermoMaterial =
            cape_open_1_2::CapeThermoMaterial::from_object(&mo).map_err(|e| {
                COBIAError::Message(format!(
                    "Error converting new MaterialStream object to ICapeThermoMaterial: {}",
                    e
                ))
            })?;
        Ok(mo)
    }

    fn get_overall_prop(
        &mut self,
        property: &CapeStringIn,
        basis: &CapeStringIn,
        results: &mut CapeArrayRealOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        self.get_phase_prop(
            &pp,
            &self.overall,
            property,
            PropertyBasis::from_string(basis)?,
            results,
        )
    }

    fn get_overall_tpfraction(
        &mut self,
        temperature: &mut CapeReal,
        pressure: &mut CapeReal,
        composition: &mut CapeArrayRealOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        self.get_phase_tpfraction(&pp, &self.overall, temperature, pressure, composition)
    }

    fn get_present_phases(
        &mut self,
        phase_labels: &mut CapeArrayStringOut,
        phase_status: &mut CapeArrayEnumerationOut<cape_open_1_2::CapePhaseStatus>,
    ) -> Result<(), COBIAError> {
        let mut count = 0_usize;
        for phase in self.phases.iter() {
            if phase.present {
                count += 1;
            }
        }
        phase_labels.resize(count)?;
        phase_status.resize(count)?;
        let mut index = 0_usize;
        for phase in self.phases.iter() {
            if phase.present {
                phase_labels.at(index).unwrap().set_string(&phase.label)?;
                phase_status[index] = phase.status;
                index += 1;
            }
        }
        Ok(())
    }

    fn get_single_phase_prop(
        &mut self,
        property: &CapeStringIn,
        phase_label: &CapeStringIn,
        basis: &CapeStringIn,
        results: &mut CapeArrayRealOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        let ph = self.get_phase(&pp, phase_label)?;
        self.get_phase_prop(
            &pp,
            ph,
            property,
            PropertyBasis::from_string(basis)?,
            results,
        )
    }

    fn get_tpfraction(
        &mut self,
        phase_label: &CapeStringIn,
        temperature: &mut CapeReal,
        pressure: &mut CapeReal,
        composition: &mut CapeArrayRealOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        let ph = self.get_phase(&pp, phase_label)?;
        self.get_phase_tpfraction(&pp, ph, temperature, pressure, composition)
    }

    fn get_two_phase_prop(
        &mut self,
        property: &CapeStringIn,
        phase_labels: &CapeArrayStringIn,
        basis: &CapeStringIn,
        results: &mut CapeArrayRealOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        let phase = self.get_two_phase(&pp, phase_labels)?;
        let basis = PropertyBasis::from_string(basis)?;
        let property_info: Option<&PropertyInfo> =
            MATERIAL_CONSTANTS.property_info_map.get(property);
        let (rank, allow_mole_basis) = match property_info {
            Some(PropertyInfo::TwoPhase(rank)) => (*rank, false),
            Some(PropertyInfo::TemperatureDerivative(base))
            | Some(PropertyInfo::PressureDerivative(base)) => {
                //temperature or pressure derivative; expect phase labels "Overall" and the phase of the base property
                match MATERIAL_CONSTANTS.property_info_map.get(base) {
                    Some(PropertyInfo::TwoPhase(rank)) => (*rank, false),
                    _ => {
                        //can only be single phase property
                        return Err(COBIAError::Message(format!(
                            "Property {} is not a two phase property",
                            property
                        )));
                    }
                }
            }
            Some(PropertyInfo::MoleNumberDerivative(base))
            | Some(PropertyInfo::MolFractionDerivative(base)) => {
                match MATERIAL_CONSTANTS.property_info_map.get(base) {
                    Some(PropertyInfo::TwoPhase(rank)) => {
                        let allow_mole_basis = match property_info.unwrap() {
                            PropertyInfo::MoleNumberDerivative(_) => true, //allow mole basis for mole number derivatives
                            _ => false, //do not allow mole basis for mole fraction derivatives
                        };
                        (*rank + 1, allow_mole_basis)
                    }
                    _ => {
                        //can only be single phase property
                        return Err(COBIAError::Message(format!(
                            "Property {} is not a two phase property",
                            property
                        )));
                    }
                }
            }
            Some(_) => {
                return Err(COBIAError::Message(format!(
                    "Property {} is not a two phase property",
                    property
                )));
            }
            None => {
                //not a known property, return as is, without basis conversion
                match phase.value_map.get(property) {
                    Some(prop_value) => {
                        if prop_value.basis != basis {
                            return Err(COBIAError::Message(format!(
                                "Cannot perform basis conversion for unknown property {}",
                                property
                            )));
                        }
                        match prop_value.value {
                            PropertyValueData::Scalar(val) => {
                                results.put_scalar(val)?;
                            }
                            PropertyValueData::Vector(ref vec) => {
                                results.put_array(vec)?;
                            }
                        }
                        return Ok(());
                    }
                    None => {
                        return Err(COBIAError::Message(format!(
                            "Property {} not found for phase {} / {}",
                            property,
                            phase_labels.at(0).unwrap(),
                            phase_labels.at(1).unwrap()
                        )));
                    }
                }
            }
        };
        match basis {
            PropertyBasis::None => {}
            PropertyBasis::Mole => {
                //only allow for mole number derivatives, not mole fraction derivatives
                if !allow_mole_basis {
                    return Err(COBIAError::Message(format!(
                        "Invalid basis {} for property {}; this property is not supported for Mole basis",
                        basis.as_str(),
                        property
                    )));
                }
            }
            PropertyBasis::Mass => {
                return Err(COBIAError::Message(format!(
                    "Invalid basis {} for property {}; this property is not supported for Mass basis",
                    basis.as_str(),
                    property
                )));
            }
        };
        //check data
        match phase.value_map.get(property) {
            Some(prop_value) => {
                match rank {
                    0 => {
                        //as scalar
                        match prop_value.value {
                            PropertyValueData::Scalar(val) => {
                                results.put_scalar(val)?;
                                Ok(())
                            }
                            PropertyValueData::Vector(_) => Err(COBIAError::Message(format!(
                                "Unexpected number of values for {} of phases {} / {}: scalar expected",
                                property,
                                phase_labels.at(0).unwrap(),
                                phase_labels.at(1).unwrap()
                            ))),
                        }
                    }
                    _ => {
                        //as vector
                        match prop_value.value {
                            PropertyValueData::Scalar(_) => Err(COBIAError::Message(format!(
                                "Unexpected number of values for {} of phases {} / {}: vector expected",
                                property,
                                phase_labels.at(0).unwrap(),
                                phase_labels.at(1).unwrap()
                            ))),
                            PropertyValueData::Vector(ref vec) => {
                                results.put_array(vec)?;
                                Ok(())
                            }
                        }
                    }
                }
            }
            None => Err(COBIAError::Message(format!(
                "Property {} not found for phase {} / {}",
                property,
                phase_labels.at(0).unwrap(),
                phase_labels.at(1).unwrap()
            ))),
        }
    }

    fn set_overall_prop(
        &mut self,
        property: &CapeStringIn,
        basis: &CapeStringIn,
        values: &CapeArrayRealIn,
    ) -> Result<(), COBIAError> {
        if self.is_feed {
            return Err(COBIAError::Message("Cannot modify feed stream".to_string()));
        }
        let num_compounds = self.property_package.read().unwrap().num_compounds;
        Self::put_phase_prop(
            num_compounds,
            &mut self.overall,
            property,
            PropertyBasis::from_string(basis)?,
            values,
        )
    }

    fn set_present_phases(
        &mut self,
        phase_labels: &CapeArrayStringIn,
        phase_status: &CapeArrayEnumerationIn<cape_open_1_2::CapePhaseStatus>,
    ) -> Result<(), COBIAError> {
        if self.is_feed {
            return Err(COBIAError::Message("Cannot modify feed stream".to_string()));
        }
        //mark all phases as not present
        for phase in self.phases.iter_mut() {
            phase.present = false;
        }
        //mark specified phases as present and set status
        self.equilibrium_phases.clear();
        for i in 0..phase_labels.size() {
            let label = phase_labels.at(i)?;
            let status = phase_status[i];
            let phase_index = {
                let pp = self.property_package.read().unwrap();
                match pp.phase_index_map.get(&label) {
                    Some(index) => *index,
                    None => {
                        return Err(COBIAError::Message(format!(
                            "Invalid/unsupported phase '{}'",
                            label
                        )));
                    }
                }
            };
            self.equilibrium_phases.push(phase_index);
            let phase = &mut self.phases[phase_index];
            phase.present = true;
            phase.status = status;
        }
        Ok(())
    }

    fn set_single_phase_prop(
        &mut self,
        property: &CapeStringIn,
        phase_label: &CapeStringIn,
        basis: &CapeStringIn,
        values: &CapeArrayRealIn,
    ) -> Result<(), COBIAError> {
        if self.is_feed {
            return Err(COBIAError::Message("Cannot modify feed stream".to_string()));
        }
        let num_compounds = self.property_package.read().unwrap().num_compounds;
        let phase = self.get_phase_mut(phase_label)?;
        Self::put_phase_prop(
            num_compounds,
            phase,
            property,
            PropertyBasis::from_string(basis)?,
            values,
        )
    }

    fn set_two_phase_prop(
        &mut self,
        property: &CapeStringIn,
        phase_labels: &CapeArrayStringIn,
        basis: &CapeStringIn,
        values: &CapeArrayRealIn,
    ) -> Result<(), COBIAError> {
        if self.is_feed {
            return Err(COBIAError::Message("Cannot modify feed stream".to_string()));
        }
        let num_compounds = self.property_package.read().unwrap().num_compounds;
        let phase = self.get_two_phase_mut(phase_labels)?;
        let basis = PropertyBasis::from_string(basis)?;
        let property_info = MATERIAL_CONSTANTS.property_info_map.get(property);
        match property_info {
            Some(PropertyInfo::TwoPhase(rank)) => {
                if basis != PropertyBasis::None {
                    //cannot be correct for a two-phase property
                    return Err(COBIAError::Message(format!(
                        "Invalid basis {} for property {}",
                        basis.as_str(),
                        property
                    )));
                }
                if *rank == 0 {
                    //scalar
                    if values.size() != 1 {
                        Err(COBIAError::Message(format!(
                            "Unexpected number of values for {}: scalar expected",
                            property
                        )))
                    } else {
                        phase.value_map.insert_from_cape_string(
                            property,
                            PropertyValue {
                                value: PropertyValueData::Scalar(values.as_slice()[0]),
                                basis: PropertyBasis::None,
                            },
                        );
                        Ok(())
                    }
                } else {
                    //insert as vector
                    if *rank == 1
                        && values.size() != num_compounds {
                            return Err(COBIAError::Message(format!(
                                "Incorrect number of values ({}) for {}: must match number of compounds ({})",
                                values.size(),
                                property,
                                num_compounds
                            )));
                        }
                    phase.value_map.insert_from_cape_string(
                        property,
                        PropertyValue {
                            value: PropertyValueData::Vector(values.as_vec()),
                            basis: PropertyBasis::None,
                        },
                    );
                    Ok(())
                }
            }
            Some(PropertyInfo::TemperatureDerivative(base))
            | Some(PropertyInfo::PressureDerivative(base)) => {
                //temperature or pressure derivative; expect phase labels "Overall" and the phase of the base property
                match MATERIAL_CONSTANTS.property_info_map.get(base) {
                    Some(PropertyInfo::TwoPhase(rank)) => {
                        if basis != PropertyBasis::None {
                            //cannot be correct for a two-phase property
                            return Err(COBIAError::Message(format!(
                                "Invalid basis {} for property {}",
                                basis.as_str(),
                                property
                            )));
                        }
                        if *rank == 0 {
                            //scalar
                            if values.size() != 1 {
                                Err(COBIAError::Message(format!(
                                    "Unexpected number of values for {}: scalar expected",
                                    property
                                )))
                            } else {
                                phase.value_map.insert_from_cape_string(
                                    property,
                                    PropertyValue {
                                        value: PropertyValueData::Scalar(values.as_slice()[0]),
                                        basis: PropertyBasis::None,
                                    },
                                );
                                Ok(())
                            }
                        } else {
                            //insert as vector
                            if *rank == 1
                                && values.size() != num_compounds {
                                    return Err(COBIAError::Message(format!(
                                        "Incorrect number of values ({}) for {}: must match number of compounds ({})",
                                        values.size(),
                                        property,
                                        num_compounds
                                    )));
                                }
                            phase.value_map.insert_from_cape_string(
                                property,
                                PropertyValue {
                                    value: PropertyValueData::Vector(values.as_vec()),
                                    basis: PropertyBasis::None,
                                },
                            );
                            Ok(())
                        }
                    }
                    _ => {
                        //can only be single phase property
                        Err(COBIAError::Message(format!(
                            "Property {} is not a two phase property",
                            property
                        )))
                    }
                }
            }
            Some(PropertyInfo::MoleNumberDerivative(base))
            | Some(PropertyInfo::MolFractionDerivative(base)) => {
                match MATERIAL_CONSTANTS.property_info_map.get(base) {
                    Some(PropertyInfo::TwoPhase(rank)) => {
                        match basis {
                            PropertyBasis::None => {}
                            PropertyBasis::Mole => {
                                //only allow for mole number derivatives, not mole fraction derivatives
                                if let PropertyInfo::MolFractionDerivative(_) =
                                    property_info.unwrap()
                                {
                                    return Err(COBIAError::Message(format!(
                                        "Invalid basis {} for property {}; this property is not supported for Mole basis",
                                        basis.as_str(),
                                        property
                                    )));
                                }
                            }
                            PropertyBasis::Mass => {
                                return Err(COBIAError::Message(format!(
                                    "Invalid basis {} for property {}; this property is not supported for Mass basis",
                                    basis.as_str(),
                                    property
                                )));
                            }
                        }
                        //insert as vector
                        match *rank {
                            0 => {
                                let expected = 2 * num_compounds; //vector for each phase in pair
                                if values.size() != expected {
                                    return Err(COBIAError::Message(format!(
                                        "Incorrect number of values ({}) for {}: expected {} values",
                                        values.size(),
                                        property,
                                        expected
                                    )));
                                }
                            }
                            1 => {
                                let expected = 2 * num_compounds * num_compounds; //matrix for each phase in pair
                                if values.size() != expected {
                                    return Err(COBIAError::Message(format!(
                                        "Incorrect number of values ({}) for {}: expected {} values",
                                        values.size(),
                                        property,
                                        expected
                                    )));
                                }
                            }
                            _ => {
                                //not sure - perform no additional checks
                            }
                        };
                        phase.value_map.insert_from_cape_string(
                            property,
                            PropertyValue {
                                value: PropertyValueData::Vector(values.as_vec()),
                                basis: PropertyBasis::None,
                            },
                        );
                        Ok(())
                    }
                    _ => {
                        //can only be single phase property
                        Err(COBIAError::Message(format!(
                            "Property {} is not a two phase property",
                            property
                        )))
                    }
                }
            }
            Some(_) => Err(COBIAError::Message(format!(
                "Property {} is not a two phase property",
                property
            ))),
            None => {
                //insert as vector
                phase.value_map.insert_from_cape_string(
                    property,
                    PropertyValue {
                        value: PropertyValueData::Vector(values.as_vec()),
                        basis: PropertyBasis::None,
                    },
                );
                Ok(())
            }
        }
    }
}

impl cape_open_1_2::ICapeThermoCompounds for MaterialStream {
    fn get_compound_constant(
        &mut self,
        props: &CapeArrayStringIn,
        comp_ids: &CapeArrayStringIn,
        contains_missing_values: &mut CapeBoolean,
        prop_vals: &mut CapeArrayValueOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        pp.pp_compounds
            .get_compound_constant(props, comp_ids, contains_missing_values, prop_vals)
    }

    fn get_compound_list(
        &mut self,
        comp_ids: &mut CapeArrayStringOut,
        formulae: &mut CapeArrayStringOut,
        names: &mut CapeArrayStringOut,
        boil_temps: &mut CapeArrayRealOut,
        molwts: &mut CapeArrayRealOut,
        casnos: &mut CapeArrayStringOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        pp.pp_compounds
            .get_compound_list(comp_ids, formulae, names, boil_temps, molwts, casnos)
    }

    fn get_const_prop_list(&mut self, props: &mut CapeArrayStringOut) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        pp.pp_compounds.get_const_prop_list(props)
    }

    fn get_num_compounds(&mut self) -> Result<CapeInteger, COBIAError> {
        let pp = self.property_package.read().unwrap();
        pp.pp_compounds.get_num_compounds()
    }

    fn get_pdependent_property(
        &mut self,
        props: &CapeArrayStringIn,
        pressure: CapeReal,
        comp_ids: &CapeArrayStringIn,
        contains_missing_values: &mut CapeBoolean,
        prop_vals: &mut CapeArrayRealOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        pp.pp_compounds.get_pdependent_property(
            props,
            pressure,
            comp_ids,
            contains_missing_values,
            prop_vals,
        )
    }

    fn get_pdependent_prop_list(
        &mut self,
        props: &mut CapeArrayStringOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        pp.pp_compounds.get_pdependent_prop_list(props)
    }

    fn get_tdependent_property(
        &mut self,
        props: &CapeArrayStringIn,
        temperature: CapeReal,
        comp_ids: &CapeArrayStringIn,
        contains_missing_values: &mut CapeBoolean,
        prop_vals: &mut CapeArrayRealOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        pp.pp_compounds.get_tdependent_property(
            props,
            temperature,
            comp_ids,
            contains_missing_values,
            prop_vals,
        )
    }

    fn get_tdependent_prop_list(
        &mut self,
        props: &mut CapeArrayStringOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        pp.pp_compounds.get_tdependent_prop_list(props)
    }
}

impl cape_open_1_2::ICapeThermoPhases for MaterialStream {
    fn get_num_phases(&mut self) -> Result<CapeInteger, COBIAError> {
        let pp = self.property_package.read().unwrap();
        pp.pp_phases.get_num_phases()
    }

    fn get_phase_info(
        &mut self,
        phase_label: &CapeStringIn,
        phase_attribute: &CapeStringIn,
        value: &mut CapeValueOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        pp.pp_phases
            .get_phase_info(phase_label, phase_attribute, value)
    }

    fn get_phase_list(
        &mut self,
        phase_labels: &mut CapeArrayStringOut,
        state_of_aggregation: &mut CapeArrayStringOut,
        key_compound_id: &mut CapeArrayStringOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        pp.pp_phases
            .get_phase_list(phase_labels, state_of_aggregation, key_compound_id)
    }
}

impl cape_open_1_2::ICapeThermoPropertyRoutine for MaterialStream {
    fn calc_and_get_ln_phi(
        &mut self,
        phase_label: &CapeStringIn,
        temperature: CapeReal,
        pressure: CapeReal,
        mole_fraction: &CapeArrayRealIn,
        f_flags: CapeInteger,
        ln_phi: &mut CapeArrayRealOut,
        ln_phi_dt: &mut CapeArrayRealOut,
        ln_phi_dp: &mut CapeArrayRealOut,
        ln_phi_dn: &mut CapeArrayRealOut,
    ) -> Result<(), COBIAError> {
        self.ensure_context_material()?;
        let pp = self.property_package.read().unwrap();
        pp.pp_property_routine.calc_and_get_ln_phi(
            phase_label,
            temperature,
            pressure,
            mole_fraction,
            f_flags,
            ln_phi,
            ln_phi_dt,
            ln_phi_dp,
            ln_phi_dn,
        )
    }

    fn calc_single_phase_prop(
        &mut self,
        props: &CapeArrayStringIn,
        phase_label: &CapeStringIn,
    ) -> Result<(), COBIAError> {
        if self.is_feed {
            return Err(COBIAError::Message("Cannot modify feed stream".to_string()));
        }
        self.ensure_context_material()?;
        let pp = self.property_package.read().unwrap();
        pp.pp_property_routine
            .calc_single_phase_prop(props, phase_label)
    }

    fn calc_two_phase_prop(
        &mut self,
        props: &CapeArrayStringIn,
        phase_labels: &CapeArrayStringIn,
    ) -> Result<(), COBIAError> {
        if self.is_feed {
            return Err(COBIAError::Message("Cannot modify feed stream".to_string()));
        }
        self.ensure_context_material()?;
        let pp = self.property_package.read().unwrap();
        pp.pp_property_routine
            .calc_two_phase_prop(props, phase_labels)
    }

    fn check_single_phase_prop_spec(
        &mut self,
        property: &CapeStringIn,
        phase_label: &CapeStringIn,
    ) -> Result<CapeBoolean, COBIAError> {
        self.ensure_context_material()?;
        let pp = self.property_package.read().unwrap();
        pp.pp_property_routine
            .check_single_phase_prop_spec(property, phase_label)
    }

    fn check_two_phase_prop_spec(
        &mut self,
        property: &CapeStringIn,
        phase_labels: &CapeArrayStringIn,
    ) -> Result<CapeBoolean, COBIAError> {
        self.ensure_context_material()?;
        let pp = self.property_package.read().unwrap();
        pp.pp_property_routine
            .check_two_phase_prop_spec(property, phase_labels)
    }

    fn get_single_phase_prop_list(
        &mut self,
        props: &mut CapeArrayStringOut,
    ) -> Result<(), COBIAError> {
        self.ensure_context_material()?;
        let pp = self.property_package.read().unwrap();
        pp.pp_property_routine.get_single_phase_prop_list(props)
    }

    fn get_two_phase_prop_list(
        &mut self,
        props: &mut CapeArrayStringOut,
    ) -> Result<(), COBIAError> {
        self.ensure_context_material()?;
        let pp = self.property_package.read().unwrap();
        pp.pp_property_routine.get_two_phase_prop_list(props)
    }
}

impl cape_open_1_2::ICapeThermoEquilibriumRoutine for MaterialStream {
    fn calc_equilibrium(
        &mut self,
        specification1: &CapeArrayStringIn,
        specification2: &CapeArrayStringIn,
        solution_type: &CapeStringIn,
    ) -> Result<(), COBIAError> {
        if self.is_feed {
            return Err(COBIAError::Message("Cannot modify feed stream".to_string()));
        }
        self.ensure_context_material()?;
        self.calc_equilibrium_internal(specification1, specification2, solution_type)
    }

    fn check_equilibrium_spec(
        &mut self,
        specification1: &CapeArrayStringIn,
        specification2: &CapeArrayStringIn,
        solution_type: &CapeStringIn,
    ) -> Result<CapeBoolean, COBIAError> {
        self.ensure_context_material()?;
        let pp = self.property_package.read().unwrap();
        pp.pp_equilibrium_routine.check_equilibrium_spec(
            specification1,
            specification2,
            solution_type,
        )
    }
}

impl cape_open_1_2::ICapeThermoUniversalConstant for MaterialStream {
    fn get_universal_constant(
        &mut self,
        constant_id: &CapeStringIn,
        constant_value: &mut CapeValueOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        match pp.pp_universal_constant {
            Some(ref uc) => uc.get_universal_constant(constant_id, constant_value),
            None => Err(COBIAError::Code(COBIAERR_NOSUCHITEM)),
        }
    }

    fn get_universal_constant_list(
        &mut self,
        constant_id_list: &mut CapeArrayStringOut,
    ) -> Result<(), COBIAError> {
        let pp = self.property_package.read().unwrap();
        match pp.pp_universal_constant {
            Some(ref uc) => uc.get_universal_constant_list(constant_id_list),
            None => {
                constant_id_list.resize(0)?;
                Ok(())
            }
        }
    }
}
