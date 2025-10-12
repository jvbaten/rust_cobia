use crate::*;

/// PMCInfo is a struct that contains the information required to register a PMC.
///
/// An array of PMCInfo objects is defined by a module that implements PMCs
///  that must be registered.
///
/// The above example expects my_pmc_class_1 and my_pmc_class_2
/// to be in scope and implement the PMCRegisterationInfo traits.
///
/// the PMCS array is then passed to the pmc_entry_points! macro,
/// a boolean to determine whether the PMCs are registered for all
/// users or just the current user.
///
/// The pmc_entry_points! macro generates the required entry points
/// to be used by the COBIA registration tool
///
/// # Example
///
/// See the example in the lib.rs file in the pmc module

pub struct PMCInfo {
	pub create_instance: fn(*mut *mut C::ICapeInterface) -> CapeResult,
	pub registration_details: fn(&CapeRegistrar) -> Result<(), COBIAError>,
	pub get_uuid: fn() -> CapeUUID,
}

/// PMCRegisterationInfo is a trait that must be implemented by a struct that
/// implements a PMC. The trait provides the information required to register
/// the PMC with the COBIA registry.
///
/// # Example
///
/// See the example in the unit_operation.rs file in the pmc module

pub trait PMCRegisterationInfo {
	fn registration_details(registrar: &CapeRegistrar) -> Result<(), COBIAError>;
	fn get_uuid() -> CapeUUID;
}

/// CapeCreateInstance is creates an instance of a PMC object. This trait is
/// typically implemented by the cape_object_implementation! macro
/// and only for objects created with the cape_object_implementation! macro,
/// which do not take any arguments for construction.

pub trait CapeCreateInstance {
	fn create_instance(instance: *mut *mut C::ICapeInterface) -> CapeResult;
}

/// class factory for a pmc_info object from a CAPE-OPEN object class
///
/// # Example:
///
/// See PMCInfo documentation

pub const fn pmc_info<T: PMCRegisterationInfo + CapeCreateInstance>() -> PMCInfo {
	PMCInfo {
		create_instance: T::create_instance,
		registration_details: T::registration_details,
		get_uuid: T::get_uuid,
	}
}

/// register a PMC into the COBIA registry
///
/// This entry point is called by the the capeRegisterObjects entry point
/// that is generated from the pmc_entry_points! macro to register a PMC

pub fn register_pmc(pmc_info: &PMCInfo, writer: &CapeRegistryWriter) -> Result<(), COBIAError> {
	let registrar = writer.get_pmc_registrar()?;
	(pmc_info.registration_details)(&registrar)?;
	let p = match process_path::get_dylib_path() {
		Some(p) => p,
		None => return Err(COBIAError::Code(COBIAERR_UNKNOWNERROR)),
	};
	let p = match p.to_str() {
		Some(p) => p,
		None => return Err(COBIAError::Code(COBIAERR_UNKNOWNERROR)),
	};
	registrar.add_location(inproc_service_type(), p)?;
	registrar.commit()?;
	Ok(())
}

/// unregister a PMC from the COBIA registry
///
/// This entry point is called by the the capeUnregisterObjects entry point
/// that is generated from the pmc_entry_points! macro to register a PMC

pub fn unregister_pmc(pmc_info: &PMCInfo, writer: &CapeRegistryWriter) -> Result<(), COBIAError> {
	writer.unregister_pmc_service(&(pmc_info.get_uuid)(), inproc_service_type())?;
	Ok(())
}

/// Generate PMC entry points
///
/// This macro creates the PMC module entry points for PMC registration,
/// unregistration and object creation.
///
/// The macro takes two arguments:
///
/// 1. A slice of PMCInfo objects that contain the information required to register
///    the PMCs
/// 2. A boolean that determines whether the PMCs are registered for all users or just the current user
///
/// # Example
///
/// See the example in the lib.rs file in the pmc module

#[macro_export]
macro_rules! pmc_entry_points {
	( $pmc_defs:expr, $register_for_all_users:expr ) => {

		/// Type registration entry point for the module.
		///
		/// This function is called by the COBIA registration tool to register the PMCs
		/// defined in the module.
		///
		/// # Returns
		/// * A `Result` indicating success or failure of the registration.

		fn register_types() -> Result<(), cobia::COBIAError> {
			//register the cobia pmc
			cobia::cape_open_initialize()?;
			let writer = cobia::CapeRegistryWriter::new($register_for_all_users)?;
			match ($pmc_defs)
				.into_iter()
				.try_for_each(|pmc| -> Result<(), cobia::COBIAError> {
					cobia::register_pmc(&pmc, &writer)?;
					Ok(())
				}) {
				Ok(_) => {
					writer.commit()?;
					Ok(())
				}
				Err(e) => {
					return Err(e);
				}
			}
		}

		///COBIA PMC registration entry point
		///
		/// This function is called by the COBIA registration tool to register the PMCs
		/// defined in the module.
		///
		/// # Returns
		/// * A `CapeResult` indicating success or failure of the registration.

		#[unsafe(no_mangle)]
		pub extern "C" fn capeRegisterObjects() -> cobia::CapeResult {
			match register_types() {
				Ok(_) => cobia::COBIAERR_NOERROR,
				Err(e) => e.as_code(),
			}
		}

		/// Type unregistration entry point for the module.
		///
		/// This function is called by the COBIA registration tool to unregister the PMCs
		/// defined in the module.
		///
		/// # Returns
		/// * A `Result` indicating success or failure of the unregistration.

		fn unregister_types() -> Result<(), cobia::COBIAError> {
			//unregister the cobia pmc
			cobia::cape_open_initialize()?;
			let writer = cobia::CapeRegistryWriter::new($register_for_all_users)?;
			match ($pmc_defs)
				.into_iter()
				.try_for_each(|pmc| -> Result<(), cobia::COBIAError> {
					cobia::unregister_pmc(&pmc, &writer)?;
					Ok(())
				}) {
				Ok(_) => {
					writer.commit()?;
					Ok(())
				}
				Err(e) => {
					return Err(e);
				}
			}
		}

		/// COBIA PMC unregistration entry point
		///
		/// This function is called by the COBIA registration tool to unregister the PMCs
		/// defined in the module.
		///
		/// # Returns
		/// * A `CapeResult` indicating success or failure of the unregistration.

		#[unsafe(no_mangle)]
		pub extern "C" fn capeUnregisterObjects() -> cobia::CapeResult {
			match unregister_types() {
				Ok(_) => cobia::COBIAERR_NOERROR,
				Err(e) => e.as_code(),
			}
		}

		/// COBIA PMC object creation entry point
		///
		/// This function is called by the COBIA runtime to create an instance of a PMC object.
		///
		/// # Arguments
		/// * `uuid` - A pointer to the UUID of the PMC object to create.
		/// * `ptr` - A mutable pointer to a pointer where the created object will be stored.
		///
		/// # Returns
		/// * A `CapeResult` indicating success or failure of the object creation.

		#[unsafe(no_mangle)]
		pub extern "C" fn capeCreateObject(
			uuid: *const cobia::CapeUUID,
			ptr: *mut *mut cobia::C::ICapeInterface,
		) -> cobia::CapeResult {
			let u = unsafe { *uuid };
			match ($pmc_defs).into_iter().find(|pmc| (pmc.get_uuid)() == u) {
				Some(pmc) => {
					let res = (pmc.create_instance)(ptr);
					if res == cobia::COBIAERR_NOERROR {
						unsafe {
							let p = *ptr as *mut cobia::C::ICapeInterface;
							(*(*p).vTbl).addReference.unwrap()((*p).me);
						}
					}
					res
				}
				None => cobia::COBIAERR_NOSUCHITEM,
			}
		}
	};
}
