use crate::C;
use crate::*;

/// Rust wrapper for ICapePMCRegistrar interface
///
/// A CapeRegistrar object is passed to the `registration_details` function
/// of the `CapePMC` trait implementation. It is used to register the component
/// in the COBIA registry.

pub struct CapeRegistrar {
	/// Pointer to the ICapePMCRegistrar interface
	pub(crate) interface: *mut C::ICapePMCRegistrar,
}

impl CapeRegistrar {

	/// Put the (default) name of the component
	///
	/// # Arguments
	/// * `name` - The name of the component
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn put_name(&self, name: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putName.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(name)
					.as_capechar_const()
					.cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Put the (default) description of the component
	///
	/// # Arguments
	/// * `description` - The description of the component
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn put_description(&self, description: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putDescription.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(description)
					.as_capechar_const()
					.cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Put the CAPE-OPEN version of the component
	///
	/// # Arguments
	/// * `cape_version` - The CAPE-OPEN version of the component, e.g. "1.2"
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn put_cape_version(&self, cape_version: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putCapeVersion.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(cape_version)
					.as_capechar_const()
					.cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Put the component software version
	///
	/// # Arguments
	/// * `component_version` - The version of the component, e.g. "1.0.0"
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn put_component_version(&self, component_version: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putComponentVersion.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(component_version)
					.as_capechar_const()
					.cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Put the vendor URL of the component
	///
	/// # Arguments
	/// * `vendor_url` - The URL of the vendor's product website
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn put_vendor_url(&self, vendor_url: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putVendorURL.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(vendor_url)
					.as_capechar_const()
					.cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Put the help URL of the component
	///
	/// # Arguments
	/// * `help_url` - The URL of the component's help documentation
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn put_help_url(&self, help_url: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putHelpURL.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(help_url)
					.as_capechar_const()
					.cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Put the about information of the component
	///
	/// # Arguments
	/// * `about` - The about information of the component, typically a short description
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn put_about(&self, about: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putAbout.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(about)
					.as_capechar_const()
					.cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Put the UUID of the component
	///
	/// # Arguments
	/// * `uuid` - The UUID of the component, a unique identifier to identify the component
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn put_uuid(&self, uuid: &CapeUUID) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putUUID.unwrap())(
				(*self.interface).me,
				uuid as *const C::CapeUUID,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Put the ProgID of the component
	///
	/// The ProgID is a programmatic human readable ID that is a placeholder for the component's UUID.
	///
	/// The ProgID points to this specific version of the component. The version-independent ProgID
	/// is used to identify the component regardless of its version.
	///
	/// # Arguments
	/// * `prog_id` - The ProgID of the component, a string identifier used in COM

	pub fn put_prog_id(&self, prog_id: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putProgId.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(prog_id)
					.as_capechar_const()
					.cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Put the version-independent ProgID of the component
	///
	/// The version-independent ProgID is used to identify the component regardless of its version.
	/// If a PME chooses to store this information, a subsequent software update of the component
	/// will cause the PME to update to the new version.
	///s
	/// # Arguments
	/// * `prog_id` - The version-independent ProgID of the component, a string identifier used in COM
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn put_version_independent_prog_id(&self, prog_id: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl)
				.putVersionIndependentProgId
				.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(prog_id)
					.as_capechar_const()
					.cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Add a category ID to the component
	///
	/// Typically a PMC registers with at least one category ID to indicate the type of component
	/// it is, such as a unit operation, property set, or data type. Also at least one category ID
	/// should be added to identify which CAPE-OPEN version the component supports.
	///
	/// # Arguments
	/// * `cat_id` - The category ID to add, a unique identifier for the component's category
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn add_cat_id(&self, cat_id: &CapeUUID) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).addCatID.unwrap())(
				(*self.interface).me,
				cat_id as *const C::CapeUUID,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Put the creation flags for the component
	///
	/// If the component has specific restrictions, such as being suitable only
	/// for the restricted threading model, this method can be used to set those flags.
	///
	/// # Arguments
	/// * `flags` - The flags to set, indicating the component's creation restrictions
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn put_flags(&self, flags: CapePMCRegistrationFlags) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putFlags.unwrap())((*self.interface).me, flags.bits())
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Add a location for the component
	///
	/// Components can be registered for multiple service types,
	/// such as 32-bit and 64-bit versions for Windows. Eeach
	/// service type has its own location identifier, such as a
	/// DLL or shared object for an in-process implementation.
	///
	/// # Arguments
	/// * `service_type` - The service type for which the location is being registered
	/// * `location` - The location identifier, such as a DLL path
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn add_location(
		&self,
		service_type: CapePMCServiceType,
		location: &str,
	) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).addLocation.unwrap())(
				(*self.interface).me,
				service_type as i32,
				CapeStringImpl::from_string(location)
					.as_capechar_const()
					.cast_mut(),
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Commit the registration of the component
	///
	/// After all the necessary information has been provided, this method is called to finalize
	/// registration.
	///
	/// # Returns
	/// * Result<(), COBIAError> - Ok if successful, Err if there was an error

	pub fn commit(&self) -> Result<(), COBIAError> {
		let result = unsafe { ((*(*self.interface).vTbl).commit.unwrap())((*self.interface).me) };
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}
}

/// Release pointer
///
/// ICapeRegistrar derives from ICobiaBase, which contains
/// addReference() and release(). The Drop trait calls release.

impl Drop for CapeRegistrar {
	fn drop(&mut self) {
		unsafe {
			((*(*self.interface).vTbl).base.release.unwrap())((*self.interface).me);
		}
	}
}

/// Add pointer reference
///
/// ICapeRegistrar derives from ICobiaBase, which contains
/// addReference() and release(). The Clone trait calls addReference.

impl Clone for CapeRegistrar {
	fn clone(&self) -> Self {
		unsafe {
			((*(*self.interface).vTbl).base.addReference.unwrap())((*self.interface).me);
		}
		CapeRegistrar {
			interface: self.interface,
		}
	}
}
