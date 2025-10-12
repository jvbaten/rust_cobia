use crate::C;
use crate::*;
use cape_smart_pointer::CapeSmartPointer;

const ICAPELIBRARYDETAILS_UUID:CapeUUID=CapeUUID::from_slice(&[0x8du8,0xe5u8,0xeau8,0x0eu8,0xe8u8,0x0au8,0x4au8,0xceu8,0xbeu8,0xfeu8,0x1fu8,0x7fu8,0x72u8,0xaau8,0xf4u8,0x84u8]);

/// CapeLibraryDetails provides details about a registered COBIA type library
///
/// # Examples
///
/// ```
/// use cobia;
/// use cobia::prelude::*;
/// use cobia::cape_open_1_2;
/// cobia::cape_open_initialize().unwrap();
/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
/// let library_details = library_enumerator.get_library_by_library_id(&cape_open_1_2::LIBRARY_ID).unwrap();
/// assert_eq!(library_details.get_name().unwrap(),"CAPEOPEN_1_2");
/// cobia::cape_open_cleanup();
/// ```

#[cape_smart_pointer(ICAPELIBRARYDETAILS_UUID)]
pub struct CapeLibraryDetails {
	pub(crate) interface: *mut C::ICapeLibraryDetails,
}

impl CapeLibraryDetails {

	/// Create a new CapeLibraryDetails from an interface pointer
	///
	/// This member is not typically called. Instead, the CapeLibraryDetails is created by the API functions that return it.
	///
	/// # Safety
	///
	/// The interface pointer must be valid and must point to an object that implements the ICapeLibraryDetails interface.
	///
	/// # Panics
	///
	/// This function panics if the interface pointer is null.

	pub(crate) fn from_interface_pointer(interface: *mut C::ICapeLibraryDetails) ->  Self {
		if interface.is_null() {
			panic!("Null pointer in creation of CapeLibraryDetails");
		}
		unsafe {((*(*interface).vTbl).base.addReference.unwrap())((*interface).me)};
		Self {
			interface
		}
	}

	/// Create a new CapeLibraryDetails from an interface pointer without adding a reference
	///
	/// This member is not typically called. Instead, the CapeLibraryDetails is created by the API functions that return it.
	///
	/// # Safety
	///
	/// The interface pointer must be valid and must point to an object that implements the ICapeLibraryDetails interface.
	///
	/// # Panics
	///
	/// This function panics if the interface pointer is null.

	pub(crate) fn attach(interface: *mut C::ICapeLibraryDetails) ->  Self {
		if interface.is_null() {
			panic!("Null pointer in creation of CapeLibraryDetails");
		}
		Self {
			interface
		}
	}

	/// Get the name of the library
	///
	/// The name of the library as it is appears in the registry.
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
	/// use cobia::cape_open_1_2;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let library_details = library_enumerator.get_library_by_library_id(&cape_open_1_2::LIBRARY_ID).unwrap();
	/// assert_eq!(library_details.get_name().unwrap(),"CAPEOPEN_1_2");
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

	/// Get the id of the library
	///
	/// Get the id of the library as CapeUUID.
	///
	/// # Errors
	///
	/// Returns an error if the id cannot be retrieved.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// use cobia::cape_open_1_2;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let library_details = library_enumerator.get_library_by_library_id(&cape_open_1_2::LIBRARY_ID).unwrap();
	/// assert_eq!(&library_details.get_uuid().unwrap(),&cape_open_1_2::LIBRARY_ID);
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
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get the version of the library
	///
	/// Get the version of the library as string.
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
	/// use cobia::cape_open_1_2;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let library_details = library_enumerator.get_library_by_library_id(&cape_open_1_2::LIBRARY_ID).unwrap();
	/// assert!(library_details.get_library_version().unwrap().starts_with("1.2")); //there are 3 numbers to a CAPE-OPEN type library version; we get the newest installed 1.2
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_library_version(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getLibraryVersion.unwrap())(
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

	/// Get the file name of the CIDL file
	///
	/// Get the full path to the CAPE-OPEN Interface Definition Language (CIDL) file that 
	/// contains the type definitions of all types in the library.
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
	/// use cobia::cape_open_1_2;
	/// use std::path::Path;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let library_details = library_enumerator.get_library_by_library_id(&cape_open_1_2::LIBRARY_ID).unwrap();
	/// assert!(library_details.get_library_path().unwrap().exists()); 
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_library_path(&self) -> Result<PathBuf, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getLibraryPath.unwrap())(
				(*self.interface).me,
				(&s.as_cape_string_out() as *const C::ICapeString).cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(PathBuf::from(s.as_string()))
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get proxy provider file name
	///
	/// Get the full path of the dynamic libary that contains the proxy interface provider for a given service,
	/// if a proxy provider for that service is indeed registered for the requested type library.
	///
	/// # Arguments
	/// 
	/// * `service` - The service for which the proxy provider is requested.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// use cobia::cape_open_1_2;
	/// use std::path::Path;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let library_details = library_enumerator.get_library_by_library_id(&cape_open_1_2::LIBRARY_ID).unwrap();
	/// match library_details.get_proxy_interface_provider_location(cobia::CapePMCServiceType::Inproc64) {
	///    Some(path) => {
	///        assert!(path.exists());
	///	   },
	///    None => {
	///        // no proxy provider for this service
	///    }
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_proxy_interface_provider_location(
		&self,
		service: CapePMCServiceType,
	) -> Option<PathBuf> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl)
				.getProxyInterfaceProviderLocation
				.unwrap())(
				(*self.interface).me,
				service as C::CapePMCServiceType,
				(&s.as_cape_string_out() as *const C::ICapeString).cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Some(PathBuf::from(s.as_string()))
		} else {
			None
		}
	}

}

