use crate::C;
use crate::*;
use cape_smart_pointer::CapeSmartPointer;

const ICAPEPMCREGISTRATIONDETAILS_UUID:CapeUUID=CapeUUID::from_slice(&[0x0eu8,0x6eu8,0x73u8,0xefu8,0xdcu8,0x77u8,0x4au8,0xcbu8,0x90u8,0x74u8,0xbbu8,0xfcu8,0x8du8,0xcbu8,0x05u8,0xe2u8]);

/// PMC registration details
///
/// The PMC registration details are for a PMC reveal the registration
/// of the PMC in the registry.
/// 
/// The PMC registration details can be used to create a PMC.
/// 
/// Selection of a PMC is typically done by the user, and the PME 
/// offers a list of PMCs to choose from by enumerating the PMCs
/// of a particular kind.
///
/// #Example
///
/// ```
/// use cobia;
/// use cobia::prelude::*;
/// cobia::cape_open_initialize().unwrap();
/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
///		println!("PMC: {}",pmc.get_name().unwrap());
/// }
/// cobia::cape_open_cleanup();
/// ```

#[cape_smart_pointer(ICAPEPMCREGISTRATIONDETAILS_UUID)]
pub struct CapePMCRegistrationDetails {
	pub(crate) interface: *mut C::ICapePMCRegistrationDetails,
}

impl CapePMCRegistrationDetails {

	/// Create a new CapePMCRegistrationDetails from an interface pointer
	///
	/// This member is not typically called. Instead, the CapePMCRegistrationDetails is created by the API functions that return it.
	///
	/// # Safety
	///
	/// The interface pointer must be valid and must point to an object that implements the ICapePMCRegistrationDetails interface.
	///
	/// # Panics
	///
	/// This function panics if the interface pointer is null.

	pub(crate) fn from_interface_pointer(interface: *mut C::ICapePMCRegistrationDetails) ->  Self {
		if interface.is_null() {
			panic!("Null pointer in creation of CapePMCRegistrationDetails");
		}
		unsafe {((*(*interface).vTbl).base.addReference.unwrap())((*interface).me)};
		Self {
			interface
		}
	}

	/// Create a new CapePMCRegistrationDetails from an interface pointer without adding a reference
	///
	/// This member is not typically called. Instead, the CapePMCRegistrationDetails is created by the API functions that return it.
	///
	/// # Safety
	///
	/// The interface pointer must be valid and must point to an object that implements the ICapePMCRegistrationDetails interface.
	///
	/// # Panics
	///
	/// This function panics if the interface pointer is null.

	pub(crate) fn attach(interface: *mut C::ICapePMCRegistrationDetails) ->  Self {
		if interface.is_null() {
			panic!("Null pointer in creation of CapePMCRegistrationDetails");
		}
		Self {
			interface
		}
	}

	/// Get the name of the PMC
	///
	/// The name of the PMC as it is appears in the registry.
	///
	/// # Errors
	///
	/// Returns an error if the name cannot be retrieved.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///		println!("PMC name: {}",pmc.get_name().unwrap());
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_name(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getName.unwrap())((*self.interface).me, (&s.as_cape_string_out() as *const C::ICapeString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get the description of the PMC
	///
	/// The description of the PMC as it is appears in the registry.
	///
	/// # Errors
	///
	/// Returns an error if the description cannot be retrieved.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///     match pmc.get_description() {
	///         Ok(description) => println!("PMC {} has description: {}",pmc.get_name().unwrap(),description),
	///         Err(err) => println!("PMC {} has no description",pmc.get_name().unwrap())
	///		}
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_description(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getDescription.unwrap())(
				(*self.interface).me,
				(&s.as_cape_string_out() as *const C::ICapeString).cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get the CAPE-OPEN version of the PMC
	///
	/// This function is deprecated and should not be used.
	///
	/// Instead one should inspect which CAPE-OPEN versions are supported
	/// through category IDs.

	pub fn get_cape_version(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getCapeVersion.unwrap())(
				(*self.interface).me,
				(&s.as_cape_string_out() as *const C::ICapeString).cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get the version of the PMC
	///
	/// The version of the PMC as it is appears in the registry.
	///
	/// # Errors
	///
	/// Returns an error if the version cannot be retrieved.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///     match pmc.get_component_version() {
	///         Ok(version) => println!("PMC {} version: {}",pmc.get_name().unwrap(),version),
	///         Err(err) => println!("PMC {} has no version",pmc.get_name().unwrap())
	///		}
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_component_version(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getComponentVersion.unwrap())(
				(*self.interface).me,
				(&s.as_cape_string_out() as *const C::ICapeString).cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get the vendor url of the PMC
	///
	/// The vendor url of the PMC as it is appears in the registry.
	///
	/// # Errors
	///
	/// Returns an error if the vendor url cannot be retrieved.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///     match pmc.get_vendor_url() {
	///         Ok(vendor_url) => println!("PMC {} vendor_url: {}",pmc.get_name().unwrap(),vendor_url),
	///         Err(err) => println!("PMC {} has no vendor_url",pmc.get_name().unwrap())
	///		}
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_vendor_url(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getVendorURL.unwrap())(
				(*self.interface).me,
				(&s.as_cape_string_out() as *const C::ICapeString).cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get the help url of the PMC
	///
	/// The help url of the PMC as it is appears in the registry.
	///
	/// # Errors
	///
	/// Returns an error if the help url cannot be retrieved.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///     match pmc.get_help_url() {
	///         Ok(help_url) => println!("PMC {} help_url: {}",pmc.get_name().unwrap(),help_url),
	///         Err(err) => println!("PMC {} has no help_url",pmc.get_name().unwrap())
	///		}
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_help_url(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getHelpURL.unwrap())(
				(*self.interface).me,
				(&s.as_cape_string_out() as *const C::ICapeString).cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get the about text of the PMC
	///
	/// The about text of the PMC as it is appears in the registry.
	/// This is a descriptive text about the PMC.
	///
	/// # Errors
	///
	/// Returns an error if the about text cannot be retrieved.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///     match pmc.get_about() {
	///         Ok(about) => println!("About {}: {}",pmc.get_name().unwrap(),about),
	///         Err(err) => println!("Nothing about PMC {}...",pmc.get_name().unwrap())
	///		}
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_about(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getAbout.unwrap())((*self.interface).me, (&s.as_cape_string_out() as *const C::ICapeString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get the id of the PMC
	///
	/// The id of the PMC. This is the main unique ID
	/// that identifies the PMC. It is a required attribute.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///		println!("uuid = {}, pmc = {}",pmc.get_uuid().unwrap(),pmc.get_name().unwrap());
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_uuid(&self) -> Result<CapeUUID, COBIAError> {
		let mut uuid = CapeUUID::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getUUID.unwrap())(
				(*self.interface).me,
				&mut uuid as *mut C::CapeUUID,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(uuid)
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get the programmatic id of the PMC
	///
	/// The help programmatic id of the PMC as it is appears in the registry.
	/// This is an optional, alternative ID to the UUID
	///
	/// # Errors
	///
	/// Returns an error if the programmatic id text cannot be retrieved.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///     match pmc.get_prog_id() {
	///         Ok(progid) => println!("PMC {} programmatic id: {}",pmc.get_name().unwrap(),progid),
	///         Err(err) => println!("PMC {} has no programmatic id...",pmc.get_name().unwrap())
	///		}
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_prog_id(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getProgId.unwrap())((*self.interface).me, (&s.as_cape_string_out() as *const C::ICapeString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get the version independent programmatic id of the PMC
	///
	/// The help independent programmatic id of the PMC as it is appears in the registry.
	/// This is an optional, alternative ID to the UUID that does not change between 
	/// software releases.
	///
	/// # Errors
	///
	/// Returns an error if the independent programmatic id text cannot be retrieved.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///     match pmc.get_version_independent_prog_id() {
	///         Ok(progid) => println!("PMC {} version indepndent programmatic id: {}",pmc.get_name().unwrap(),progid),
	///         Err(err) => println!("PMC {} has no version indepndent programmatic id...",pmc.get_name().unwrap())
	///		}
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_version_independent_prog_id(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl)
				.getVersionIndependentProgId
				.unwrap())((*self.interface).me, (&s.as_cape_string_out() as *const C::ICapeString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get all category IDs implemented by the PMC
	///
	/// This allows for checking what kind of PMC it is, and which additional properties
	/// it has (including which CAPE-OPEN versions it supports). 
	///
	/// Programmatic IDs are typically defined in type librararies. All major PMC categories, 
	/// such as Unit Operation, Property Package Manager, etc, are defined in the cape_open
	/// type library. 
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///     let cat_ids = pmc.get_cat_ids().unwrap();
	///     print!("PMC {} implements: ",pmc.get_name().unwrap());
	/// 	let mut first=true;
	/// 	for cat_id in cat_ids {
	/// 		if first {
	/// 			first=false;
	/// 		} else {
	/// 			print!(", ");
	/// 		}
	///         print!("{}",cat_id);
	///         ///print the name so that we can see what it means
	/// 		let cat_key=cobia::CapeRegistryKey::from_path(&format!("/types/categories/{}",cat_id));
	/// 		if let Ok(cat_key) = cat_key {
	/// 			let cat_name=cat_key.get_string_value("name",None);
	/// 			if let Ok(cat_name) = cat_name {
	/// 				print!("={}",cat_name);
	/// 			}
	/// 		}
	///     }
	/// 	println!("");
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_cat_ids(&self) -> Result<Vec<CapeUUID>, COBIAError> {
		let mut a = CapeArrayStringVec::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getCatIDs.unwrap())(
				(*self.interface).me,
				(&a.as_cape_array_string_out() as *const C::ICapeArrayString).cast_mut()
			)
		};
		if result == COBIAERR_NOERROR {
			let mut catids = Vec::<CapeUUID>::new();
			let catids_as_strings = a.as_string_vec();
			catids.reserve(catids_as_strings.len());
			for catid_str in catids_as_strings {
				match CapeUUID::from_string(&catid_str) {
					Ok(cat_id) => catids.push(cat_id),
					Err(err) => {
						return Err(err);
					}
				}
			}
			Ok(catids)
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Check if the PMC implements a specific category ID
	///
	/// This allows for checking what kind of PMC it is, and which additional properties
	/// it has (including which CAPE-OPEN versions it supports).
	///
	/// Programmatic IDs are typically defined in type librararies. All major PMC categories,
	/// such as Unit Operation, Property Package Manager, etc, are defined in the cape_open
	/// type library.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// use cobia::cape_open;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// //let's create some thermo! Enumerate all Property Package Managers and stand-alone Property Packages
	/// let pmc_types=[cape_open::CATEGORYID_PROPERTYPACKAGEMANAGER,cape_open::CATEGORYID_STANDALONEPROPERTYPACKAGE];
	/// let pmcs = pmc_enumerator.pmcs(&pmc_types).unwrap();
	/// for pmc in pmcs {
	///     println!("Found {}: {} ({})",
	///         if pmc.implements_cat_id(&cape_open::CATEGORYID_PROPERTYPACKAGEMANAGER).unwrap() {"Property Package Manager"} else {"Stand-alone Property Package"},
	/// 		pmc.get_name().unwrap(),
	/// 		pmc.get_description().unwrap());
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn implements_cat_id(&self, cat_id: &CapeUUID) -> Result<bool, COBIAError> {
		let mut boolval = 0u32;
		let result = unsafe {
			((*(*self.interface).vTbl).implementsCatID.unwrap())(
				(*self.interface).me,
				cat_id as *const C::CapeUUID,
				&mut boolval as *mut u32,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(boolval != 0)
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get the supported service types for the PMC
	///
	/// The service types are the types of services that the PMC can provide.
	/// 
	/// Typically used by the PMC to find the most suitable service type in
	/// the current context.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///     let service_types = pmc.get_service_types().unwrap();
	///     println!("PMC {} supports the following service types:",pmc.get_name().unwrap());
	///     for service_type in service_types {
	///         println!("\t{}",service_type);
	///     }
	///		println!("");
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_service_types(&self) -> Result<Vec<CapePMCServiceType>, COBIAError> {
		let mut a:CapeArrayEnumerationVec<CapePMCServiceType> = CapeArrayEnumerationVec::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getServiceTypes.unwrap())(
				(*self.interface).me,
				(&a.as_cape_array_enumeration_out() as *const C::ICapeArrayEnumeration).cast_mut()
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(a.as_vec().clone())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get the location of the PMC for the specified service type
	/// 
	/// The meaning of the location depends on the service type. For 
	/// an in-process services it is a path to a shared library. For 
	/// a COM service, it is the COM class ID.
	///
	/// The location for the best suitable service type typically is 
	/// obtained by COBIA to create the PMC.
	///
	/// # Errors
	///
	/// Returns an error if the location cannot be retrieved; this
	/// probably means a service is not registered for the requested 
	/// type
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// let pmcs = pmc_enumerator.all_pmcs().unwrap();
	/// for pmc in pmcs {
	///     println!("{}:",pmc.get_name().unwrap());
	/// 	for service_type in cobia::CapePMCServiceType::iter() {
	/// 		match pmc.get_location(service_type) {
	/// 			Ok(loc) => {
	/// 				println!("\t{}: {}",service_type,loc);
	/// 			},
	/// 			Err(_) => {}
	/// 		}
	/// 	} 
	/// 	println!("");
	/// }
	/// cobia::cape_open_cleanup();
	/// ```
	pub fn get_location(&self, service_type: CapePMCServiceType) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getLocation.unwrap())(
				(*self.interface).me,
				service_type as i32,
				(&s.as_cape_string_out() as *const C::ICapeString).cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Check if the PMC is registered for all users for the specified service type
	///
	/// This function checks if the PMC is registered for all users for the specified service type.
	/// This can be used to check whether the registration information is obtained from the 
	/// current user registry hive or the all users registry hive. For any PMC that is registered for
	/// both all users and the current user, the current user registration overrides the all users
	/// registration.
	///
	/// Note that this property is not available for COM service types, that are taken from the 
	/// Windows registry.
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// let pmcs = pmc_enumerator.all_pmcs().unwrap();
	/// for pmc in pmcs {
	///     println!("{}:",pmc.get_name().unwrap());
	/// 	for service_type in cobia::CapePMCServiceType::iter() {
	/// 		match pmc.registered_for_all_users(service_type) {
	/// 			Ok(is_all_users) => {
	/// 				println!("\t{}: {}",service_type,if is_all_users {"ALL USERS"} else {"CURRENT USER"});
	/// 			},
	/// 			Err(_) => {}
	/// 		}
	/// 	} 
	/// 	println!("");
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn registered_for_all_users(
		&self,
		service_type: CapePMCServiceType,
	) -> Result<bool, COBIAError> {
		let mut boolval = 0u32;
		let result = unsafe {
			((*(*self.interface).vTbl).registeredForAllUsers.unwrap())(
				(*self.interface).me,
				service_type as i32,
				&mut boolval as *mut u32,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(boolval != 0)
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get the registration flags of the PMC registration
	///
	/// The registration flags provide information about the requirements
	/// and capabilities of the PMC, for example details about its threading
	/// model.


	pub fn get_flags(&self) -> Result<CapePMCRegistrationFlags, COBIAError> {
		let mut flags: C::CapePMCRegistrationFlags = 0;
		let result = unsafe {
			((*(*self.interface).vTbl).getFlags.unwrap())(
				(*self.interface).me,
				&mut flags as *mut C::CapePMCRegistrationFlags,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapePMCRegistrationFlags::from_bits_truncate(flags as i32))
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Create an instance of the PMC
	///
	/// This function creates an instance of the PMC. The PMC is created
	/// with the specified creation flags. The flags can be used to specify 
	/// creation options, such as the threading model requirements.
	///
	/// # Parameters
	///
	/// * `flags` - The PMC creation flags to use for creating the PMC.
	///
	/// # Examples
	///
	/// ```no_run
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// let ppm_info_result=pmc_enumerator.get_pmc_by_prog_id("COCO_TEA.PropertyPackManager");
	/// match ppm_info_result {
	/// 	Ok(ppm_info) => {
	/// 		let ppm_result=ppm_info.create_instance(cobia::CapePMCCreationFlags::AllowRestrictedThreading);	//we only use the object in this thread
	/// 		match ppm_result {
	/// 			Ok(ppm) => {
	/// 				//show name
	/// 				let iden_result=cobia::cape_open_1_2::CapeIdentification::from_object(&ppm);
	/// 				match iden_result {
	/// 					Ok(iden) => {
	///							let mut name = cobia::CapeStringImpl::new();
	/// 						iden.get_component_name(&mut name).unwrap();
	/// 						println!("PPM name: {}",name);
	/// 					}
	/// 					Err(_) => {
	/// 						eprintln!("Object does not implement CAPEOPEN_1_2::ICapeIdentification");
	/// 					}
	/// 				}
	/// 			}
	/// 			Err(err) => {
	/// 				eprintln!("Cannot find TEA property package manager (not installed?): {err}");
	/// 			}
	/// 		};
	/// 	}
	/// 	Err(err) => {
	/// 		eprintln!("Cannot find TEA property package manager (not installed?): {err}");
	/// 	}
	/// };
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn create_instance(&self, flags: CapePMCCreationFlags) -> Result<CapeObject, COBIAError> {
		let mut instance: *mut C::ICapeInterface = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).createInstance.unwrap())(
				(*self.interface).me,
				flags.bits() as i32,
				&mut instance as *mut *mut C::ICapeInterface,
			)
		};
		if result == COBIAERR_NOERROR {
			if instance.is_null() {
				Err(COBIAError::Code(COBIAERR_NULLPOINTER))
			} else {
				Ok(CapeObject::attach(instance))
			}			
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}


}


