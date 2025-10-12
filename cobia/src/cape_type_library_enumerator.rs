use crate::C;
use crate::*;

/// Enumerator for all registered type libraries.
///
/// This is used to get the details of each library by name or UUID,
/// or to get all libraries.

pub struct CapeTypeLibraries {
	pub(crate) interface: *mut C::ICapeLibraryEnumerator,
}

impl CapeTypeLibraries {

	/// Create a new library enumerator.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// //...
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn new() -> Result<CapeTypeLibraries, COBIAError> {
		let mut key: *mut C::ICapeLibraryEnumerator = std::ptr::null_mut();
		let result =
			unsafe { C::capeGetLibraryEnumerator(&mut key as *mut *mut C::ICapeLibraryEnumerator) };
		if result == COBIAERR_NOERROR {
			Ok(CapeTypeLibraries { interface: key })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get the library details by library UUID.
	///
	/// Get the details of a library by its UUID.
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

	pub fn get_library_by_library_id(&self, library_id: &CapeUUID) -> Result<CapeLibraryDetails, COBIAError> {
		let mut key: *mut C::ICapeLibraryDetails = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).getLibraryByUUID.unwrap())(
				(*self.interface).me,
				(library_id as *const C::CapeUUID).cast_mut(),
				&mut key as *mut *mut C::ICapeLibraryDetails,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapeLibraryDetails { interface: key })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get the library details by library name.
	///
	/// Get the details of a library by its name.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// use cobia::cape_open_1_2;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let library_details = library_enumerator.get_library_by_name("CAPEOPEN_1_2").unwrap();
	/// assert_eq!(library_details.get_name().unwrap(),"CAPEOPEN_1_2");
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_library_by_name(&self, lib_name: &str) -> Result<CapeLibraryDetails, COBIAError> {
		let mut key: *mut C::ICapeLibraryDetails = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).getLibraryByName.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(lib_name).as_capechar_const(),
				&mut key as *mut *mut C::ICapeLibraryDetails,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapeLibraryDetails { interface: key })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get the library details by interface id.
	///
	/// Get the details of a library the interface id of any contained interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// use cobia::cape_open_1_2;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let library_details = library_enumerator.get_library_by_interface_id(&cape_open_1_2::ICAPEIDENTIFICATION_UUID).unwrap();
	/// assert_eq!(library_details.get_name().unwrap(),"CAPEOPEN_1_2");
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_library_by_interface_id(&self, interface_id: &CapeUUID) -> Result<CapeLibraryDetails, COBIAError> {
		let mut key: *mut C::ICapeLibraryDetails = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).getLibraryByInterfaceUUID.unwrap())(
				(*self.interface).me,
				(interface_id as *const C::CapeUUID).cast_mut(),
				&mut key as *mut *mut C::ICapeLibraryDetails,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapeLibraryDetails { interface: key })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get all type libraries.
	///
	/// Get a collection of all registered type libraries.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let libraries = library_enumerator.libraries().unwrap();
	/// assert!(libraries.size() > 0); //normally the CAPE-OPEN type libraries are registered
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn libraries(&self) -> Result<CobiaCollection<CapeLibraryDetails>,COBIAError> {
		let mut p: *mut C::ICobiaCollection = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).getAllLibraries.unwrap())(
				(*self.interface).me,
				&mut p as *mut *mut C::ICobiaCollection
			)
		};
		if result == COBIAERR_NOERROR {
			if p.is_null() {
				Err(COBIAError::Code(COBIAERR_NULLPOINTER))
			} else {
				Ok(CobiaCollection::attach(p))
			}
		} else {
			Err(COBIAError::Code(result))
		}
	}
	
}

/// Release pointer
///
/// ICapeTypeLibraries derives from ICobiaBase, which contains
/// addReference() and release(). The Drop trait calls release.

impl Drop for CapeTypeLibraries {
	fn drop(&mut self) {
		unsafe {
			((*(*self.interface).vTbl).base.release.unwrap())((*self.interface).me);
		}
	}
}

/// Add pointer reference
///
/// ICapeTypeLibraries derives from ICobiaBase, which contains
/// addReference() and release(). The Clone trait calls addReference.

impl Clone for CapeTypeLibraries {
	fn clone(&self) -> Self {
		unsafe {
			((*(*self.interface).vTbl).base.addReference.unwrap())((*self.interface).me);
		}
		CapeTypeLibraries {
			interface: self.interface,
		}
	}
}

