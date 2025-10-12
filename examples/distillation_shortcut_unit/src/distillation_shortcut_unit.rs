use cobia::cape_open_1_2::ICapeUnitPort;
use cobia::*;
use std::collections::HashSet;
use std::io::Write;
use std::cell::RefCell;
use std::default::Default;
use chrono;
use crate::shared_unit_data::*;
use crate::port_collection::PortCollection;
use crate::material_port::MaterialPort;
use crate::parameter_collection::ParameterCollection;
use crate::real_parameter::RealParameter;
use crate::integer_parameter::IntegerParameter;
use crate::string_parameter::StringParameter;

#[cfg(target_os = "windows")]
use crate::gui;


//use cobia::prelude::*;

///The DistillationShortcutUnit is an example of a Unit Operation that implements the 
/// CAPE-OPEN 1.2 standard.
///
/// A CAPE-OPEN 1.2 Unit Operation must implemented the following interfaces:
/// - ICapeIdentification
/// - ICapeUtilities
/// - ICapeUnit
/// - ICapeReport (optional)
///
/// The unit operation is creatable; the public CAPE-OPEN class factory is implemented in lib.rs;
/// to facilitate the registration of this object into the COBIA registry, the object implements
/// the PMCRegisterationInfo trait.
///
/// Once registered, the package can be instantiated and created in all CAPE-OPEN
/// compliant simulators.

#[cape_object_implementation(
		interfaces = {
			cape_open_1_2::ICapeUtilities,
			cape_open_1_2::ICapeIdentification,
			cape_open_1_2::ICapeUnit,
			cape_open_1_2::ICapeReport,
			cape_open_1_2::ICapePersist,
		},
		new_arguments= {} //create through the ::new function (without arguments)
  )]
pub struct DistillationShortcutUnit {
	/// Shared data for the unit, containing unit-specific information
	shared_unit_data: SharedUnitDataRef,
	/// The description of the unit operation
	description: CapeStringImpl,
	/// The name of the last run report
	last_run_report_name: CapeStringImpl,
	/// The content of the last run report
	last_run_report : String,
	/// The collection of ports for this unit operation
	port_collection: cape_open_1_2::CapeCollection<cape_open_1_2::CapeUnitPort>,
	/// The feed port of the unit operation
	feed : cape_open_1_2::CapeUnitPort,
	/// The distillate (top) product port of the unit operation
	distillate_product : cape_open_1_2::CapeUnitPort,
	/// The bottom product port of the unit operation
	bottom_product : cape_open_1_2::CapeUnitPort,
	/// The collection of parameters for this unit operation
	parameter_collection: cape_open_1_2::CapeCollection<cape_open_1_2::CapeParameter>,
	/// The IDs of the compounds in the feed material object
	compound_ids : CapeArrayStringVec,
	/// The names of the compounds in the feed material object
	compound_names : CapeArrayStringVec,
	/// Paramete for selection of the light key compound
	light_key_compound : cape_open_1_2::CapeStringParameter,
	/// Parameter for selection of the heavy key compound
	heavy_key_compound : cape_open_1_2::CapeStringParameter,
	/// Parameter for recovery of the light key compound
	light_key_compound_recovery : cape_open_1_2::CapeRealParameter,
	/// Parameter for recovery of the heavy key compound
	heavy_key_compound_recovery : cape_open_1_2::CapeRealParameter,
	/// Parameter to specify factor of reflux ratio above minimum reflux ratio
	reflux_ratio_factor : cape_open_1_2::CapeRealParameter,
	/// Parameter to specify the maximum number of iterations
	maximum_iterations : cape_open_1_2::CapeIntegerParameter,
	/// Parameter to specify the convergence tolerance
	convergence_tolerance : cape_open_1_2::CapeRealParameter,
	/// Number of stages result
	number_of_stages : cape_open_1_2::CapeRealParameter,
	/// Reflux ratio result
	reflux_ratio : cape_open_1_2::CapeRealParameter,
	//feed stage location result
	feed_stage_location : cape_open_1_2::CapeRealParameter,
	//index of light key compound in the compound list; determined during validation and used in calculation
	light_key_compound_index : i32, 
	//index of heavy key compound in the compound list; determined during validation and used in calculation
	heavy_key_compound_index : i32,
	//vapor phase ID, determined during validation and used in calculation
	vapor_phase_id : CapeStringImpl, 
	//liquid phase IDs, determined during validation and used in calculation
	liquid_phase_ids : CapeArrayStringVec,
	//The IDs of the compounds in the feed material object
	phase_ids : CapeArrayStringVec,
	//diagnostics interface of the simulation context
	diagnostics : Option<cape_open_1_2::CapeDiagnostic>,
}

impl DistillationShortcutUnit {
	/// The default name of the unit operation
	pub(crate) const NAME: &'static str = "Distillation ShortCut";
	/// The default description of the unit operation
	const DESCRIPTION: &'static str = "Distillation ShortCut unit operation based on Fenske-Underwood-Gilliland-Kirkbride method";
	/// The ProgID of the unit operation, used for registration in the COBIA registry
	const PROGID: &'static str = "DistillationShortcut.DistillationShortcutUnit";

	/// Creates a new instance of the DistillationShortcutUnit.
	///
	/// # Returns:
	/// * A new instance of the DistillationShortcutUnit.

	fn new() -> Self {
		let shared_unit_data : SharedUnitDataRef = std::rc::Rc::new(RefCell::new(SharedUnitData::default()));
		let dimensionless = vec!(0.0,0.0,0.0); //trailing zeroes can be omitted, but some PMEs do not like less than 3 elements 
		let fractional = vec!(0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,1.0); //dimensionless but relative
		let mut unit_operation=Self {
			cobia_object_data: Default::default(), //this member is generated by cape_object_implementation and can be set to default()
			description: CapeStringImpl::from_string(Self::DESCRIPTION),
			last_run_report_name: CapeStringImpl::from_string("Calculation Report"),
			last_run_report : "Report not available".to_string(),
			port_collection : PortCollection::create(shared_unit_data.clone()),
			feed : MaterialPort::create(
					CapeStringImpl::from(format!("Feed")),
					CapeStringImpl::from(format!("Feed port of {}",Self::NAME)),
					true,
					shared_unit_data.clone()),
			distillate_product : MaterialPort::create(
					CapeStringImpl::from(format!("Distillate product")),
					CapeStringImpl::from(format!("Distillate (light) product port of {}",Self::NAME)),
					false,
					shared_unit_data.clone()),
			bottom_product : MaterialPort::create(
					CapeStringImpl::from(format!("Bottom product")),
					CapeStringImpl::from(format!("Bottom (heavy) product port of {}",Self::NAME)),
					false,
					shared_unit_data.clone()),
			shared_unit_data:shared_unit_data.clone(),
			parameter_collection : ParameterCollection::create(shared_unit_data.clone()),
			compound_ids : CapeArrayStringVec::new(),
			compound_names : CapeArrayStringVec::new(),
			light_key_compound : StringParameter::create(
				CapeStringImpl::from(format!("Light key compound")),
				CapeStringImpl::from(format!("Name of the light key compound")),
				true,
				shared_unit_data.clone(),
				CapeStringImpl::from(""),
				Some(CapeArrayStringVec::new()),
				false,
			),
			heavy_key_compound : StringParameter::create(
				CapeStringImpl::from(format!("Heavy key compound")),
				CapeStringImpl::from(format!("Name of the heavy key compound")),
				true,
				shared_unit_data.clone(),
				CapeStringImpl::from(""),
				Some(CapeArrayStringVec::new()),
				false,
			),
			light_key_compound_recovery : RealParameter::create(
				CapeStringImpl::from(format!("Light component recovery")),
				CapeStringImpl::from(format!("Fraction of light component that goes to distillate product")),
				true,
				shared_unit_data.clone(),
				f64::NAN,
				0.0,
				1.0,
				fractional.clone()
				),
			heavy_key_compound_recovery : RealParameter::create(
				CapeStringImpl::from(format!("Heavy component recovery")),
				CapeStringImpl::from(format!("Fraction of heavy component that goes to bottoms product")),
				true,
				shared_unit_data.clone(),
				f64::NAN,
				0.0,
				1.0,
				fractional.clone()
				),
			reflux_ratio_factor : RealParameter::create(
				CapeStringImpl::from(format!("Reflux ratio factor")),
				CapeStringImpl::from(format!("Factor of reflux ratio above minimum reflux ratio")),
				true,
				shared_unit_data.clone(),
				1.15,
				1.0,
				f64::NAN,
				dimensionless.clone()
				),
			maximum_iterations : IntegerParameter::create(
				CapeStringImpl::from(format!("Maximum iterations")),
				CapeStringImpl::from(format!("Maximum number of iterations")),
				true,
				shared_unit_data.clone(),
				20,
				1,
				1000,
			),
            convergence_tolerance : RealParameter::create(
                CapeStringImpl::from(format!("Relative tolerance")),
                CapeStringImpl::from(format!("Tolerance of product component flow rates w.r.t. total feed flow rate")),
                true,
                shared_unit_data.clone(),
                1e-6,
                1e-2,
                1e-12,
                dimensionless.clone()
            ),
			number_of_stages : RealParameter::create(
				CapeStringImpl::from(format!("Number of stages")),
				CapeStringImpl::from(format!("Estimated number of stages in the column")),
				false,
				shared_unit_data.clone(),
				f64::NAN,
				f64::NAN,
				f64::NAN,
				dimensionless.clone()
				),
			reflux_ratio : RealParameter::create(
				CapeStringImpl::from(format!("Reflux ratio")),
				CapeStringImpl::from(format!("Estimated reflux ratio")),
				false,
				shared_unit_data.clone(),
				f64::NAN,
				f64::NAN,
				f64::NAN,
				fractional.clone()
				),
			feed_stage_location : RealParameter::create(
				CapeStringImpl::from(format!("Feed stage location")),
				CapeStringImpl::from(format!("Estimated feed stage location in the column")),
				false,
				shared_unit_data.clone(),
				f64::NAN,
				f64::NAN,
				f64::NAN,
				dimensionless.clone()
				),
			light_key_compound_index : -1,
			heavy_key_compound_index : -1,
			vapor_phase_id : CapeStringImpl::new(),
            liquid_phase_ids : CapeArrayStringVec::new(),
			phase_ids : CapeArrayStringVec::new(),
            diagnostics : None,
		};
		//add ports to collection
		let port_collection=unsafe {PortCollection::borrow_mut(&mut unit_operation.port_collection)};
		port_collection.add_port(unit_operation.feed.clone());
		port_collection.add_port(unit_operation.distillate_product.clone());
		port_collection.add_port(unit_operation.bottom_product.clone());
		//add parameters to the collection
		let parameter_collection=unsafe {ParameterCollection::borrow_mut(&mut unit_operation.parameter_collection)};
		parameter_collection.add_parameter(unit_operation.light_key_compound.clone());
		parameter_collection.add_parameter(unit_operation.heavy_key_compound.clone());
		parameter_collection.add_parameter(unit_operation.light_key_compound_recovery.clone());
		parameter_collection.add_parameter(unit_operation.heavy_key_compound_recovery.clone());
		parameter_collection.add_parameter(unit_operation.reflux_ratio_factor.clone());
		parameter_collection.add_parameter(unit_operation.maximum_iterations.clone());
		parameter_collection.add_parameter(unit_operation.convergence_tolerance.clone());
		parameter_collection.add_parameter(unit_operation.number_of_stages.clone());
		parameter_collection.add_parameter(unit_operation.reflux_ratio.clone());
		parameter_collection.add_parameter(unit_operation.feed_stage_location.clone());
		//return the unit
		return unit_operation
	}

	pub fn get_name(&self) -> String {
		self.shared_unit_data.borrow().name.to_string()
	}

	pub fn set_name(&mut self, name: &str) {
		let mut borrowed_conf = self.shared_unit_data.borrow_mut();
		borrowed_conf.name.set_string(name);
	}

	pub fn get_description(&self) -> String {
		self.description.to_string()
	}

	pub fn set_description(&mut self, desc: &str) {
		self.description.set_string(desc);
	}

	pub fn get_compound_list(&self) -> Vec<String> {
		let mut compounds=Vec::new();
		for name in self.compound_names.iter() {
			compounds.push(name.to_string());
		}
		compounds
	}

	pub fn get_light_key_compound(&self) -> String {
		let mut name=cobia::CapeStringImpl::new();
		self.light_key_compound.get_value(&mut name).unwrap();
		name.to_string()
	}

	pub fn set_light_key_compound(&mut self, name: &str) -> Result<(),COBIAError> {
		self.light_key_compound.set_value(&CapeStringImpl::from_string(name))
	}

	pub fn get_heavy_key_compound(&self) -> String {
		let mut name=cobia::CapeStringImpl::new();
		self.heavy_key_compound.get_value(&mut name).unwrap();
		name.to_string()
	}

	pub fn set_heavy_key_compound(&mut self, name: &str) -> Result<(),COBIAError> {
		self.heavy_key_compound.set_value(&CapeStringImpl::from_string(name))
	}

	pub fn get_light_key_compound_recovery(&self) -> f64 {
		self.light_key_compound_recovery.get_value().unwrap()
	}

	pub fn set_light_key_compound_recovery(&mut self, recovery: f64) -> Result<(),COBIAError> {
		self.light_key_compound_recovery.set_value(recovery)
	}

	pub fn get_heavy_key_compound_recovery(&self) -> f64 {
		self.heavy_key_compound_recovery.get_value().unwrap()
	}

	pub fn set_heavy_key_compound_recovery(&mut self, recovery: f64) -> Result<(),COBIAError> {
		self.heavy_key_compound_recovery.set_value(recovery)
	}

	pub fn get_reflux_ratio_factor(&self) -> f64 {
		self.reflux_ratio_factor.get_value().unwrap()
	}

	pub fn set_reflux_ratio_factor(&mut self, factor: f64) -> Result<(),COBIAError> {
		self.reflux_ratio_factor.set_value(factor)
	}

	pub fn get_maximum_iterations(&self) -> i32 {
		self.maximum_iterations.get_value().unwrap()
	}

	pub fn set_maximum_iterations(&mut self, iterations: i32) -> Result<(),COBIAError> {
		self.maximum_iterations.set_value(iterations)
	}

	pub fn get_convergence_tolerance(&self) -> f64 {
		self.convergence_tolerance.get_value().unwrap()
	}

	pub fn set_convergence_tolerance(&mut self, tolerance: f64) -> Result<(),COBIAError> {
		self.convergence_tolerance.set_value(tolerance)
	}

	pub fn get_number_of_stages(&self) -> f64 {
		self.number_of_stages.get_value().unwrap()
	}

	pub fn get_reflux_ratio(&self) -> f64 {
		self.reflux_ratio.get_value().unwrap()
	}

	pub fn get_feed_stage_location(&self) -> f64 {
		self.feed_stage_location.get_value().unwrap()
	}

	pub fn get_ports(&self) -> Vec<cape_open_1_2::CapeUnitPort> {
		vec!(
			cape_open_1_2::CapeUnitPort::from_object(&self.feed).unwrap(),
			cape_open_1_2::CapeUnitPort::from_object(&self.distillate_product).unwrap(),
			cape_open_1_2::CapeUnitPort::from_object(&self.bottom_product).unwrap(),
		)
	}

}

impl std::fmt::Display for DistillationShortcutUnit {

	/// Format the DistillationShortcutUnit as a string for display purposes.
	///
	/// The std::fmt::Display interface is used when generating the 
	/// source name of the object that raises an error.
	///
	/// # Arguments:
	/// * `f` - A mutable reference to a `std::fmt::Formatter` where the formatted string will be written.
	///
	/// # Returns:
	/// * A `std::fmt::Result` indicating the success or failure of the formatting operation.

	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f,"{} unit \"{}\"", Self::NAME, self.shared_unit_data.borrow().name)
    }
}

impl PMCRegisterationInfo for DistillationShortcutUnit {

	/// Get the UUID of the DistillationShortcutUnit.
	///
	/// This is the unique identifier for the DistillationShortcutUnit that
	/// is used for registration in the COBIA registry.
	///
	/// # Returns:
	/// * A `CapeUUID` representing the unique identifier of the DistillationShortcutUnit.

	fn get_uuid() -> CapeUUID {
		//{8EE9C111-484F-4E94-B14E-C24C70DA1384}
		CapeUUID::from_slice(&[0x8Eu8,0xE9u8,0xC1u8,0x11u8,0x48u8,0x4Fu8,0x4Eu8,0x94u8,0xB1u8,0x4Eu8,0xC2u8,0x4Cu8,0x70u8,0xDAu8,0x13u8,0x84])
	}

	/// Provide registration details for the DistillationShortcutUnit.
	///
	/// This entry point is called by the self-registration entry
	/// point to register the DistillationShortcutUnit in the COBIA registry.
	///
	/// # Arguments:
	/// * `registrar` - A reference to the `CapeRegistrar` used for registration.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure of the registration process.

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
		registrar.add_cat_id(&cape_open::CATEGORYID_UNITOPERATION)?;
		registrar.add_cat_id(&cape_open_1_2::CATEGORYID_COMPONENT_1_2)?;
		Ok(())
	}
}

impl cape_open_1_2::ICapeIdentification for DistillationShortcutUnit {

	/// Get the name of the component.
	///
	/// # Arguments:
	/// * `name` - A mutable reference to a `CapeStringOut` where the name will be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the name is set in `name`.

	fn get_component_name(&mut self,name:&mut CapeStringOut) -> Result<(), COBIAError> {
		name.set(&self.shared_unit_data.borrow().name)?;
		Ok(())
	}

	/// Get the description of the component.
	///
	/// # Arguments:
	/// * `description` - A mutable reference to a `CapeStringOut` where the description will be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the description is set in `description`.

	fn get_component_description(&mut self,description:&mut CapeStringOut) -> Result<(), COBIAError> {
		description.set(&self.description)?;
		Ok(())
	}

	/// Set the name of the component.
	///
	/// A primary PMC object must allow its name to be changed by the PME.
	///
	/// # Arguments:
	/// * `name` - A reference to a `CapeStringIn` containing the new name to be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the name is set in the shared unit data.

	fn set_component_name(&mut self, name: &CapeStringIn) -> Result<(), COBIAError> {
		let mut borrowed_conf = self.shared_unit_data.borrow_mut();
		borrowed_conf.name.set(name);
		Ok(())
	}

	/// Set the description of the component.
	///
	/// A primary PMC object should allow its description to be changed by the PME.
	///
	/// # Arguments:
	/// * `desc` - A reference to a `CapeStringIn` containing the new description to be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the description is set in the shared unit data.

	fn set_component_description(&mut self, desc: &CapeStringIn) -> Result<(), COBIAError> {
		self.description.set(desc);
		Ok(())
	}
}

impl cape_open_1_2::ICapeUtilities for DistillationShortcutUnit {

	/// Get the parameter collection of the unit operation.
	///
	/// ICapeUtilities is implemented by all PMC objects, but not all of them have parameters.
	///
	/// An object that does not have parameters can return an empty collection, or simply an error.
	///
	/// This unit operation does have a parameter collection, which is returned by this method.
	///
	/// # Returns:
	/// * A `Result` containing a `CapeCollection` of `CapeParameter` objects representing the parameters of the unit operation.
	
	fn get_parameters(&mut self) -> Result<cape_open_1_2::CapeCollection<cape_open_1_2::CapeParameter>,COBIAError> {
		Ok(self.parameter_collection.clone())
	}

	/// Entry point for the PME to set the simulation context.
	///
	/// The simulation context offers a number of services, including
	/// the ability to log messages to the simulator log.
	///
	/// If it exposes a Diagnostics interface, it will be used to log warnings.
	///
	/// # Arguments:
	/// * `_` - A `CapeSimulationContext` object representing the simulation context to be set.
	///
	/// # Returns:
	/// * 'Ok'

	fn set_simulation_context(&mut self,simulation_context:cape_open_1_2::CapeSimulationContext) -> Result<(),COBIAError> {
		self.diagnostics = None;
		if let Ok(diag) = cape_open_1_2::CapeDiagnostic::from_object(&simulation_context) {
			self.diagnostics=Some(diag);
		}
		Ok(())
	}

	/// Initialize is called by the PME to initialize the unit operation.
	///
	/// Initialize must be called prior to any other method calls, except
	/// depersistence and setting the simulation context.
	///
	/// If initialize fails, the PME must not call any other methods on 
	/// the PMC, including terminate. If initialize succeeds, the PME
	/// must call terminate before the PMC is destroyed.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure of the initialization process.

	fn initialize(&mut self) -> Result<(),COBIAError> {
		Ok(()) //nothing to do
	}

	/// Terminate is called by the PME to terminate the unit operation.
	///
	/// Must be called by the PME before the PMC is destroyed.
	///
	/// Upon termination, the PMC must drop all external references.
	///
	/// # Returns:
	///
	/// * A `Result` indicating success or failure of the termination process.

	fn terminate(&mut self) -> Result<(),COBIAError> {
		//disconnect all ports
		{
			let port_collection=unsafe{PortCollection::borrow_mut(&mut self.port_collection)};
			for port in port_collection.iter_mut () {
				let port=unsafe{MaterialPort::borrow_mut(port)};
				let _ = (*port).disconnect();
			}
		}
		//release diagnostics interface
		self.diagnostics=None;
		Ok(())
	}
	
	/// Edit is called by the PME to edit the unit operation.
	///
	/// The only place where the PMC may show a modal dialog is in this method.
	///
	/// After returning from this method, in case of a CapeModified return value,
	/// the PME must re-obtain port and parameter details. After initialization, 
	/// port and parameter collections and meta-data on ports and parameters
	/// may only change in this method.
	///
	/// # Arguments:
	/// * `_` - A `CapeWindowId` representing the window ID of the PME that is calling this method.
	///
	/// # Returns:
	/// * A `Result` containing a `CapeEditResult` indicating the result of the edit operation.

	fn edit(&mut self,parent:CapeWindowId) -> Result<cape_open_1_2::CapeEditResult,COBIAError> {
		#[cfg(target_os = "windows")]
		{
			let mut unit_dlg=gui::UnitDialogHandler::new(parent,self);
			match unit_dlg.show() {
				Ok(_) => {
					Ok(
						if unit_dlg.get_handler().is_modified() {
							self.shared_unit_data.borrow_mut().validation_status=cape_open_1_2::CapeValidationStatus::CapeNotValidated;
							cape_open_1_2::CapeEditResult::CapeModified
						} else {
							cape_open_1_2::CapeEditResult::CapeNotModified
						}
					)
				},
				Err(e) => {Err(COBIAError::Message(e.to_string()))}
			}
		}
		#[cfg(not(target_os = "windows"))]
		{
			//Edit functionality is currently only supplied for Windows,
			// pending extension of the html_dialog module with linux
			// and MacOS support
			Err(COBIAError::Code(COBIAERR_NOTIMPLEMENTED))
		}
	}
}

impl DistillationShortcutUnit {

	/// Internal method for validation of the unit operation.
	///
	/// During validation, the unit operation checks that:
	/// * The feed port is connected to a material object,
	/// * The distillate product port is connected to a material object,
	/// * The bottom product port is connected to a material object,
	/// * The material objects connected to the feed, distillate product, and bottom product ports have the same compounds,
	/// * The heavy key compound is selected and valid; its index is determined as well,
	/// * The light key compound is selected and valid; its index is determined as well,
	/// * The light and heavy key compound selections are not equal,
	/// * A vapor phase is defined; the vapor phase ID is determined as well,
	/// * All parameters are valid.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure of the validation process.

	pub fn validate_internal(&mut self) -> Result<(),COBIAError> {
		//check that the ports are connnected
		// performance note: the following can be re-used between calls if stored as distillation_shortcut_unit class members
		let mut product_compound_ids = CapeArrayStringVec::new();
		let mut product_compound_names = CapeArrayStringVec::new();
		let mut compound_formulae = CapeArrayStringVec::new();
		let mut boil_temps = CapeArrayRealVec::new();
		let mut molwts = CapeArrayRealVec::new();
		let mut casnos = CapeArrayStringVec::new();
		let mut states_of_aggregation = CapeArrayStringVec::new();
		let mut key_compound_ids = CapeArrayStringVec::new();
		let mut product_phase_ids = CapeArrayStringVec::new();
		let mut port_error : Option<COBIAError> = None; //cache port errors until we have the compound list
		let vapor = CapeStringImpl::from_string("vapor");
        let liquid = CapeStringImpl::from_string("liquid");
		//check feed
		{
			let fd=unsafe{MaterialPort::borrow(&self.feed)};
			match fd.get_connected_material() {
				None => {
					port_error= Some(COBIAError::Message("Feed port is not connected".into()));
				},
				Some(mo) => {
					let compounds=cape_open_1_2::CapeThermoCompounds::from_object(&mo)?;
					compounds.get_compound_list(
							&mut self.compound_ids,
							&mut compound_formulae,
							&mut self.compound_names,
							&mut boil_temps,
							&mut molwts,
							&mut casnos)?;
					if self.compound_ids.size()==0 && port_error.is_none() {
						port_error= Some(COBIAError::Message("Material Object connected to feed does not contain any compounds".into()));
					}
					//determine which is the vapor phase ID
					// we assume that the vapor phase ID is the same for the feed, distillate product, and bottom product material objects
					let phases=cape_open_1_2::CapeThermoPhases::from_object(&mo)?;
					phases.get_phase_list(&mut self.phase_ids,&mut states_of_aggregation,&mut key_compound_ids)?;
					if self.phase_ids.size()!=states_of_aggregation.size() && port_error.is_none() {
						if port_error.is_none() {
							port_error= Some(COBIAError::Message("Material Object connected to feed returns inconsistent number of phases and states of aggregation".into()));
						}
					} else {
						//find vapor phase
						self.vapor_phase_id.set_string("");
						self.liquid_phase_ids.resize(0);
						for (i, state_of_aggregation) in states_of_aggregation.iter().enumerate() {
							if state_of_aggregation.eq_ignore_case(&vapor) {
								if !self.vapor_phase_id.is_empty() && port_error.is_none() {
									port_error= Some(COBIAError::Message("Material Object connected to feed defines more than one vapor phase".into()));
                                } else {
                                    self.vapor_phase_id.set(&self.phase_ids[i]);
                                }
                            } else if state_of_aggregation.eq_ignore_case(&liquid) {
								let size=self.liquid_phase_ids.size();
                                self.liquid_phase_ids.resize(size+1);
                                self.liquid_phase_ids[size].set(&self.phase_ids[i]);
                            }
                        }
						if self.vapor_phase_id.is_empty() && port_error.is_none() {
							port_error= Some(COBIAError::Message("Material Object connected to feed does not define vapor phase".into()));
						}
						if self.liquid_phase_ids.is_empty() && port_error.is_none() {
                            port_error= Some(COBIAError::Message("Material Object connected to feed does not define any liquid phase".into()));
						}
					}
				}
			};
		}
		//check distillate product
		if port_error.is_none() {
			let tp=unsafe{MaterialPort::borrow(&self.distillate_product)};
			match tp.get_connected_material() {
				None => {
					port_error=Some(COBIAError::Message("Distillate product port is not connected".into()));
				},
				Some(mo) => {
					//get compounds
					let compounds=cape_open_1_2::CapeThermoCompounds::from_object(&mo)?;
					compounds.get_compound_list(
							&mut product_compound_ids,
							&mut compound_formulae,
							&mut product_compound_names,
							&mut boil_temps,
							&mut molwts,
							&mut casnos)?;
					//check consistent compound IDs
					if product_compound_ids!=self.compound_ids {
						port_error=Some(COBIAError::Message("Material objects connected to feed and top product do not have the same compounds".into()));
					}
					//get phases
					let phases=cape_open_1_2::CapeThermoPhases::from_object(&mo)?;
					phases.get_phase_list(&mut product_phase_ids,&mut states_of_aggregation,&mut key_compound_ids)?;
					//check consistent phase IDs
					if product_phase_ids!=self.phase_ids && port_error.is_none() {
						port_error=Some(COBIAError::Message("Material objects connected to feed and top product do not have the same phases".into()));
					}
				}
			};
		}
		//check bottom product
		if port_error.is_none() {
			let bp=unsafe{MaterialPort::borrow(&self.bottom_product)};
			match bp.get_connected_material() {
				None => {
					port_error=Some(COBIAError::Message("Bottom product port is not connected".into()));
				},
				Some(mo) => {
					//get compounds
					let compounds=cape_open_1_2::CapeThermoCompounds::from_object(&mo)?;
					compounds.get_compound_list(
							&mut product_compound_ids,
							&mut compound_formulae,
							&mut product_compound_names,
							&mut boil_temps,
							&mut molwts,
							&mut casnos)?;
					//check consistent compound IDs
					if product_compound_ids!=self.compound_ids {
						port_error=Some(COBIAError::Message("Material objects connected to feed and bottom product do not have the same compounds".into()));
					}
					//get phases
					let phases=cape_open_1_2::CapeThermoPhases::from_object(&mo)?;
					phases.get_phase_list(&mut product_phase_ids,&mut states_of_aggregation,&mut key_compound_ids)?;
					//check consistent phase IDs
					if product_phase_ids!=self.phase_ids && port_error.is_none() {
						port_error=Some(COBIAError::Message("Material objects connected to feed and bottom product do not have the same phases".into()));
					}
				}
			};
		}
		//add choice list to the light and heavy key compound parameters
		unsafe { StringParameter::borrow_mut(&mut self.light_key_compound).set_possible_values(Some(&self.compound_names)) };
		unsafe { StringParameter::borrow_mut(&mut self.heavy_key_compound).set_possible_values(Some(&self.compound_names)) };
		//raise port error if any
		if let Some(e) = port_error {
			return Err(e);
		}
		//check that all parameters are ok
		{
			let parameter_collection=unsafe{ParameterCollection::borrow_mut(&mut self.parameter_collection)};
			for parameter in parameter_collection.iter_mut() {
				let mut msg=CapeStringImpl::new();
				if parameter.validate(&mut msg).unwrap()==(false as CapeBoolean) {
					//parameter is not valid, return error
					let mut name=CapeStringImpl::new();
					let iden=cape_open_1_2::CapeIdentification::from_object(parameter)?;
					iden.get_component_name(&mut name)?;
					return Err(COBIAError::Message(format!("Parameter {} is not valid: {}",name,msg)));
				}
			}
		}
		//determine the light key compound index in the compound list
		{
			let light_key_compound_parameter=unsafe { StringParameter::borrow(&mut self.light_key_compound)};
			let light_key_compound_name = light_key_compound_parameter.value();
			self.light_key_compound_index=-1;
			for (i,compound_name) in self.compound_names.iter().enumerate() {
				if compound_name.eq_ignore_case(light_key_compound_name) {
					self.light_key_compound_index=i as i32;
					break;
				}
			}
			if self.light_key_compound_index==-1 {
				return Err(COBIAError::Message(format!("Light key compound '{}' is not defined feed material object",light_key_compound_name)));
			}
		}
		//determine the heavy key compound index in the compound list
		{
			let heavy_key_compound_parameter=unsafe { StringParameter::borrow(&mut self.heavy_key_compound)};
			let heavy_key_compound_name = heavy_key_compound_parameter.value();
			self.heavy_key_compound_index=-1;
			for (i,compound_name) in self.compound_names.iter().enumerate() {
				if compound_name.eq_ignore_case(heavy_key_compound_name) {
					self.heavy_key_compound_index=i as i32;
					break;
				}
			}
			if self.heavy_key_compound_index==-1 {
				return Err(COBIAError::Message(format!("Heavy key compound '{}' is not defined feed material object",heavy_key_compound_name)));
			}
		}
		//check that the heay and light key compound selections are not equal
		if self.light_key_compound_index==self.heavy_key_compound_index {
			return Err(COBIAError::Message("Light and heavy key compound selections must be different".into()));
		} 
		//valid
		return Ok(());
	}

	/// Calculate dew or bubble point, and the K values
	///
	/// #Arguments:
	/// * `stream_name` - The name of the stream for which the dew or bubble point is calculated.
	/// * `material_object` - The material object for which the dew or bubble point is calculated.
	/// * `material_equilibrium_routine` - The material equilibrium routine to be used for the calculation.
	/// * `flow_rates` - The flow rates for the material object.
	/// * `pressure_value` - The pressure value for the material object.
	/// * `phase_fraction_value` - The vapor phase fraction for the material object (0 for bubble point, 1 for dew point).
	/// * `pressure` - The string literal 'pressure'
	/// * `phase_fraction` - The string literal 'phase_fraction'
	/// * `fraction` - The string literal 'fraction'
	/// * `flow` - The string literal 'flow'
	/// * `mole` - The string literal 'mole'
	/// * `no_basis` - Empty string literal, used to indicate that no basis applies to a property
	/// * `vapor_composition` - The vapor composition buffer, re-used
	/// * `liquid_composition` - The liquid composition buffer, re-used
	/// * `present_phases` - The present phases buffer, re-used
	/// * `present_phase_status` - The present phase status buffer, re-used
	/// * `pressure_condition` - The flash condition that identifies overall pressure,
	/// * `vapor_fraction_condition` - The flash condition that identifies vapor phase fraction,
	/// * `solution_type` - The solution type to be used for the calculation, the string literal Unspecified
	/// * `phase_status_unspecified` - The phase status buffer, re-used, used to indicate that the phase status is unspecified
	/// * `k_values` - Receives the K values at the phase boundary
	///
	/// # Returns:
	/// * A `Result` indicating success or failure of the calculation process.
	fn calculate_dew_or_bubble_point(
		&mut self,
		stream_name:&str,
        material_object : &cape_open_1_2::CapeThermoMaterial,
		material_equilibrium_routine : &cape_open_1_2::CapeThermoEquilibriumRoutine,
		flow_rates:&CapeArrayRealVec,
		pressure_value:&CapeArrayRealScalar,
        phase_fraction_value:&CapeArrayRealScalar,
        pressure:&CapeStringImpl,
		phase_fraction:&CapeStringImpl,
        fraction:&CapeStringImpl,
		flow:&CapeStringImpl,
		mole:&CapeStringImpl,
		no_basis:&CapeStringImpl,
		vapor_composition:&mut CapeArrayRealVec,
		liquid_composition:&mut CapeArrayRealVec,
        present_phases:&mut CapeArrayStringVec,
		present_phase_status: &mut CapeArrayEnumerationVec<cape_open_1_2::CapePhaseStatus >,
		pressure_condition:&CapeArrayStringVec,
		vapor_fraction_condition:&CapeArrayStringVec,
		solution_type:&CapeStringImpl,
        phase_status_unspecified:&CapeArrayEnumerationVec::<cape_open_1_2::CapePhaseStatus>,
        k_values:&mut Vec<f64>,
	) -> Result<(),COBIAError> {
        //set current rate, pressure and vapor fraction, and flash to obtain the dew or bubble point
        material_object.set_overall_prop(flow, mole, flow_rates)?;
        material_object.set_overall_prop(pressure, no_basis, pressure_value)?;
        material_object.set_single_phase_prop(phase_fraction,&self.vapor_phase_id,mole,phase_fraction_value)?;
        material_object.set_present_phases(&self.phase_ids,phase_status_unspecified)?;
        match material_equilibrium_routine.calc_equilibrium(pressure_condition,vapor_fraction_condition,solution_type) {
            Ok (_) => { },
            Err (e) =>
            {
                return Err(COBIAError::Message(format!("Error calculating dew or bubble point for {}: {}", stream_name,e)));
            }
        }
		//validate that there is a liquid and a vapor phase, and get phase compositions
        material_object.get_present_phases(present_phases,present_phase_status)?;
        let mut has_vapor_phase = false;
		let mut has_liquid_phase = false;
		for phase_id in present_phases.iter() {
			if phase_id.eq_ignore_case(&self.vapor_phase_id) {
				has_vapor_phase = true;
                material_object.get_single_phase_prop(fraction, phase_id, mole,vapor_composition)?;
				if vapor_composition.size()!= self.compound_names.size() {
					return Err(COBIAError::Message("Vapor composition has invalid number of elements".into()));
				}
            } else {
				for liquid_phase_id in self.liquid_phase_ids.iter() {
					if phase_id.eq_ignore_case(liquid_phase_id) {
						has_liquid_phase = true;
						material_object.get_single_phase_prop(fraction, phase_id, mole,liquid_composition)?;
                        if liquid_composition.size()!= self.compound_names.size() {
                            return Err(COBIAError::Message("Liquid composition has invalid number of elements".into()));
                        }
						break; //no need to check further
					}
				}
			}
			if has_liquid_phase && has_vapor_phase {
				break; //no need to check further
			}
		}
		if !has_vapor_phase {
			return Err(COBIAError::Message(format!{"No vapor phase present at dew or bubble point for stream '{}'", stream_name}));
		}
		if !has_liquid_phase {
            return Err(COBIAError::Message(format!{"No liquid phase present at dew or bubble point for stream '{}'", stream_name}));
		}
		//get the K values for the distillate product
		k_values.clear();
        for (y,x) in vapor_composition.as_vec().iter().zip(liquid_composition.as_vec().iter()) {
			//avoid numerical errors by assuming a lower limit of 1e-15 mol/mol on composition
            k_values.push(f64::max(*y, 1e-15) / f64::max(*x, 1e-15)); // k = y/x
		}
		//ok
		Ok(())
	}

	/// Calculate overall enthalpy at the current phase equilibrium
	///
	/// # Arguments:
	/// * `material_object` - The material object for which the overall enthalpy is calculated.
	/// * `material_calculation_routine` - The material calculation routine to be used for the calculation.
	/// * `present_phases` - The present phases buffer, re-used.
	/// * `present_phase_status` - The present phase status buffer, re-used.
	/// * `phase_fraction` - The string literal 'phaseFraction'.
	/// * `mole` - The string literal 'mole'.
	///
	/// # Returns:
	/// * A `Result` containing the overall enthalpy value, J/mol
	/// 
	fn calculate_overall_enthalpy(
		&mut self,
		material_object : &cape_open_1_2::CapeThermoMaterial,
		material_calculation_routine : &cape_open_1_2::CapeThermoPropertyRoutine,
        present_phases:&mut CapeArrayStringVec,
		present_phase_status:&mut CapeArrayEnumerationVec::<cape_open_1_2::CapePhaseStatus>,
        phase_fraction:&CapeStringImpl,
		mole:&CapeStringImpl,
	) -> Result<f64,COBIAError> {
		//result
        let mut overall_enthalpy = 0.0;
		//string literal enthalpy
		let enthalpy = CapeStringImpl::from("enthalpy");
		//property list {enthalpy} for single phase property calculation
		let mut enthalpy_list = CapeArrayStringVec::new();
        enthalpy_list.resize(1);
        enthalpy_list[0].set(&enthalpy);
		//obtain the present phases
        material_object.get_present_phases(present_phases,present_phase_status)?;
		//iterate over the present phases and calculate the overall enthalpy
		for phase_id in present_phases.iter() {
			//get phase fraction
			let mut phase_fraction_value = CapeArrayRealScalar::new();
			material_object.get_single_phase_prop(phase_fraction, phase_id, mole, &mut phase_fraction_value)?;
			if phase_fraction_value.value()<=0.0 {
				//no need for enthalpy, phase is incipient
				continue;
			}
			//calculate enthalpy
            material_calculation_routine.calc_single_phase_prop(&enthalpy_list,phase_id)?;
			//get enthalpy value
            let mut enthalpy_value=CapeArrayRealScalar::new();
			material_object.get_single_phase_prop(&enthalpy, phase_id, mole, &mut enthalpy_value)?;
			//add to the overall enthalpy
            overall_enthalpy+=enthalpy_value.value()*phase_fraction_value.value();
		}
		Ok(overall_enthalpy)
	}

	/// Warning
	///
	/// This function adds a warning message to the diagnostics interface, if available,
	/// and adds it to the calculation log.
	///
	/// # Arguments:
	/// * `message` - A string slice containing the warning message to be logged.
	/// 
	fn warning(&mut self, message: &str) {
		if let Some(diag) = &self.diagnostics {
			let _ = diag.log_message(&CapeStringImpl::from_string(format!{"warning {}: {}",self.shared_unit_data.borrow().name,message}));
		}
		self.last_run_report.push_str("Warning: ");
        self.last_run_report.push_str(message);
        self.last_run_report.push('\n');
	}

	/// Calculate the unit operation.
	///
	/// Upon calculating, a unit operation must:
	/// * Obtain values from the inlet ports,
	/// * Specify all product ports; Material product ports must be specified with flow rate and a phase equilibrium, which must be requested from the PME,
	/// * Set the values for output parameters.
	///
	/// The PME must ensure that the unit operation is valid before calling this method, but it is goopd practice to check that the unit is indeed validated.
	///
	/// This example is written with simplicity in mind; a production implementation could pre-allocate all array variable objects to avoid re-allocation,
	/// e.g. by making them all class members.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure of the calculation process.

    fn calculate_model(&mut self) -> Result<(),COBIAError> {
        //the PME may only call this method if the unit operation in a valid state.
        // however, we make sure
        if self.shared_unit_data.borrow().validation_status != cape_open_1_2::CapeValidationStatus::CapeValid {
            //validate
            self.validate_internal()?;
        }
        //set up some string constants
        // a production application could cache these strings for efficiency;
        // see the salt-water package for an example on how to do this
        let flow = CapeStringImpl::from("flow");
        let mole = CapeStringImpl::from("mole");
        let no_basis = CapeStringImpl::new();
        let pressure = CapeStringImpl::from("pressure");
        let phase_fraction = CapeStringImpl::from("phaseFraction");
		let fraction = CapeStringImpl::from("fraction"); //mole or mass fraction, aka composition
        //get the required information from the feed port
        let mut feed_rates = CapeArrayRealVec::new();
        //get the material connected to the feed port
        let feed_material = unsafe{MaterialPort::borrow(&self.feed)}.get_connected_material().ok_or_else( || COBIAError::Message("Feed port is not connected".into()))?;
        //get material object interface
        let feed_material_object = cape_open_1_2::CapeThermoMaterial::from_object(&feed_material)?;
        //get component flow rates
        feed_material_object.get_overall_prop(&flow, &mole, &mut feed_rates)?;
        if feed_rates.size() != self.compound_names.size() {
            return Err(COBIAError::Message("Number of compound flows returned by material object does not match number of compounds".into()))
        }
        //get the pressure from the feed material object
        let mut feed_pressure=CapeArrayRealScalar::new(); //Pa
        feed_material_object.get_overall_prop(&pressure, &no_basis, &mut feed_pressure)?;
        //Property calculations at the feed material object are not allowed;
        // we do our calculations directly on the product material objects.
		//Obtain the necessary interfaces to the product streams
        let distillate_material = unsafe {MaterialPort::borrow(&self.distillate_product)}.get_connected_material().ok_or_else( || COBIAError::Message("Distillate port is not connected".into()))?;
        let distillate_material_object = cape_open_1_2::CapeThermoMaterial::from_object(&distillate_material)?;
		let distillate_material_calculation_routine = cape_open_1_2::CapeThermoPropertyRoutine::from_object(&distillate_material)?;
        let distillate_material_equilibrium_routine = cape_open_1_2::CapeThermoEquilibriumRoutine::from_object(&distillate_material)?;
        let bottoms_material = unsafe{MaterialPort::borrow(&self.bottom_product)}.get_connected_material().ok_or_else( || COBIAError::Message("Bottom product port is not connected".into()))?;
        let bottoms_material_object = cape_open_1_2::CapeThermoMaterial::from_object(&bottoms_material)?;
        let bottoms_material_calculation_routine = cape_open_1_2::CapeThermoPropertyRoutine::from_object(&bottoms_material)?;
        let bottoms_material_equilibrium_routine = cape_open_1_2::CapeThermoEquilibriumRoutine::from_object(&bottoms_material)?;
        //products are flashed at pressure and vapor fraction
        let pressure_condition = CapeArrayStringVec::from_slice(& [pressure.as_string(), no_basis.as_string(), "overall".into()]);
        let vapor_fraction_condition = CapeArrayStringVec::from_slice(&[phase_fraction.as_string(), mole.as_string(), self.vapor_phase_id.as_string()]);
        let solution_type = CapeStringImpl::from_string("Unspecified");
        let distillate_vapor_fraction = CapeArrayRealScalar::from(1.0);
        let bottoms_vapor_fraction = CapeArrayRealScalar::from(0.0);
        //for all flashes we allow all phases, and do not specify an initial guess
        let mut phase_status_unspecified = CapeArrayEnumerationVec::<cape_open_1_2::CapePhaseStatus>::new();
        phase_status_unspecified.resize(self.phase_ids.size(), cape_open_1_2::CapePhaseStatus::CapeUnknownphasestatus);
        //allocate some variables to be re-used during calculations
        let mut present_phases = CapeArrayStringVec::new();
        let mut present_phase_status = CapeArrayEnumerationVec::<cape_open_1_2::CapePhaseStatus>::new();
        let mut vapor_composition = CapeArrayRealVec::new(); 
        let mut liquid_composition = CapeArrayRealVec::new();
        let mut distillate_k_values=Vec::with_capacity(self.compound_names.size());
        let mut bottoms_k_values=Vec::with_capacity(self.compound_names.size());
        let mut effective_k_values= Vec::with_capacity(self.compound_names.size());
        let mut alpha = Vec::with_capacity(self.compound_names.size());
		//calculate the enthalpy of the feed, to determine quality
        // note: CAPE-OPEN does not allow calculations on a feed material; we perform the calculation on the distillate material object
        distillate_material_object.copy_from_material(&feed_material_object)?;
        let feed_enthalpy=self.calculate_overall_enthalpy(
            &distillate_material_object,
            &distillate_material_calculation_routine,
            &mut present_phases,
            &mut present_phase_status,
            &phase_fraction,
            &mole,
        ).or_else( |e| {
            Err(COBIAError::Message(format!("Error calculating enthalpy for feed: {}", e)))
        })?;
		//do a dew point calculation at feed composition, to determine K values and overall enthalpy
		// note: CAPE-OPEN does not allow calculations on a feed material; we perform the calculation on the distillate material object
        self.calculate_dew_or_bubble_point(
            &"Feed",
            &distillate_material_object,
            &distillate_material_equilibrium_routine,
            &feed_rates,
            &feed_pressure,
            &distillate_vapor_fraction,
            &pressure,
            &phase_fraction,
            &fraction,
            &flow,
            &mole,
            &no_basis,
            &mut vapor_composition,
            &mut liquid_composition,
            &mut present_phases,
            &mut present_phase_status,
            &pressure_condition,
            &vapor_fraction_condition,
            &solution_type,
			&phase_status_unspecified,
            &mut distillate_k_values,
        )?;
		//calculate enthalpy at saturation point
		let dew_point_feed_enthalpy=self.calculate_overall_enthalpy(
			&distillate_material_object,
			&distillate_material_calculation_routine,
			&mut present_phases,
			&mut present_phase_status,
			&phase_fraction,
			&mole,
		).or_else( |e| {
			Err(COBIAError::Message(format!("Error calculating dew point enthalpy for feed: {}", e)))
		})?;
        //do a bubble point calculation at feed composition, to determine K values and overall enthalpy
        // note: CAPE-OPEN does not allow calculations on a feed material; we perform the calculation on the bottoms material object
        self.calculate_dew_or_bubble_point(
            &"Feed",
            &bottoms_material_object,
            &bottoms_material_equilibrium_routine,
            &feed_rates,
            &feed_pressure,
            &bottoms_vapor_fraction,
            &pressure,
            &phase_fraction,
            &fraction,
            &flow,
            &mole,
            &no_basis,
            &mut vapor_composition,
            &mut liquid_composition,
            &mut present_phases,
            &mut present_phase_status,
            &pressure_condition,
            &vapor_fraction_condition,
            &solution_type,
            &phase_status_unspecified,
            &mut bottoms_k_values,
        )?;
        //calculate enthalpy at saturation point
        let bubble_point_feed_enthalpy=self.calculate_overall_enthalpy(
            &bottoms_material_object,
            &bottoms_material_calculation_routine,
            &mut present_phases,
            &mut present_phase_status,
            &phase_fraction,
            &mole,
        ).or_else( |e| {
            Err(COBIAError::Message(format!("Error calculating dew point enthalpy for feed: {}", e)))
        })?;
		//calculate the quality of the feed
		let feed_quality=(dew_point_feed_enthalpy-feed_enthalpy)/(dew_point_feed_enthalpy-bubble_point_feed_enthalpy);
		//report feed quality
		self.last_run_report.push_str(&format!("Feed quality: {:.4}\n", feed_quality));
		if feed_quality< -f64::EPSILON {
			self.last_run_report.push_str("Feed stream is super-heated");
		} else if feed_quality>1.0+f64::EPSILON {
			//log
			self.last_run_report.push_str("Feed stream is sub-cooled");
		}
		//get effective K values for feed for initial guess
        effective_k_values.clear();
        for (distillate_k, bottoms_k) in distillate_k_values.iter().zip(bottoms_k_values.iter()) {
            effective_k_values.push(f64::sqrt(distillate_k*bottoms_k)); // k_eff = sqrt(k_distillate * k_bottoms)
        }
		//Part 1: Fenske calculation
        // estimate top and bottom flow
        //  for this example we assume heavy and light key split as specified; all other compounds split 50/50
        let mut distillate_rates = feed_rates.clone();
        let mut bottoms_rates = feed_rates.clone(); 
        let light_key_compound_recovery = unsafe{RealParameter::borrow(&self.light_key_compound_recovery).value};
        let rate_light_key_compound_distillate=light_key_compound_recovery * feed_rates[self.light_key_compound_index as usize];
        let rate_light_key_compound_bottoms=(1.0 - light_key_compound_recovery) * feed_rates[self.light_key_compound_index as usize];
        let heavy_key_compound_recovery = unsafe {RealParameter::borrow(&self.heavy_key_compound_recovery).value};
        let rate_heavy_key_compound_bottoms=heavy_key_compound_recovery * feed_rates[self.heavy_key_compound_index as usize];
        let rate_heavy_key_compound_distillate=(1.0 - heavy_key_compound_recovery) * feed_rates[self.heavy_key_compound_index as usize];
		//estimate product rates
		for i in 0..self.compound_names.size() {
			if i==self.light_key_compound_index as usize {
				//light key compound
				distillate_rates[i] = rate_light_key_compound_distillate;
				bottoms_rates[i] = rate_light_key_compound_bottoms;
			} else if i==self.heavy_key_compound_index as usize {
				//heavy key compound
				distillate_rates[i] = rate_heavy_key_compound_distillate;
				bottoms_rates[i] = rate_heavy_key_compound_bottoms;
			} else if effective_k_values[i]>=effective_k_values[self.light_key_compound_index as usize] {
				//light, all to top
                distillate_rates[i] = feed_rates[i];
				bottoms_rates[i] = 0.0;
            } else if effective_k_values[i]<=effective_k_values[self.heavy_key_compound_index as usize] {
				//heavy, all to bottom
				distillate_rates[i] = 0.0;
				bottoms_rates[i] = feed_rates[i];
			} else {
				//all other compounds; 
				let ratio=(effective_k_values[i]-effective_k_values[self.heavy_key_compound_index as usize])/(effective_k_values[self.light_key_compound_index as usize]-effective_k_values[self.heavy_key_compound_index as usize]+f64::EPSILON);
				//split between distillate and bottoms
                distillate_rates[i] = ratio*feed_rates[i];
                bottoms_rates[i] = (1.0-ratio)*feed_rates[i];
			}
		}
        //get the maximum number of iterations
        let maximum_iterations = unsafe{IntegerParameter::borrow(&self.maximum_iterations).value};
        //get the convergence tolerance
        let convergence_tolerance = unsafe{RealParameter::borrow(&self.convergence_tolerance).value};
        //determine the maximum compound flow rate deviation from the total feed flow rate and convergence tolerance
		let total_feed_flow_rate = feed_rates.as_vec().iter().sum::<f64>();
		if total_feed_flow_rate <= 0.0 {
			return Err(COBIAError::Message("Total feed flow rate is zero".into()));
		}
        let max_compound_flow_rate_deviation = total_feed_flow_rate * convergence_tolerance;
		//the numerator in the minimum stages equation is constant
		let numerator_min_stage=f64::ln((rate_light_key_compound_distillate*rate_heavy_key_compound_bottoms)/(rate_light_key_compound_bottoms*rate_heavy_key_compound_distillate));
        //loop over the maximum number of iterations
        let mut min_number_of_stages : f64;
        let mut number_of_iterations = 0;
        loop {
            //increate iteration count
            number_of_iterations += 1;
            if number_of_iterations > maximum_iterations {
                return Err(COBIAError::Message(format!("Maximum number of iterations ({}) exceeded", maximum_iterations)));
            }
			//calculate distillate product 
            self.calculate_dew_or_bubble_point(
                &"Distillate",
                &distillate_material_object,
                &distillate_material_equilibrium_routine,
                &distillate_rates,
                &feed_pressure,
                &distillate_vapor_fraction,
                &pressure,
                &phase_fraction,
                &fraction,
                &flow,
                &mole,
                &no_basis,
                &mut vapor_composition,
                &mut liquid_composition,
                &mut present_phases,
                &mut present_phase_status,
                &pressure_condition,
                &vapor_fraction_condition,
                &solution_type,
                &phase_status_unspecified,
                &mut distillate_k_values,
            )?;
			//calculate bottoms product
            self.calculate_dew_or_bubble_point(
                &"Bottoms",
                &bottoms_material_object,
                &bottoms_material_equilibrium_routine,
                &bottoms_rates,
                &feed_pressure,
                &bottoms_vapor_fraction,
                &pressure,
                &phase_fraction,
                &fraction,
                &flow,
                &mole,
                &no_basis,
                &mut vapor_composition,
                &mut liquid_composition,
                &mut present_phases,
                &mut present_phase_status,
                &pressure_condition,
                &vapor_fraction_condition,
                &solution_type,
                &phase_status_unspecified,
                &mut bottoms_k_values,
            )?;
			//get the effecive k values
			effective_k_values.clear();
			for (distillate_k, bottoms_k) in distillate_k_values.iter().zip(bottoms_k_values.iter()) {
				effective_k_values.push(f64::sqrt(distillate_k*bottoms_k)); // k_eff = sqrt(k_distillate * k_bottoms)
			}
			//get the alpha values
			alpha.clear();
			let k_heavy_component=effective_k_values[self.heavy_key_compound_index as usize];
			for effective_k in effective_k_values.iter() {
				alpha.push(effective_k/k_heavy_component); // alpha_i = k_eff_i / k_eff_heavy
			}
			//calculate the minimum number of theoretical stages
			min_number_of_stages=numerator_min_stage/f64::ln(alpha[self.light_key_compound_index as usize]);
			//while updating the rates, keep track of maximum flow rate error
			let mut max_component_flow_rate_error = 0.0;
            //calculate the updated bottoms rates
            for (i,bottoms_rate) in bottoms_rates.as_mut_vec().iter_mut().enumerate() {
                //calculate the bottoms rate for component i
                let new_bottoms_rate = feed_rates[i] / (1.0+(rate_heavy_key_compound_distillate/rate_heavy_key_compound_bottoms)*f64::powf(alpha[i], min_number_of_stages));
                //update the error
                max_component_flow_rate_error= f64::max(max_component_flow_rate_error, f64::abs(new_bottoms_rate - *bottoms_rate));
                //update the bottoms rate
                *bottoms_rate = new_bottoms_rate;
            }
			//calculate the updated distillate rates
			for (i,distillate_rate) in distillate_rates.as_mut_vec().iter_mut().enumerate() {
				//calculate the distillate rate for component i
				let new_distillate_rate = f64::max(feed_rates[i] - bottoms_rates[i],0.0); //ensure no negative flow rates due to round-off error
				//update the error
                max_component_flow_rate_error= f64::max(max_component_flow_rate_error, f64::abs(new_distillate_rate - *distillate_rate));
				//update the distillate rate
				*distillate_rate = new_distillate_rate;
			}
			//check convergence
			if max_component_flow_rate_error <= max_compound_flow_rate_deviation {
				//converged, break the loop
				break;
			}
        }
		//report the number of iterations
		self.last_run_report.push_str(&format!{"Fenske calculation took {} iterations.\n",number_of_iterations});
		//report the minimum number of stages
        self.last_run_report.push_str(&format!{"Minimum number of stages: {}.\n",min_number_of_stages});
        //Part 2: Underwood calculation
		let mut feed_x_times_alpha= Vec::with_capacity(self.compound_names.size());
		for (i,feed_rate) in feed_rates.as_vec().iter().enumerate() {
			//calculate the feed composition times alpha
			feed_x_times_alpha.push((feed_rate * alpha[i])/total_feed_flow_rate);
		}
		//we look for theta in the range [1,alpha] where 1 is the relative volatility of
		// the heavy key compound, and alpha is the relative volatility next one up;
		// warn if this is not the heavy key compound
		let mut theta_max=alpha[self.light_key_compound_index as usize];
		let mut limiting_compound=-1;
		for i in 0..self.compound_names.size() {
			if i!=self.light_key_compound_index as usize && i!=self.heavy_key_compound_index as usize && alpha[i]>1.0 && alpha[i]<theta_max {
				//found a new next alpha, update it
				theta_max=alpha[i];
				limiting_compound=i as i32;
			}
		}
		if limiting_compound>=0 {
			self.warning(&format!("Underwood calculation uses relative volatility of compound {} as light compound limit",self.compound_names[limiting_compound as usize]));
		}
		let mut theta_min=1.0;
		//solution is bracketed, bisect to find theta
		let mut theta=(theta_max+theta_min)*0.5;
		let mut iteration=0;
		loop {
            //increment iteration count
            iteration += 1;
            if iteration > maximum_iterations {
                return Err(COBIAError::Message(format!("Underwood calculation did not converge within {} iterations",maximum_iterations)));
            }
			//calculate f = sum(feed_x_times_alpha/(alpha-theta))-feed_vapor_fraction
			let mut residual=feed_quality-1.0;
			for i in 0..self.compound_names.size() {
                residual+=feed_x_times_alpha[i]/(alpha[i]-theta);
			}
			if f64::abs(residual) <= convergence_tolerance {
				//converged
				break;
			}
			if residual>0.0 {
				//residual is positive, theta is too high
				theta_max=theta;
				theta=0.5*(theta_min+theta_max);
			} else {
				//residual is negative, theta is too low
				theta_min=theta;
				theta=0.5*(theta_min+theta_max);
			}
			//check convergence on step size
			if theta_max-theta_min <= convergence_tolerance {
				//converged
				break;
			}
		}
		//report convergence
        self.last_run_report.push_str(&format!{"Underwood calculation took {} iterations.\n",number_of_iterations});
		//calculate Rmin from theta
		let mut r_min=-1.0;
		let total_distillate_rate=distillate_rates.as_vec().iter().sum::<f64>();
        for i in 0..self.compound_names.size() {
            r_min+=(alpha[i]*distillate_rates[i])/(total_distillate_rate*(alpha[i]-theta));
        }
		//report minimum reflux ratio
		self.last_run_report.push_str(&format!{"Minimum reflux ratio: {:.4}\n",r_min});
        let r= unsafe{RealParameter::borrow(&self.reflux_ratio_factor).value}*r_min; //actual reflux ratio
		unsafe{RealParameter::borrow_mut(&mut self.reflux_ratio).value=r}; //update the reflux ratio
		//Part 3: Gilliland calculation
		let gilliland_x=(r-r_min)/(r+1.0);
		let gilliland_y=1.0-f64::exp((1.0-54.4*gilliland_x)*(gilliland_x-1.0)/((11.0+117.2*gilliland_x)*f64::sqrt(gilliland_x)));
		let number_of_stages=(gilliland_y+min_number_of_stages)/(1.0-gilliland_y);
        unsafe{RealParameter::borrow_mut(&mut self.number_of_stages).value=number_of_stages}; //update the number of stages
		//Part 4: Kirkbride calculation
        let total_bottoms_rate=bottoms_rates.as_vec().iter().sum::<f64>();
		let ratio=(total_distillate_rate*feed_rates[self.heavy_key_compound_index as usize]*bottoms_rates[self.light_key_compound_index as usize]*bottoms_rates[self.light_key_compound_index as usize])/
                  (total_bottoms_rate*feed_rates[self.light_key_compound_index as usize]*distillate_rates[self.heavy_key_compound_index as usize]*distillate_rates[self.heavy_key_compound_index as usize]);
        let ratio=f64::powf(ratio,0.206);
		let n_feed=number_of_stages/(ratio+1.0);
        unsafe{RealParameter::borrow_mut(&mut self.feed_stage_location).value=n_feed}; //update feed stage location
		//all ok
		Ok(())
    }

}

impl cape_open_1_2::ICapeUnit for DistillationShortcutUnit {

	/// Get the port collection of the unit operation.
	///
	/// # Returns:
	/// * A `Result` containing a `CapeCollection` of `CapeUnitPort` objects representing the ports of the unit operation.

    fn ports(&mut self) -> Result<cape_open_1_2::CapeCollection<cape_open_1_2::CapeUnitPort>,COBIAError> {
        Ok(self.port_collection.clone()) //what is being cloned here is the smart pointer, not its content.
    }

	/// Get the validation status of the unit operation.
	///
	/// # Returns:
	/// * A `Result` containing a `CapeValidationStatus` indicating the validation status of the unit operation.

    fn get_val_status(&mut self) -> Result<cape_open_1_2::CapeValidationStatus,COBIAError> {
		Ok(self.shared_unit_data.borrow().validation_status)
    }

	/// Calculate the unit operation.
	///
	/// The calculate method is called by the PME to perform the calculation of the unit operation.
	///
	/// This is a wrapper around the `calculate_model` method, which is the actual implementation of the calculation logic.
	///
	/// The wrapper is in place just to catch the error generated by the `calculate_model` method and add it the report.

    fn calculate(&mut self) -> Result<(),COBIAError> {
		//report the start of the calculation, log date and time
		let calculation_start_time=chrono::Local::now();
		self.last_run_report= "Calculation started at ".to_string();
        self.last_run_report.push_str(&calculation_start_time.format("%Y-%m-%d %H:%M:%S").to_string());
        self.last_run_report.push('\n');
		match self.calculate_model() {
			Ok(_) => {
				//report the end of the calculation, duration
				let seconds=((chrono::Local::now()-calculation_start_time).num_milliseconds() as f64)*1e-3;
                self.last_run_report.push_str(&format!("Calculation finished in {:.3} seconds.\n",seconds));
				Ok(())
			},
			Err(e) => {
                self.last_run_report.push_str("Calculation failed: ");
                self.last_run_report.push_str(&e.to_string());
				self.last_run_report.push('\n');
				Err(e)
			}
		}
    }

	/// Validation entry point for the PME to validate the unit operation.
	///
	/// The PME must call this method to validate the unit operation before calling calculate.
	///
	/// # Arguments:
	/// * `message` - A mutable reference to a `CapeStringOut` which must be set in case of an invalid state.
	///
	/// # Returns:
	/// * A `Result` containing a `CapeBoolean` indicating whether the unit operation is valid (`true`) or invalid (`false`).
	
    fn validate(&mut self,message:&mut CapeStringOut) -> Result<CapeBoolean,COBIAError> {
		match self.validate_internal() {
			Ok(_) => {
				self.shared_unit_data.borrow_mut().validation_status=cape_open_1_2::CapeValidationStatus::CapeValid;
				Ok(true as CapeBoolean)
			},
			Err(e) => {
				message.set_string(e.to_string())?;
				self.shared_unit_data.borrow_mut().validation_status=cape_open_1_2::CapeValidationStatus::CapeInvalid;
				Ok(false as CapeBoolean)
			}
		}
	}
}

impl cape_open_1_2::ICapeReport for DistillationShortcutUnit {

	/// Get the names of the reports available for this unit operation.
	///
	/// The PME can call this method to obtain the names of the reports that can be generated by this unit operation.
	///
	/// # Arguments:
	/// * `names` - A mutable reference to a `CapeArrayStringOut` where the report names will be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure of the operation.

    fn get_report_names(&mut self,names:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
        names.resize(1)?;
		names.at(0)?.set(&self.last_run_report_name)?;
		Ok(())
    }

	/// Get the types available for a given report.
	///
	/// Report types are returns as MIME types, such as "text/plain" or "application/pdf".
	///
	/// Be aware that compatibility with CAPE-OPEN 1.1 only allows for text typed reports.
	///
	/// # Arguments:
	/// * `name` - A reference to a `CapeStringIn` containing the name of the report for which the types are requested.
	/// * `types` - A mutable reference to a `CapeArrayStringOut` where the report types will be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure of the operation.

    fn report_types(&mut self,name:&CapeStringIn,types:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
		if name.eq_ignore_case(&self.last_run_report_name) {
			types.resize(1)?;
			types.at(0)?.set_string("text/plain")?;
			Ok(())
		} else {
			Err(COBIAError::Code(COBIAERR_INVALIDARGUMENT))
		}
	}

	/// Get the locales available for a given report.
	///
	/// Report locales are used to specify the language and region for the report.
	/// In this example, only the "en" locale is supported.
	///
	/// # Arguments:
	/// * `name` - A reference to a `CapeStringIn` containing the name of the report for which the locales are requested.
	/// * `_type` - A reference to a `CapeStringIn` containing the type of the report (not used in this example).
	/// * `locales` - A mutable reference to a `CapeArrayStringOut` where the report locales will be set.
    ///
	/// # Returns:
	/// * A `Result` indicating success or failure of the operation.

    fn report_locales(&mut self,name:&CapeStringIn,_type:&CapeStringIn,locales:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
		if name.eq_ignore_case(&self.last_run_report_name) {
			locales.resize(1)?;
			locales.at(0)?.set_string("en")?;
			Ok(())
		} else {
			Err(COBIAError::Code(COBIAERR_INVALIDARGUMENT))
		}
    }

	/// Check if a report specification is valid.
	///
	/// This method checks if the report name, type, and locale combination is valid.
	///
	/// # Arguments:
	/// * `name` - A reference to a `CapeStringIn` containing the name of the report to check.
	/// * `_type` - A reference to a `CapeStringIn` containing the type of the report to check.
	/// * `locale` - A reference to a `CapeStringIn` containing the locale of the report to check.
	///
	/// # Returns:
	/// * A `Result` containing a `CapeBoolean` indicating whether the report specification is valid (`true`) or not (`false`).

    fn check_report_spec(&mut self,name:&CapeStringIn,_type:&CapeStringIn,locale:&CapeStringIn) -> Result<CapeBoolean,COBIAError> {
		if name.eq_ignore_case(&self.last_run_report_name) {
			if _type.is_empty() || _type.to_string().to_lowercase()=="text/plain" {
				if locale.is_empty() || locale.to_string().to_lowercase()=="en" {
					return Ok(true as CapeBoolean);
				}				
			}
		}
		Ok(false as CapeBoolean)
    }

	/// Generate a report based on the specified name, type, and locale.
	///
	/// This method generates a report based on the last run report name and returns its content.
	///
	/// # Arguments:
	/// * `name` - A reference to a `CapeStringIn` containing the name of the report to generate.
	/// * `_type` - A reference to a `CapeStringIn` containing the type of the report to generate.
	/// * `locale` - A reference to a `CapeStringIn` containing the locale of the report to generate.
	/// * `report_content` - A mutable reference to a `CapeStringOut` where the report content will be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure of the report generation process.

    fn generate_report(&mut self,name:&CapeStringIn,_type:&CapeStringIn,locale:&CapeStringIn,report_content:&mut CapeStringOut) -> Result<(),COBIAError> {
        if name.eq_ignore_case(&self.last_run_report_name) {
			if _type.is_empty() || _type.to_string().to_lowercase()=="text/plain" {
				if locale.is_empty() || locale.to_string().to_lowercase()=="en" {
					return report_content.set_string(&self.last_run_report);
				} else {
					Err(COBIAError::Message("Invalid/unsupported report locale".into()))
				}
			} else {
				Err(COBIAError::Message("Invalid/unsupported report mime type".into()))
			}
		} else {
			Err(COBIAError::Message("Invalid report name".into()))
		}
	}

	/// Generate a report file based on the specified name, type, locale, and file name.
	///
	/// This method generates a report and writes it to a file. Implemented as a wrapper
	/// around the `generate_report` method.
	///
	/// # Arguments:
	/// * `name` - A reference to a `CapeStringIn` containing the name of the report to generate.
	/// * `_type` - A reference to a `CapeStringIn` containing the type of the report to generate.
	/// * `locale` - A reference to a `CapeStringIn` containing the locale of the report to generate.
	/// * `file_name` - A reference to a `CapeStringIn` containing the name of the file to write the report to.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure of the file generation process.

    fn generate_report_file(&mut self,name:&CapeStringIn,_type:&CapeStringIn,locale:&CapeStringIn,file_name:&CapeStringIn) -> Result<(),COBIAError> {
		let mut report_text=CapeStringImpl::new();
		self.generate_report(name,_type,locale,&mut CapeStringOutFromProvider::from(&mut report_text).as_cape_string_out())?;
		let mut file = match std::fs::File::create(file_name.as_string()) {
			Ok(f) => f,
			Err(e) => return Err(COBIAError::Message(format!("Error creating file: {}",e))),
		};
		match file.write_all(report_text.as_string().as_bytes()) {
			Ok(_) => Ok(()),
			Err(e) => return Err(COBIAError::Message(format!("Error writing to file: {}",e))),
		}
    }

}

impl cape_open_1_2::ICapePersist for DistillationShortcutUnit {

    /// Save the state of the unit operation.
	///
	/// This method is called by the PME to save the state of the unit operation.
	/// It saves all modifiable data to the `CapePersistWriter` and optionally clear the dirty flag.
	///
	/// # Arguments:
	/// * `writer` - A `CapePersistWriter` used to write the state of the unit operation.
	/// * `clear_dirty` - A `CapeBoolean` indicating whether to clear the dirty flag after saving.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure of the save operation.

    fn save(&mut self,writer:cape_open_1_2::CapePersistWriter,clear_dirty:CapeBoolean) -> Result<(),COBIAError> {
		//save the name
		writer.add_string(&CapeStringImpl::from_string("Name"),&self.shared_unit_data.borrow().name)?;
		//save the description
		writer.add_string(&CapeStringImpl::from_string("Description"),&self.description)?;
		//save all parameter values
		{
			let parameter_collection=unsafe{ParameterCollection::borrow_mut(&mut self.parameter_collection)};
			for parameter in parameter_collection.iter_mut() {
				//in this example for simplicity we access the parameters through their CAPE-OPEN interface;
				// this is however not needed, as we can access the parameters directly, which would be more
				// efficient but perhaps harder to read (and involves unsafe operations)
				let mut parameter_name=CapeStringImpl::new();
				let iden=cape_open_1_2::CapeIdentification::from_object(parameter)?;
				iden.get_component_name(&mut parameter_name)?;
				let data_type=parameter.get_type().unwrap();
				match data_type {
					cape_open_1_2::CapeParamType::CapeParameterReal => {
						let real_parameter=cape_open_1_2::CapeRealParameter::from_object(parameter)?;
						let value=real_parameter.get_value()?;
						writer.add_real(&parameter_name,value)?;
					},
                    cape_open_1_2::CapeParamType::CapeParameterInteger => {
                        let integer_parameter=cape_open_1_2::CapeIntegerParameter::from_object(parameter)?;
                        let value=integer_parameter.get_value()?;
                        writer.add_integer(&parameter_name,value)?;
                    },
					cape_open_1_2::CapeParamType::CapeParameterString => {
						let string_parameter=cape_open_1_2::CapeStringParameter::from_object(parameter)?;
						let mut value=CapeStringImpl::new();
						string_parameter.get_value(&mut value)?;
						writer.add_string(&parameter_name,&value)?;
					},
					_  => {
						return Err(COBIAError::Message("Internal error: unexpected data type for parameter".into()));
					}
				}
			}
		}
		//save the last run report content
		writer.add_string(&CapeStringImpl::from_string("LastRunReport"),&CapeStringImpl::from_string(&self.last_run_report))?;
		//clear the dirty flag if requested
		if clear_dirty != 0 {
			self.shared_unit_data.borrow_mut().dirty=false;
		}
		Ok(())
    }

	/// Load the state of the unit operation.
	///
	/// This method is called by the PME to load the state of the unit operation from a `CapePersistReader`.
	/// It loads all modifiable data from the reader and sets the dirty flag to false.
	///
	/// # Arguments:
	/// * `reader` - A `CapePersistReader` used to read the state of the unit operation.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure of the load operation.

    fn load(&mut self,reader:cape_open_1_2::CapePersistReader) -> Result<(),COBIAError> {
		//load the name
		reader.get_string(&CapeStringImpl::from_string("Name"),&mut self.shared_unit_data.borrow_mut().name)?;
		//load the description
		reader.get_string(&CapeStringImpl::from_string("Description"),&mut self.description)?;
		//make a dictionary of string values, case sensitive
		let mut value_names=CapeArrayStringVec::new();
		reader.get_value_names(&mut value_names)?;
		let mut name_set=HashSet::new();
		for value_name in value_names.iter() {
			name_set.insert(value_name.to_string());
		}
		//read parameter values
		// only values that were actually saved are restored, so that new parameter can be added over time
		// which will (if not saved) retain their default values
		{
			let parameter_collection=unsafe{ParameterCollection::borrow_mut(&mut self.parameter_collection)};
			for parameter in parameter_collection.iter_mut() {
				//read the parameter values
				//output parameters cannot be set through the CAPE-OPEN interface; 
				// we access the parameters directly (through their interface pointer) to set the value
				let data_type=parameter.get_type().unwrap();
				match data_type {
					cape_open_1_2::CapeParamType::CapeParameterReal => {
						let real_parameter=unsafe { RealParameter::borrow_mut(parameter) };
						if name_set.contains(&real_parameter.name.to_string()) {
							//parameter not found in the saved values, skip it; it will keep its default value
							real_parameter.value=reader.get_real(&real_parameter.name)?;
						}
					},
                    cape_open_1_2::CapeParamType::CapeParameterInteger => {
                        let integer_parameter=unsafe { IntegerParameter::borrow_mut(parameter) };
                        if name_set.contains(&integer_parameter.name.to_string()) {
                            //parameter not found in the saved values, skip it; it will keep its default value
                            integer_parameter.value=reader.get_integer(&integer_parameter.name)?;
                        }
                    },
					cape_open_1_2::CapeParamType::CapeParameterString => {
						let string_parameter=unsafe { StringParameter::borrow_mut(parameter) };
						if name_set.contains(&string_parameter.name.to_string()) {
							reader.get_string(&string_parameter.name,&mut string_parameter.value)?;
						}
					},
					_  => {
						return Err(COBIAError::Message("Internal error: unexpected data type for parameter".into()));
					}
				}
			}
		}
		//load the last run report content
		let mut last_run_report=CapeStringImpl::new();
		reader.get_string(&CapeStringImpl::from_string("LastRunReport"),&mut last_run_report)?;
		self.last_run_report=last_run_report.as_string();
		//clear the dirty flag if requested
		self.shared_unit_data.borrow_mut().dirty=false;
		//ok
		Ok(())
    }

	/// Get the dirty flag of the unit operation.
	///
	/// The dirty flag indicates whether the unit operation has unsaved changes.
	///
	/// # Returns:
	/// * A `Result` containing a `CapeBoolean` indicating whether the unit operation is dirty (`true`) or not (`false`).

    fn get_is_dirty(&mut self) -> Result<CapeBoolean,COBIAError> {
        Ok(self.shared_unit_data.borrow().dirty as CapeBoolean)
    }
}