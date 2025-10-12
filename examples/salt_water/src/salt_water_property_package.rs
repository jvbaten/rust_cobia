use cobia::*;
use crate::salt_water_calculator;
use crate::property_tables;
use crate::phase_equilibrium_type::PhaseEquilibriumType;
use strum::{EnumCount, IntoEnumIterator};

///The SaltWaterPropertyPackage is an example of a property package that implements the 
/// CAPE-OPEN 1.2 standard.
///
/// The salt water calculations are implemented in salt_water_calculator.rs.
///
/// The package defines a single liquid phase, and two compounds: water and sodium chloride.
///
/// A CAPE-OPEN 1.2 property package must implemented the following interfaces:
/// - ICapeIdentification
/// - ICapeUtilities
/// - ICapeThermoMaterialContext
/// - ICapeThermoCompounds
/// - ICapeThermoPhases
/// - ICapeThermoPropertyRoutine
/// - ICapeThermoEquilibriumRoutine
/// - ICapeThermoUniversalConstant (optional)
///
/// The package is creatable; the public CAPE-OPEN class factory is implemented in lib.rs;
/// to facilitate the registration of this object into the COBIA registry, the object implements
/// the PMCRegisterationInfo trait.
///
/// Once registered, the package can be instantiated and created in all CAPE-OPEN
/// compliant simulators.
#[cape_object_implementation(
		interfaces = {
			cape_open_1_2::ICapeUtilities,
			cape_open_1_2::ICapeIdentification,
			cape_open_1_2::ICapeThermoMaterialContext,
			cape_open_1_2::ICapeThermoCompounds,
			cape_open_1_2::ICapeThermoPhases,
			cape_open_1_2::ICapeThermoPropertyRoutine,
			cape_open_1_2::ICapeThermoEquilibriumRoutine,
			cape_open_1_2::ICapeThermoUniversalConstant
		}
  )]
pub(crate) struct SaltWaterPropertyPackage {
	/// The name of the components, which for a primary CAPE-OPEN PMC object must be modifiable
	name: String,
	/// The description of the component, which for a primary CAPE-OPEN PMC object should be modifiable
	description: String,
	/// The interface to the active material which contains calculation properties, and on which calculation results are stored
	material: Option<cape_open_1_2::CapeThermoMaterial>,
	/// Map of the constant properties by identifier, to the values for the two compounds. Filled on first use.
	constant_property_map: CapeOpenMap<[CapeValueContent;2]>,
	/// The indices of the compounds in the active material object. Water has index 0, NaCl has index 1.
	material_compound_indices: Vec<usize>,
	/// The index of the NaCl compound in the active material object. If not present, pure water is assumed.
	material_nacl_index: Option<usize>,
	//*******************************************
	//temporary values that are cached and reused
	//*******************************************
	/// Compound IDs on the active material object
	mat_comp_ids : CapeArrayStringVec,
	/// Compound formulae on the active material object
	mat_formulae : CapeArrayStringVec,
	/// Compound names on the active material object
	mat_comp_names : CapeArrayStringVec,
	/// Compound boiling points on the active material object
	mat_boil_temps : CapeArrayRealVec,
	/// Compound molecular weights on the active material object
	mat_molecular_weights : CapeArrayRealVec,
	/// Compound CAS registry numbers on the active material object
	mat_cas_registry_numbers : CapeArrayStringVec,
	/// List of phases resulting from a flash calculation (contains only liquid)
	phase_list : CapeArrayStringVec,
	/// Phase status of the phases resulting from a flash calculation (at equilibrium)
	phase_status : CapeArrayEnumerationVec<cape_open_1_2::CapePhaseStatus>,
	/// Buffer for obtaining property values from the active material object
	property_value : CapeArrayRealVec,
	/// Buffer for scalar properties for obtaining property values from the active material object
	scalar_property_value : CapeArrayRealScalar,
	//****************
	//constant strings
	//****************
	/// The string "fraction" used for the mole fraction property
	fraction : CapeStringImpl,
	/// The string "temperature" used for the temperature property
	temperature: CapeStringImpl,
	/// The string "pressure" used for the pressure property
	pressure : CapeStringImpl,
	/// The string "phaseFraction" used for the phase fraction property
	phase_fraction : CapeStringImpl,
	/// The string "mole" used for the basis of getting and setting properties
	mole : CapeStringImpl,
	/// The string "enthalpy" used for the enthalpy property
	enthalpy : CapeStringImpl,
	/// The string "entropy" used for the entropy property
	entropy : CapeStringImpl,
	/// The empty string used for the basis of getting and setting properties
	empty_string : CapeStringImpl,
	/// The string "H2O" used to check against the active material object compound list
	h2o : CapeStringConstNoCase,
	/// The string "NaCl" used to check against the active material object compound list
	nacl : CapeStringConstNoCase,
	/// The string "Liquid" used to check against the specified phase in property calculations
	liquid: CapeStringConstNoCase,
}

/// Implementation of the Default trait is required for creatable CAPE-OPEN objects
/// as the class factory does not allow for constructor parameters.
impl std::default::Default for SaltWaterPropertyPackage {
	fn default() -> Self {
		Self {
			cobia_object_data: Default::default(),
			name: Self::NAME.to_string(),
			description: Self::DESCRIPTION.to_string(),
			material : None,
			constant_property_map : CapeOpenMap::new(),
			material_compound_indices : Vec::new(),
			material_nacl_index:None,
			mat_comp_ids : CapeArrayStringVec::new(),
			mat_formulae : CapeArrayStringVec::new(),
			mat_comp_names : CapeArrayStringVec::new(),
			mat_boil_temps : CapeArrayRealVec::new(),
			mat_molecular_weights : CapeArrayRealVec::new(),
			mat_cas_registry_numbers : CapeArrayStringVec::new(),
			phase_list : CapeArrayStringVec::new(),
			phase_status : CapeArrayEnumerationVec::<cape_open_1_2::CapePhaseStatus>::new(),
			property_value : CapeArrayRealVec::new(),
			scalar_property_value: CapeArrayRealScalar::new(),
			fraction : CapeStringImpl::from_string("fraction"),
			temperature: CapeStringImpl::from_string("temperature"),
			pressure : CapeStringImpl::from_string("pressure"),
			phase_fraction : CapeStringImpl::from_string("phaseFraction"),
			mole : CapeStringImpl::from_string("mole"),
			enthalpy : CapeStringImpl::from_string("enthalpy"),
			entropy : CapeStringImpl::from_string("entropy"),
			empty_string : CapeStringImpl::new(),
			h2o : CapeStringConstNoCase::from_string("H2O"),
			nacl : CapeStringConstNoCase::from_string("NaCl"),
			liquid: CapeStringConstNoCase::from_string("Liquid"),
		}
	}
}

impl SaltWaterPropertyPackage {
	///The default name for new packages, and the object name as it appears in the COBIA registy
	const NAME: &'static str = "Salt Water";
	///The default description for new packages, and the object description as it appears in the COBIA registy
	const DESCRIPTION: &'static str = "Salt water property package";
	///The ProgID of the package, as it appears in the COBIA registry
	const PROGID: &'static str = "SaltWater.SaltWater";

	/// The list of compound IDs for the two compounds in the package
	const COMP_FORMULAS: [&'static str;2]=["H2O","NaCl"];
	/// The list of compound names for the two compounds in the package
	const COMP_NAMES: [&'static str;2]=["Water","Sodium Chloride"];
	/// The list of compound boiling points for the two compounds in the package
	const COMP_BOIL_TEMPS: [f64;2]=[373.15,1738.0];
	/// The list of compound melting points for the two compounds in the package
	const COMP_MELT_TEMPS: [f64;2]=[273.15,1073.8];
	/// The list of compound molecular weights for the two compounds in the package
	pub(crate) const COMP_MOLWTS: [f64;2]=[18.01528,58.443];
	/// The list of compound CAS registry numbers for the two compounds in the package
	const COMP_CASNOS: [&'static str;2]=["7732-18-5","7647-14-5"];
	/// The list of compound SMILES strings for the two compounds in the package
	const COMP_SMILES: [&'static str;2]=["O","[Na+].[Cl-]"];
	/// The list of compound IUPAC names for the two compounds in the package
	const COMP_IUPAC_NAMES: [&'static str;2]=["oxidane","sodium;chloride"];


	/// This function is called at the start of any function that requires the context 
	/// material to be set. It checks whether a active material object is present, and if so,
	/// it check which compounds are present on the active material object
	///
	/// It is possible for the PME to select a subset of the compounds supported by the
	/// package. For this particular package, not having NaCl is acceptable, as this 
	/// allows for calculations at zero salinity. Not having water is not acceptable,
	/// and returns and error. 
	///
	/// Some other common errors, such as an empty list, multiple appearances of the 
	/// same compound in the list, or unknown compounds, are also checked here

	fn check_context_material(&mut self) -> Result<(), COBIAError> {
		if self.material_compound_indices.is_empty() {
			self.material_nacl_index=None;
			match &self.material {
				Some(material) => {
					match cape_open_1_2::CapeThermoCompounds::from_object(material) {
						Ok(compounds) => {
							compounds.get_compound_list(
									&mut self.mat_comp_ids,
									&mut self.mat_formulae,
									&mut self.mat_comp_names,
									&mut self.mat_boil_temps,
									&mut self.mat_molecular_weights,
									&mut self.mat_cas_registry_numbers)?;
							if self.mat_comp_ids.is_empty() {
								return Err(COBIAError::Message("material objects has no compounds".to_string()));
							}
							for comp in self.mat_comp_ids.iter() {
								//for more compounds one should consider a more efficient search, e.g. a HashMap
								//  note that the items that are compared against are of type CapeStringConstNoCase, which is encoded to the platform string at construction
								if self.h2o==*comp {
									self.material_compound_indices.push(0);
								} else if self.nacl==*comp {
									self.material_nacl_index=Some(self.material_compound_indices.len());
									self.material_compound_indices.push(1);
								} else {
									self.material_compound_indices.clear();
									return Err(COBIAError::Message(format!("Unknown compound ID: {}",comp)));
								}
							}
							//check no double items
							let mut used:[bool;2]=[false,false];
							for index in self.material_compound_indices.iter() {
								if used[*index] {
									self.material_compound_indices.clear();
									return Err(COBIAError::Message("duplicate compounds on material object".to_string()));
								}
								used[*index]=true;
							}
							if !used[0] {
								//this package needs water on the MO
								self.material_compound_indices.clear();
								return Err(COBIAError::Message("material object does not contain water".to_string()));
							}
							Ok(())
						},
						Err(_) => Err(COBIAError::Message("material object does not implement ICapeThermoCompounds".to_string()))
					}
				},
				None => Err(COBIAError::Message("Material Object is not set".to_string()))
			}
		} else {
			Ok(())
		}
	}
}

/// The Display trait is required; it is used by the COBIA package to format the 
/// object that is the source of an error, when a CAPE-OPEN error is returned.
///
/// The returned string should include the name and the type of the object so 
/// that it can be identified in the environment by which the error is processed.

impl std::fmt::Display for SaltWaterPropertyPackage {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{} package \"{}\"", Self::NAME, self.name)
	}
}

/// The registration information is needed for PMC objects that can be created
/// by extenal applicaitons, and therefore need to be in the COBIA registry. This
/// defines the unique identifier for the object, but also how the object is
/// visible to the end-user.

impl PMCRegisterationInfo for SaltWaterPropertyPackage {

	/// The UUID of the package, which is used to identify the package in the COBIA registry

	fn get_uuid() -> CapeUUID {
		//{C3C43FDD-958D-4ED8-BF4D-B801D6D75328}
		CapeUUID::from_slice(&[0xC3u8,0xC4u8,0x3Fu8,0xDDu8,0x95u8,0x8Du8,0x4Eu8,0xD8u8,0xBFu8,0x4Du8,0xB8u8,0x01u8,0xD6u8,0xD7u8,0x53u8,0x28u8])
	}

	/// The registration details of the package, which are used to register the package in the COBIA registry
	///
	/// # Arguments
	/// * `registrar` - The registrar object that is used to register the package in the COBIA registry


	fn registration_details(registrar: &CapeRegistrar) -> Result<(), COBIAError> {
		registrar.put_name(Self::NAME)?;
		registrar.put_description(Self::DESCRIPTION)?;
		registrar.put_cape_version("1.2")?;
		registrar.put_component_version(env!("CARGO_PKG_VERSION"))?;
		registrar.put_vendor_url("https://www.amsterchem.com/")?;
		registrar.put_about(Self::DESCRIPTION)?;
		registrar.put_uuid(&Self::get_uuid())?;
		registrar.put_version_independent_prog_id(Self::PROGID)?;
		registrar.put_prog_id(&format!("{}.1",Self::PROGID))?;
		registrar.add_cat_id(&cape_open::CATEGORYID_STANDALONEPROPERTYPACKAGE)?;
		registrar.add_cat_id(&cape_open_1_2::CATEGORYID_COMPONENT_1_2)?;
		Ok(())
	}
}

/// The ICapeIdentification interface is used to identify any CAPE-OPEN object. It consists of a name
/// and description. For primary PMC objects (creatable objects), the name must be read/write and the 
/// description should be read/write. Also, for primary PMC objects, the initial name and description
/// should match the information in the COBIA registry.

impl cape_open_1_2::ICapeIdentification for SaltWaterPropertyPackage {

	/// Obtain the name of the object
	/// 
	/// # Arguments
	/// * `name` - The name of the object, which is returned through a CapeStringOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not

	fn get_component_name(&mut self,name:&mut CapeStringOut) -> Result<(), COBIAError> {
		name.set_string(&self.name)?;
		Ok(())
	}

	/// Obtain the description of the object
	///
	/// # Arguments
	/// * `description` - The description of the object, which is returned through a CapeStringOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not

	fn get_component_description(&mut self,description:&mut CapeStringOut) -> Result<(), COBIAError> {
		description.set_string(&self.description)?;
		Ok(())
	}

	/// Set the name of the object
	///
	/// # Arguments
	/// * `name` - The name of the object, which is specified through a CapeStringIn object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not

	fn set_component_name(&mut self, name: &CapeStringIn) -> Result<(), COBIAError> {
		self.name = name.to_string();
		Ok(())
	}

	/// Set the description of the object
	///
	/// # Arguments
	/// * `desc` - The description of the object, which is specified through a CapeStringIn object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not

	fn set_component_description(&mut self, desc: &CapeStringIn) -> Result<(), COBIAError> {
		self.description = desc.to_string();
		Ok(())
	}
}

/// The ICapeUtilities interface must be implemented by all CAPE-OPEN Primary PMC (creatable) objects.

impl cape_open_1_2::ICapeUtilities for SaltWaterPropertyPackage {

	/// Obtain the parameter collection of the object
	/// 
	/// This object has no parameters, and the method always returns a not implemented error.
	///
	/// # Arguments
	/// * `params` - The parameter collection of the object, which is returned through a CapeCollection object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
	
	fn get_parameters(&mut self) -> Result<cape_open_1_2::CapeCollection<cape_open_1_2::CapeParameter>,COBIAError> {
		Err(COBIAError::Code(COBIAERR_NOTIMPLEMENTED))
	}

	/// Set the simulation context of the object
	/// 
	/// The Simulation Context object provides a number of services implemented by the PMC, including
	/// the ability to log messages. It is not used by this object.
	///
	/// # Arguments
	/// * `context` - The simulation context of the object, which is specified through a CapeSimulationContext object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not

	fn set_simulation_context(&mut self,_:cape_open_1_2::CapeSimulationContext) -> Result<(),COBIAError> {
		Ok(()) //not needed
	}

	/// Initialize the object
	///
	/// This object does not need any initialization, and the method always returns success. Initialization must be 
	/// called by the PME on any CAPE-OPEN Primary PMC Object, after persistence (if the object is depersist) but 
	/// before any other method is called (except setting the simulation context).
	///
	/// Any object that is successfully initialized must be terminated before it can be destroyed. If Initialize
	/// returns an error, Terminate must not be called.
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not

	fn initialize(&mut self) -> Result<(),COBIAError> {
		Ok(()) //nothing to do
	}

	/// Terminate the object
	///
	/// During termination, and object must release all external references. For this object, 
	/// the only extenal reference is the active material object. 
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not

	fn terminate(&mut self) -> Result<(),COBIAError> {
		//drop all external references
		self.material=None;
		Ok(())
	}

	/// Edit the object; this is the only time the object may show a modal dialog.
	///
	/// Editing may be invoked to alter the state of the object, or just to inspect the
	/// object. Therefore the edit function must indicate whether the object has changed
	/// so that the PME may re-obtain the information exposed by the object (supported
	/// compounds, phases, ...) and invalidate any solution that involves calculations
	/// by the object
	///
	/// # Arguments
	/// * `window_id` - The window ID of the parent window
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not; 
	///              contains modification state for a successful operation
	fn edit(&mut self,_:CapeWindowId) -> Result<cape_open_1_2::CapeEditResult,COBIAError> {
		Err(COBIAError::Code(COBIAERR_NOTIMPLEMENTED)) //no edit dialog
	}
}

/// The ICapeThermoMaterialContext interface must be implemented by a property package to allow 
/// setting the active material object on which all calculations are to be done. Calculation conditions
/// are obtained from the active material object, and calculation results are written to the 
/// active material object.

impl cape_open_1_2::ICapeThermoMaterialContext for SaltWaterPropertyPackage {

	/// Set the active material object
	///
	/// # Arguments
	/// * `material` - The active material object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
	fn set_material(&mut self,material:cape_open_1_2::CapeThermoMaterial) -> Result<(),COBIAError> {
		self.material=Some(material);
		self.material_compound_indices.clear();
		Ok(())
	}
	/// Clear the active material object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
	fn unset_material(&mut self) -> Result<(),COBIAError> {
		self.material=None;
		self.material_compound_indices.clear();
		Ok(())
	}
}

/// The ICapeThermoCompounds interface must be implemented by a property package to expose
/// the supported list of compounds. The active list of compounds is that on the active
/// material object
impl cape_open_1_2::ICapeThermoCompounds for SaltWaterPropertyPackage {

	/// Get one or more compound constant values for one or more compounds
	///
	/// # Arguments
	/// * `props` - The list of compound properties to be obtained, which is specified through a CapeArrayStringIn object
	/// * `comp_ids` - The list of compound IDs, which is specified through a CapeArrayStringIn object; if empty, values are to be returned for all compounds
	/// * `contains_missing_values` - A boolean value that indicates whether the result contains missing values, which is returned through a CapeBoolean object
	/// * `prop_vals` - The list of compound property values, which is returned through a CapeArrayValueOut object; for each property, the values for all compounds are returned
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not

	fn get_compound_constant(&mut self,props:&CapeArrayStringIn,comp_ids:&CapeArrayStringIn,contains_missing_values:&mut CapeBoolean,prop_vals:&mut CapeArrayValueOut) -> Result<(),COBIAError> {
		//check 
		if self.constant_property_map.is_empty() {
			//fill constant property map
			self.constant_property_map.insert("normalboilingpoint".into(),[CapeValueContent::Real(Self::COMP_BOIL_TEMPS[0]),CapeValueContent::Real(Self::COMP_BOIL_TEMPS[1])]);
			self.constant_property_map.insert("molecularweight".into(),[CapeValueContent::Real(Self::COMP_MOLWTS[0]),CapeValueContent::Real(Self::COMP_MOLWTS[1])]);
			self.constant_property_map.insert("casregistrynumber".into(),[CapeValueContent::String(Self::COMP_CASNOS[0].to_string()),CapeValueContent::String(Self::COMP_CASNOS[1].to_string())]);
			self.constant_property_map.insert("chemicalformula".into(),[CapeValueContent::String(Self::COMP_FORMULAS[0].to_string()),CapeValueContent::String(Self::COMP_FORMULAS[1].to_string())]);
			self.constant_property_map.insert("iupacname".into(),[CapeValueContent::String(Self::COMP_IUPAC_NAMES[0].to_string()),CapeValueContent::String(Self::COMP_IUPAC_NAMES[1].to_string())]);
			self.constant_property_map.insert("normalfreezingpoint".into(),[CapeValueContent::Real(Self::COMP_MELT_TEMPS[0]),CapeValueContent::Real(Self::COMP_MELT_TEMPS[1])]);
			self.constant_property_map.insert("smilesformula".into(),[CapeValueContent::String(Self::COMP_SMILES[0].to_string()),CapeValueContent::String(Self::COMP_SMILES[1].to_string())]);
		}
		//check comp_ids
		let mut comp_indices: Vec<u32>=Vec::new();
		if comp_ids.is_empty() {
			comp_indices.push(0);
			comp_indices.push(1);
		} else {
			for comp_id in comp_ids.iter() {
				//For more compounds, this should be a HashMap based lookup
				if self.h2o==comp_id {
					comp_indices.push(0);
				} else if self.nacl==comp_id {
					comp_indices.push(1);
				} else {
					return Err(COBIAError::Message(format!("Unknown compound ID: {}",comp_id)));
				}
			}
		}
		//compile result
		*contains_missing_values=false as CapeBoolean;
		let mut results: Vec<CapeValueContent>=Vec::new();
		results.reserve(comp_ids.size()*props.size());
		for prop in props.iter() {
			//find in property map
			let values: &[CapeValueContent;2]=
				match self.constant_property_map.get(&prop) {
					Some(values) => {
						values
					},
					None => {
						return Err(COBIAError::Message(format!("Unsupported compound constant: {}",prop)));
					}
				};
			for comp_index in comp_indices.iter() {
				results.push(values[*comp_index as usize].clone());
			}
		}
		prop_vals.put_array(&results)?;
		Ok(())
	}

	/// Get the list of compounds supported by the package; water and sodium chloride for this package
	///
	/// # Arguments
	/// * `comp_ids` - The list of compound IDs, which is returned through a CapeArrayStringOut object
	/// * `formulae` - The list of compound formulae, which is returned through a CapeArrayStringOut object
	/// * `names` - The list of compound names, which is returned through a CapeArrayStringOut object
	/// * `boil_temps` - The list of compound boiling points, which is returned through a CapeArrayRealOut object
	/// * `molwts` - The list of compound molecular weights, which is returned through a CapeArrayRealOut object
	/// * `casnos` - The list of compound CAS registry numbers, which is returned through a CapeArrayStringOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not

	fn get_compound_list(&mut self,comp_ids:&mut CapeArrayStringOut,formulae:&mut CapeArrayStringOut,names:&mut CapeArrayStringOut,boil_temps:&mut CapeArrayRealOut,molwts:&mut CapeArrayRealOut,casnos:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
		comp_ids.put_array(&Self::COMP_FORMULAS)?;
		formulae.put_array(&Self::COMP_FORMULAS)?;
		names.put_array(&Self::COMP_NAMES)?;
		boil_temps.put_array(&Self::COMP_BOIL_TEMPS)?;
		molwts.put_array(&Self::COMP_MOLWTS)?;
		casnos.put_array(&Self::COMP_CASNOS)?;
		Ok(())
	}

	/// Get a list of supported compound constants
	///
	/// # Arguments
	/// * `props` - The list of compound properties, which is returned through a CapeArrayStringOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
	fn get_const_prop_list(&mut self,props:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
		props.put_array(&[
			"normalBoilingPoint",
			"molecularWeight",
			"casRegistryNumber",
			"chemicalFormula",
			"iupacName",
			"normalFreezingPoint",
			"SMILESformula"])?;
		Ok(())
	}

	/// Get the number of compounds supported by this package
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not, 
	///              containing the number of compounds in case of success
	fn get_num_compounds(&mut self) -> Result<CapeInteger,COBIAError> {
		Ok(2)
	}

	/// Calculate pressure dependent properties at specified pressure
	/// 
	/// This package does not support pressure dependent properties, and the method always fails.
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not, 
	///              containing the number of compounds in case of success
	fn get_pdependent_property(&mut self,_:&CapeArrayStringIn,_:CapeReal,_:&CapeArrayStringIn,_:&mut CapeBoolean,_:&mut CapeArrayRealOut) -> Result<(),COBIAError> {
		//no pressure dependent property support
		Err(COBIAError::Code(COBIAERR_NOTIMPLEMENTED))
	}

	/// Get the list of pressure dependent properties supported by this package
	///
	/// This package does not support pressure dependent properties, so returns an empty list.
	///
	/// # Arguments
	/// * `compounds` - The list of compound IDs, which is specified through a CapeArrayStringIn object
	/// * `props` - The list of pressure dependent properties, which is returned through a CapeArrayStringOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
	fn get_pdependent_prop_list(&mut self,props:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
		//no pressure dependent property support
		let empty_list:[&str;0]=[];
		props.put_array(&empty_list)?;
		Ok(())
	}

	/// Calculate temperature dependent properties at specified temperature
	///
	/// This package does not support temperature dependent properties, and the method always fails.
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not, 
	///              containing the number of compounds in case of success
	fn get_tdependent_property(&mut self,_:&CapeArrayStringIn,_:CapeReal,_:&CapeArrayStringIn,_:&mut CapeBoolean,_:&mut CapeArrayRealOut) -> Result<(),COBIAError> {
		//no temperature dependent property support
		Err(COBIAError::Code(COBIAERR_NOTIMPLEMENTED))
	}

	/// Get the list of temperature dependent properties supported by this package
	///
	/// This package does not support temperature dependent properties, so returns an empty list.
	///
	/// # Arguments
	/// * `props` - The list of temperature dependent properties, which is returned through a CapeArrayStringOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
	fn get_tdependent_prop_list(&mut self,props:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
		//no temperature dependent property support
		let empty_list:[&str;0]=[];
		props.put_array(&empty_list)?;
		Ok(())
	}
}

/// The ICapeThermoPhases interface must be implemented by a property package to expose
/// the supported list of phases. 
impl cape_open_1_2::ICapeThermoPhases for SaltWaterPropertyPackage {

	/// Get the number of phases supported by this package. 
	///
	/// This package supports 1 phase.
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not,
	///              and in case of success returns the number of phases supported by the package.
    fn get_num_phases(&mut self) -> Result<CapeInteger,COBIAError> {
        Ok(1) //only liquid
    }

	/// Get information on a phase.
	///
	/// # Arguments
	/// * `phase_label` - The label of the phase, which is specified through a CapeStringIn object
	/// * `phase_attribute` - The attribute of the phase, which is specified through a CapeStringIn object
	/// * `value` - The value of the phase attribute, which is returned through a CapeValueOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
    fn get_phase_info(&mut self,phase_label:&CapeStringIn,phase_attribute:&CapeStringIn,value:&mut CapeValueOut) -> Result<(),COBIAError> {
        if self.liquid==*phase_label {
			//for get_phase_info, performance is not critical. If performance is critical, one should
			// not convert to string, but compare the CapeConstString directly, which is pre-encoded,
			// or build a HashMap using CapeConstString as key
			match phase_attribute.as_string().to_lowercase().as_str() {
				"stateofaggregation" => {value.set_string("Liquid")?;Ok(())}
				"keycompoundid" => {value.set_string("H2O")?;Ok(())}
				"excludedcompoundid" => {value.set_empty()?;Ok(())}
				_ => return Err(COBIAError::Message(format!("Unsupported phase attribute: {}",phase_attribute)))
			}			
		} else {
			Err(COBIAError::Message(format!("Unsupported phase: {}",phase_label)))
		}
    }

	/// Get the list of phases supported by this package
	/// 
	/// This package supports only one phase: liquid. The liquid
	/// phase exposes water as its key compound. As there is only one liquid phase, this is not
	/// really necessary, but it is obvious in this context that the liquid phase is always
	/// and acquous phase.
	///
	/// # Arguments
	/// * `phase_labels` - The list of phase labels, which is returned through a CapeArrayStringOut object
	/// * `state_of_aggregation` - The list of state of aggregation, which is returned through a CapeArrayStringOut object
	/// * `key_compound_id` - The list of key compound IDs, which is returned through a CapeArrayStringOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
    fn get_phase_list(&mut self,phase_labels:&mut CapeArrayStringOut,state_of_aggregation:&mut CapeArrayStringOut,key_compound_id:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
        phase_labels.put_array(&["Liquid"])?;
		state_of_aggregation.put_array(&["Liquid"])?;
		key_compound_id.put_array(&["H2O"])?;
		Ok(())
    }
}

/// The ICapeThermoPropertyRoutine interface must be implemented by a property package to allow
/// a client to calculate single-phase and two-phase properties. 
impl cape_open_1_2::ICapeThermoPropertyRoutine for SaltWaterPropertyPackage {

	/// Calculate the fugacity coefficient of a compound in a phase
	///
	/// Not supported by this package.
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
    fn calc_and_get_ln_phi(&mut self,_:&CapeStringIn,_:CapeReal,_:CapeReal,_:&CapeArrayRealIn,_:CapeInteger,_:&mut CapeArrayRealOut,_:&mut CapeArrayRealOut,_:&mut CapeArrayRealOut,_:&mut CapeArrayRealOut) -> Result<(),COBIAError> {
        //this package does not support fugacity coefficient calculations
		Err(COBIAError::Code(COBIAERR_NOTIMPLEMENTED))
    }

	/// Calculate one or more single-phase properties for the specified phase
	///
	/// Obtains the calculation inputs from the active material object, and 
	/// sets the calculation results on the active material object.
	///
	/// The underlying property calculations are performed by the `salt_water_calculator` module.
	///
	/// # Arguments
	/// * `props` - The list of properties to be calculated, which is specified through a CapeArrayStringIn object
	/// * `phase_label` - The label of the phase, which is specified through a CapeStringIn object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
    fn calc_single_phase_prop(&mut self,props:&CapeArrayStringIn,phase_label:&CapeStringIn) -> Result<(),COBIAError> {
		self.check_context_material()?;
		if self.liquid!=*phase_label {
			return Err(COBIAError::Message(format!("Unsupported phase: {}",phase_label)));
		}
		if !props.is_empty() {
			//get temperature, pressure and composition (composition in mass units)
			let material=self.material.as_ref().unwrap(); //cannot be None, see check_context_material
			let x_nacl=match self.material_nacl_index {
				None => 0.0,
				Some(nacl_index) => {
					material.get_single_phase_prop(&self.fraction,phase_label,&self.mole,&mut self.property_value)?;
					if self.property_value.size()!=self.material_compound_indices.len() {
						return Err(COBIAError::Message("unexpected number of values for mole fraction".to_string()));
					}
					self.property_value[nacl_index]
				}
			};
			material.get_single_phase_prop(&self.temperature,phase_label,&self.empty_string,&mut self.scalar_property_value)?;
			let temperature = self.scalar_property_value.value();
			material.get_single_phase_prop(&self.pressure,phase_label,&self.empty_string,&mut self.scalar_property_value)?;
			let pressure = self.scalar_property_value.value();
			let prop_table = &property_tables::PROPERTYTABLES;
			for prop in props.iter() {
				match prop_table.get_single_phase_property(&prop) {
					Some(single_phase_property) => {
						match single_phase_property {
							property_tables::SinglePhaseProperty::Viscosity => {
								match salt_water_calculator::viscosity(temperature,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::ViscositydTemperature => {
								match salt_water_calculator::viscosity_d_temperature(temperature,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::ViscositydPressure => {
								match salt_water_calculator::viscosity_d_pressure(temperature,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::ViscositydMoles => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::intenstive_dn(salt_water_calculator::viscosity_d_x_nacl(temperature,x_nacl),x_nacl) {
										Ok(value) => {
											let mut values=[0.0,0.0];
											for i in 0..self.material_compound_indices.len() {
												values[i]=value[self.material_compound_indices[i]];
											}	
											material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealSlice::new(&values))?;
										
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
							property_tables::SinglePhaseProperty::ViscositydMolFraction => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::unconstrained_dx(salt_water_calculator::viscosity_d_x_nacl(temperature,x_nacl)) {
										Ok(value) => {
											let mut values=[0.0,0.0];
											for i in 0..self.material_compound_indices.len() {
												values[i]=value[self.material_compound_indices[i]];
											}	
											material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealSlice::new(&values))?;
										
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
							property_tables::SinglePhaseProperty::ThermalConductivity=> {
								match salt_water_calculator::thermal_conductivity(temperature,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::ThermalConductivitydTemperature => {
								match salt_water_calculator::thermal_conductivity_d_temperature(temperature,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::ThermalConductivitydPressure => {
								match salt_water_calculator::thermal_conductivity_d_pressure(temperature,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::ThermalConductivitydMoles => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::intenstive_dn(salt_water_calculator::thermal_conductivity_d_x_nacl(temperature,x_nacl),x_nacl) {
										Ok(value) => {
											let mut values=[0.0,0.0];
											for i in 0..self.material_compound_indices.len() {
												values[i]=value[self.material_compound_indices[i]];
											}	
											material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealSlice::new(&values))?;
										
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
							property_tables::SinglePhaseProperty::ThermalConductivitydMolFraction => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::unconstrained_dx(salt_water_calculator::thermal_conductivity_d_x_nacl(temperature,x_nacl)) {
										Ok(value) => {
											let mut values=[0.0,0.0];
											for i in 0..self.material_compound_indices.len() {
												values[i]=value[self.material_compound_indices[i]];
											}	
											material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealSlice::new(&values))?;
										
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
							property_tables::SinglePhaseProperty::Enthalpy => {
								match salt_water_calculator::enthalpy(temperature,pressure,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::EnthalpydTemperature => {
								match salt_water_calculator::enthalpy_d_temperature(temperature,pressure,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::EnthalpydPressure => {
								match salt_water_calculator::enthalpy_d_pressure(temperature,pressure,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::EnthalpydMoles => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::extenstive_dn(salt_water_calculator::enthalpy_d_x_nacl(temperature,pressure,x_nacl),salt_water_calculator::enthalpy(temperature,pressure,x_nacl),x_nacl) {
										Ok(value) => {
											let mut values=[0.0,0.0];
											for i in 0..self.material_compound_indices.len() {
												values[i]=value[self.material_compound_indices[i]];
											}	
											material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealSlice::new(&values))?;
										
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
							property_tables::SinglePhaseProperty::EnthalpydMolFraction => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::unconstrained_dx(salt_water_calculator::enthalpy_d_x_nacl(temperature,pressure,x_nacl)) {
										Ok(value) => {
											let mut values=[0.0,0.0];
											for i in 0..self.material_compound_indices.len() {
												values[i]=value[self.material_compound_indices[i]];
											}	
											material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealSlice::new(&values))?;
										
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
							property_tables::SinglePhaseProperty::Entropy => {
								match salt_water_calculator::entropy(temperature,pressure,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::EntropydTemperature => {
								match salt_water_calculator::entropy_d_temperature(temperature,pressure,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::EntropydPressure => {
								match salt_water_calculator::entropy_d_pressure(temperature,pressure,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::EntropydMoles => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::extenstive_dn(salt_water_calculator::entropy_d_x_nacl(temperature,pressure,x_nacl),salt_water_calculator::entropy(temperature,pressure,x_nacl),x_nacl) {
										Ok(value) => {
											let mut values=[0.0,0.0];
											for i in 0..self.material_compound_indices.len() {
												values[i]=value[self.material_compound_indices[i]];
											}	
											material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealSlice::new(&values))?;
										
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
							property_tables::SinglePhaseProperty::EntropydMolFraction => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::unconstrained_dx(salt_water_calculator::entropy_d_x_nacl(temperature,pressure,x_nacl)) {
										Ok(value) => {
											let mut values=[0.0,0.0];
											for i in 0..self.material_compound_indices.len() {
												values[i]=value[self.material_compound_indices[i]];
											}	
											material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealSlice::new(&values))?;
										
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
							property_tables::SinglePhaseProperty::Density => {
								match salt_water_calculator::density(temperature,pressure,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::DensitydTemperature => {
								match salt_water_calculator::density_d_temperature(temperature,pressure,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::DensitydPressure => {
								match salt_water_calculator::density_d_pressure(temperature,pressure,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::DensitydMoles => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::intenstive_dn(salt_water_calculator::density_d_x_nacl(temperature,pressure,x_nacl),x_nacl) {
										Ok(value) => {
											let mut values=[0.0,0.0];
											for i in 0..self.material_compound_indices.len() {
												values[i]=value[self.material_compound_indices[i]];
											}	
											material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealSlice::new(&values))?;
										
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
							property_tables::SinglePhaseProperty::DensitydMolFraction => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::unconstrained_dx(salt_water_calculator::density_d_x_nacl(temperature,pressure,x_nacl)) {
										Ok(value) => {
											let mut values=[0.0,0.0];
											for i in 0..self.material_compound_indices.len() {
												values[i]=value[self.material_compound_indices[i]];
											}	
											material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealSlice::new(&values))?;
										
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
							property_tables::SinglePhaseProperty::Volume => {
								match salt_water_calculator::density(temperature,pressure,x_nacl) {
									Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(1.0/value))?;},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::VolumedTemperature => {
								match salt_water_calculator::density_d_temperature(temperature,pressure,x_nacl) {
									Ok(derivative_value) => {
										match salt_water_calculator::density(temperature,pressure,x_nacl) {
											Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(-derivative_value/(value*value)))?;},
											Err(msg) => {return Err(COBIAError::Message(msg));}
										}
									},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::VolumedPressure => {
								match salt_water_calculator::density_d_pressure(temperature,pressure,x_nacl) {
									Ok(derivative_value) => {
										match salt_water_calculator::density(temperature,pressure,x_nacl) {
											Ok(value) => {material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealScalar::from(-derivative_value/(value*value)))?;},
											Err(msg) => {return Err(COBIAError::Message(msg));}
										}
									},
									Err(msg) => {return Err(COBIAError::Message(msg));}
								}
							},
							property_tables::SinglePhaseProperty::VolumedMoles => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::extenstive_reciprocal_dn(salt_water_calculator::density_d_x_nacl(temperature,pressure,x_nacl),salt_water_calculator::density(temperature,pressure,x_nacl),x_nacl) {
										Ok(value) => {
											let mut values=[0.0,0.0];
											for i in 0..self.material_compound_indices.len() {
												values[i]=value[self.material_compound_indices[i]];
											}	
											material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealSlice::new(&values))?;
										
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
							property_tables::SinglePhaseProperty::VolumedMolFraction => {
								if self.material_compound_indices.len()==1 {
									material.set_single_phase_prop(&prop,phase_label,&self.empty_string,&CapeArrayRealScalar::from(0.0))?;
								} else {
									match salt_water_calculator::unconstrained_dx(salt_water_calculator::density_d_x_nacl(temperature,pressure,x_nacl)) {
										Ok(derivative_value) => {
											match salt_water_calculator::density(temperature,pressure,x_nacl) {
												Ok(value) => {
													let mut values=[0.0,0.0];
													for i in 0..self.material_compound_indices.len() {
														values[i]=-derivative_value[self.material_compound_indices[i]]/(value*value);
													}	
													material.set_single_phase_prop(&prop,phase_label,&self.mole,&CapeArrayRealSlice::new(&values))?;
												},
												Err(msg) => {return Err(COBIAError::Message(msg));}
											}
									
										},
										Err(msg) => {return Err(COBIAError::Message(msg));}
									}
								} 
							},
						}
					}
					None => {return Err(COBIAError::Message(format!("Unsupported single phase property: {}",prop)));}
				}
    		}
		}
		Ok(())
    }

	/// Calculate two-phase properties for the specified phase pair.
	///
	/// This package does not support two-phase properties, and the method always fails.
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
    fn calc_two_phase_prop(&mut self,_:&CapeArrayStringIn,_:&CapeArrayStringIn) -> Result<(),COBIAError> {
		//only one phase is defined
		Err(COBIAError::Code(COBIAERR_NOTIMPLEMENTED))
    }

	/// Check whether a single-phase property calculation is supported for the specified phase.
	///
	/// This package supports a number of single-phase properties, which are defined in the
	/// `property_tables` module. 
	///
	/// # Arguments
	/// * `property` - The property to be checked, which is specified through a CapeStringIn object
	/// * `phase_label` - The label of the phase, which is specified through a CapeStringIn object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not, and if succesful, whether the property is supported.
    fn check_single_phase_prop_spec(&mut self,property:&CapeStringIn,phase_label:&CapeStringIn) -> Result<CapeBoolean,COBIAError> {
		let mut is_supported:CapeBoolean=false as CapeBoolean;
        if self.liquid==*phase_label {
			match (&property_tables::PROPERTYTABLES).get_single_phase_property(property) {
				Some(_) => {is_supported=true as CapeBoolean},
				None => {}
			}
		} 
		Ok(is_supported)
    }

	/// Check whether a two-phase property calculation is supported for the specified phase pair.
	///
	/// This package does not support two-phase properties, and the method always returns false.
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
    fn check_two_phase_prop_spec(&mut self,_:&CapeStringIn,_:&CapeArrayStringIn) -> Result<CapeBoolean,COBIAError> {
		//only one phase is defined
		Ok(false as CapeBoolean)
    }

	/// Get a list of supported single-phase properties.
	///
	/// This package supports a number of single-phase properties, which are defined in the
	/// `property_tables` module. The list of supported properties is returned through the
	/// `props` argument.
	///
	/// # Arguments
	/// * `props` - The list of supported single-phase properties, which is returned through a CapeArrayStringOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
    fn get_single_phase_prop_list(&mut self,props:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
		let n=property_tables::SinglePhaseProperty::COUNT;
		let mut prop_list:Vec<String>=Vec::with_capacity(n);
		for prop in property_tables::SinglePhaseProperty::iter() {
			prop_list.push(prop.name().to_string());
		}
        props.put_array(&prop_list)?;
		Ok(())
    }

	/// Get a list of supported two-phase properties.
	///
	/// This package does not support two-phase properties, and the method always returns an empty list.		
	/// # Arguments
	/// * `props` - The list of supported two-phase properties, which is returned through a CapeArrayStringOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
    fn get_two_phase_prop_list(&mut self,props:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
		//only one phase is defined (so alas, no surface tension)
		let empty_list:[&str;0]=[];
		props.put_array(&empty_list)?;
		Ok(())
    }
}

/// The ICapeThermoPropertyRoutine interface must be implemented by a property package to allow
/// a client to calculate phase equilibria.
impl cape_open_1_2::ICapeThermoEquilibriumRoutine for SaltWaterPropertyPackage {

	/// Calculate the phase equilibrium at specified conditions.
	///
	/// Given the specified conditions and overall composition, return the phase equilibrium that 
	/// matches those conditions. As this package only supports one phase, the resulting phase equilibrium
	/// is always a liquid phase at the specified conditions.
	///
	/// The underlying calculations are performed by the `salt_water_calculator` module.
	///
	/// # Arguments
	/// * `specification1` - The first specification for the phase equilibrium, which is specified through a CapeStringIn object
	/// * `specification2` - The second specification for the phase equilibrium, which is specified through a CapeStringIn object
	/// * `solution_type` - The type of solution to be used for the phase equilibrium calculation, which is specified through a CapeStringIn object; only the "Unspecified" solution type is supported.
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
    fn calc_equilibrium(&mut self,specification1:&CapeArrayStringIn,specification2:&CapeArrayStringIn,solution_type:&CapeStringIn) -> Result<(),COBIAError> {
		self.check_context_material()?;
		let material=self.material.as_ref().unwrap(); //cannot be None, see check_context_material
		material.get_present_phases(&mut self.phase_list,&mut self.phase_status)?;
		if self.phase_list.size()!=1 || self.liquid!=self.phase_list[0] {
			return Err(COBIAError::Message("Unsupported list of allowed phases: only single phase liquid flash is supported".to_string()));
		}
        let flash_type=PhaseEquilibriumType::new(specification1,specification2,solution_type)?;
		let mut temperature=f64::NAN;
		let mut pressure=f64::NAN;
		//check salinity range
		let x_nacl=match self.material_nacl_index {
			None => 0.0,
			Some(nacl_index) => {
				material.get_overall_prop(&self.fraction,&self.mole,&mut self.property_value)?;
				if self.property_value.size()!=self.material_compound_indices.len() {
					return Err(COBIAError::Message("unexpected number of values for mole fraction".to_string()));
				}
				self.property_value[nacl_index]
			}
		};
		if x_nacl<0.0 || x_nacl>0.04033898281 {
			//outside of this range our enthalpy and entropy calculations are not valid
			return Err(COBIAError::Message("Salinity outside of supported range of [0,0.04033898281]".to_string()));
		}
		match flash_type {
			PhaseEquilibriumType::TemperaturePressure => {},
			PhaseEquilibriumType::PressureEnthalpy => {
				//get pressure, enthalpy and salinity
				material.get_overall_prop(&self.pressure,&self.empty_string,&mut self.scalar_property_value)?;
				pressure = self.scalar_property_value.value();
				material.get_overall_prop(&self.enthalpy,&self.mole,&mut self.scalar_property_value)?;
				let enthalpy=self.scalar_property_value.value();
				//calculate temperature to match enthaply
				match salt_water_calculator::solve_temperature_from_enthalpy(enthalpy,pressure,x_nacl) {
					Ok(value) => temperature=value,
					Err(e) => return Err(COBIAError::Message(e))
				}
			},
			PhaseEquilibriumType::PressureEntropy => {
				//get pressure, entropy and salinity
				material.get_overall_prop(&self.pressure,&self.empty_string,&mut self.scalar_property_value)?;
				pressure = self.scalar_property_value.value();
				material.get_overall_prop(&self.entropy,&self.mole,&mut self.scalar_property_value)?;
				let entropy=self.scalar_property_value.value();
				let x_nacl=match self.material_nacl_index {
					None => 0.0,
					Some(nacl_index) => {
						material.get_overall_prop(&self.fraction,&self.mole,&mut self.property_value)?;
						if self.property_value.size()!=self.material_compound_indices.len() {
							return Err(COBIAError::Message("unexpected number of values for mole fraction".to_string()));
						}
						self.property_value[nacl_index]
					}
				};
				//calculate temperature to match enthaply
				match salt_water_calculator::solve_temperature_from_entropy(entropy,pressure,x_nacl) {
					Ok(value) => temperature=value,
					Err(e) => return Err(COBIAError::Message(e))
				}
			},
		}
		//single phase at t and p, and overall composition
		if temperature.is_nan() {
			//temperature from MO
			material.get_overall_prop(&self.temperature,&self.empty_string,&mut self.scalar_property_value)?;
			temperature = self.scalar_property_value.value();
		} else {
			//put temperature on overall phase
			self.scalar_property_value.set(temperature);
			material.set_overall_prop(&self.temperature,&self.empty_string,&self.scalar_property_value)?;
		}
		if pressure.is_nan() {
			//pressure from MO
			material.get_overall_prop(&self.pressure,&self.empty_string,&mut self.scalar_property_value)?;
			pressure = self.scalar_property_value.value();
		} else {
			//put pressure on overall phase
			self.scalar_property_value.set(pressure);
			material.set_overall_prop(&self.pressure,&self.empty_string,&self.scalar_property_value)?;
		}
		//check that result is within operating region of the calculator - assume this is [0,120]C and [0,12]MPa
		// outside of these ranges, the enthapy and entropy calculations are not valid
		if temperature<273.15 || temperature>393.15 {
			return Err(COBIAError::Message("Temperature outside of supported range of [0,120] °C".to_string()));
		}
		if pressure<0.0 || pressure>12.0e6 {
			return Err(COBIAError::Message("Pressure outside of supported range of [0,12] MPa".to_string()));
		}
		//set phase list on MO
		self.phase_list.resize(1);
		self.phase_list[0].set_string("Liquid");
		self.phase_status.resize(1,cape_open_1_2::CapePhaseStatus::CapeUnknownphasestatus);
		self.phase_status[0]=cape_open_1_2::CapePhaseStatus::CapeAtequilibrium;
		material.set_present_phases(&self.phase_list,&self.phase_status)?;		
		//set temperature
		self.scalar_property_value.set(temperature);
		material.set_single_phase_prop(&self.temperature,&self.phase_list[0],&self.empty_string,&self.scalar_property_value)?;
		//set pressure
		self.scalar_property_value.set(pressure);
		material.set_single_phase_prop(&self.pressure,&self.phase_list[0],&self.empty_string,&self.scalar_property_value)?;
		//set phase fraction
		self.scalar_property_value.set(1.0);
		material.set_single_phase_prop(&self.phase_fraction,&self.phase_list[0],&self.mole,&self.scalar_property_value)?;
		//get overall composition
		material.get_overall_prop(&self.fraction,&self.mole,&mut self.property_value)?;
		//set phase composition
		material.set_single_phase_prop(&self.fraction,&self.phase_list[0],&self.mole,&self.property_value)?;
		//done
		Ok(())
    }

	/// Check whether a particular phase equilibrium calculation is supported.
	///
	/// # Arguments
	/// * `specification1` - The first specification for the phase equilibrium, which is specified through a CapeStringIn object
	/// * `specification2` - The second specification for the phase equilibrium, which is specified through a CapeStringIn object
	/// * `solution_type` - The type of solution to be used for the phase equilibrium calculation, which is specified through a CapeStringIn object; only the "Unspecified" solution type is supported.
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not, and if successful, whether the phase equilibrium calculation is supported.
    fn check_equilibrium_spec(&mut self,specification1:&CapeArrayStringIn,specification2:&CapeArrayStringIn,solution_type:&CapeStringIn) -> Result<CapeBoolean,COBIAError> {
		self.check_context_material()?;
		let material=self.material.as_ref().unwrap(); //cannot be None, see check_context_material
		material.get_present_phases(&mut self.phase_list,&mut self.phase_status)?;
		if self.phase_list.size()!=1 || self.liquid!=self.phase_list[0] {
			return Ok(false as CapeBoolean); //only flashes that allow liquid are supported
		}
        match PhaseEquilibriumType::new(specification1,specification2,solution_type) {
			Ok(_) => Ok(true as CapeBoolean), //supported
			Err(_) => Ok(false as CapeBoolean), //not supported
		}
    }
}


/// The ICapeThermoPropertyRoutine interface can optionally be implemented by a property package to allow
/// a client to obtain values for universal constants
impl cape_open_1_2::ICapeThermoUniversalConstant for SaltWaterPropertyPackage {

	/// Get the value of a universal constant.
	///
	/// for get_universal_constant, performance is not critical. If performance is critical, one should
	///  not convert to string, but compare the CapeConstString directly, which is pre-encoded,
	///  or build a HashMap using CapeConstString as key.
	///
	/// # Arguments
	/// * `constant_id` - The ID of the universal constant to be retrieved, which is specified through a CapeStringIn object
	/// * `constant_value` - The value of the universal constant, which is returned through a CapeValueOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
    fn get_universal_constant(&mut self,constant_id:&CapeStringIn,constant_value:&mut CapeValueOut) -> Result<(),COBIAError> {
		match constant_id.as_string().to_lowercase().as_str() {
			"avogadroconstant" => {constant_value.set_real(6.02214199e23)?;Ok(())},
			"boltzmannconstant" => {constant_value.set_real(1.3806503e-23)?;Ok(())},
			"idealgasstatereferencepressure" => {constant_value.set_real(101325.0)?;Ok(())},
			"molargasconstant" => {constant_value.set_real(8.314472)?;Ok(())},
			"speedoflightinvacuum" => {constant_value.set_real(299792458.0e8)?;Ok(())},
			"standardaccelerationofgravity" => {constant_value.set_real(9.80665)?;Ok(())},
			_ => Err(COBIAError::Message(format!("Unknown universal constant: {}",constant_id)))
		}        
    }

	/// Get the list of universal constants supported by the property package.
	///
	/// # Arguments
	/// * `constant_id_list` - The list of supported universal constants, which is returned through a CapeArrayStringOut object
	///
	/// # Returns
	/// * `Result` - A result object that indicates whether the operation was successful or not
    fn get_universal_constant_list(&mut self,constant_id_list:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
		constant_id_list.put_array(&["avogadroConstant","boltzmannConstant","idealGasStateReferencePressure","molarGasConstant","speedOfLightInVacuum","standardAccelerationOfGravity"])?;
		Ok(())
    }
}

