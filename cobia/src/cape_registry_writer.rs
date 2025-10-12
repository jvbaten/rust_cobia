use crate::C;
use crate::*;
use std::path::Path;

/// Opens the COBIA registry for writing
///
/// This struct is used to write to the COBIA registry. To 
/// write values in the registry one obtains a sub-key
/// using create_key, which is created if it does not already
/// exist.
///
/// Values are not written to the registry until commit() is
/// called.
///
/// # Example
///
/// ```
/// use cobia::*;
/// cobia::cape_open_initialize().unwrap();
/// let writer = CapeRegistryWriter::new(false).unwrap();
/// cobia::cape_open_cleanup();
/// ```

pub struct CapeRegistryWriter {
	interface: *mut C::ICapeRegistryWriter,
}

impl CapeRegistryWriter {

	/// Opens the COBIA registry for writing
	///
	/// This function opens the COBIA registry for writing. If all_users is true
	/// the registry is opened for all users, otherwise it is opened for the current
	/// user.
	///
	/// Opening the registry for writing to the all-users hive typically requires 
	/// administrative privileges.
	///
	/// # Arguments
	///
	/// * `all_users` - If true the registry is opened for all users, otherwise it is opened for the current user.
	///
	/// # Returns
	///
	/// A CapeRegistryWriter object if successful, otherwise a COBIAError.
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn new(all_users: bool) -> Result<CapeRegistryWriter, COBIAError> {
		let mut writer: *mut C::ICapeRegistryWriter = std::ptr::null_mut();
		let result = unsafe {
			C::capeGetRegistryWriter(all_users, &mut writer as *mut *mut C::ICapeRegistryWriter)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapeRegistryWriter { interface: writer })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Creates or opens key in the registry
	///
	/// This function creates or opens a key in the registry. If the key does not
	/// exist it is created.
	///
	/// # Arguments
	///
	/// * `key_name` - The name of the key to create or open. Must be an absolute path, starting with a forward slash.
	///
	/// # Returns
	///
	/// A CapeRegistryKeyWriter object if successful, otherwise a COBIAError.
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// let key=writer.create_key("/cobia_rust_create_key").unwrap();
	/// //note the key does not actually appear in the registry at this point, as we did
	/// //not call commit().
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn create_key(&self, key_name: &str) -> Result<CapeRegistryKeyWriter<'_>, COBIAError> {
		let mut key: *mut C::ICapeRegistryKeyWriter = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).createKey.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(key_name)
					.as_capechar_const()
					.cast_mut(),
				&mut key as *mut *mut C::ICapeRegistryKeyWriter,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapeRegistryKeyWriter { interface: key, writer:self })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Opens key in the registry for reading.
	///
	/// This function returns a read-only key; for a writable key, use create_key instead.
	///
	/// This function opens an existing key in the registry. If the key does not
	/// exist it fails.
	///
	/// # Arguments
	///
	/// * `key_name` - The name of the key to open. Must be an absolute path, starting with a forward slash.
	///
	/// # Returns
	///
	/// A CapeRegistryKeyWriter object if successful, otherwise a COBIAError.
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// let key=writer.get_key("/types").unwrap();
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_key(&self, key_name: &str) -> Result<CapeRegistryKey, COBIAError> {
		let mut key: *mut C::ICapeRegistryKey = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).getKey.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(key_name)
					.as_capechar_const()
					.cast_mut(),
				&mut key as *mut *mut C::ICapeRegistryKey,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapeRegistryKey { interface: key })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Deletes key in the registry
	///
	/// This function deletes a key in the registry. If the key does not
	/// exist it succeeds. Changes to do not take effect until commit() is called.
	///
	/// # Arguments
	///
	/// * `key_name` - The name of the key to delete. Must be an absolute path, starting with a forward slash.
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// //first create a sub key for the current user
	/// {
	///    let writer = CapeRegistryWriter::new(false).unwrap();
	///    writer.create_key("/cobia_rust_delete_key").unwrap();
	///    writer.commit().unwrap();
	/// }
	/// //now delete the key for the current user
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.delete_key("/cobia_rust_delete_key").unwrap();
	/// writer.commit().unwrap();
	/// cobia::cape_open_cleanup();
	/// ```
	/// 
	/// This will succeed, as the key does not exist:
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.delete_key("/key_that_does_not_exist").unwrap();
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn delete_key(&self, key_name: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).deleteKey.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(key_name)
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

	/// Deletes a value in the registry
	///
	/// This function deletes a value in the registry. If the value does not
	/// exist it fails. Changes to do not take effect until commit() is called.
	///
	/// # Arguments
	///
	/// * `key_name` - The name of the key containing the value. Must be an absolute path, starting with a forward slash.
	/// * `value_name` - The name of the value to delete.
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// //first create a sub key for the current user
	/// {
	///    let writer = CapeRegistryWriter::new(false).unwrap();
	///    writer.create_key("/cobia_rust_delete_value").unwrap().
	///     set_string_value("test_value", "test_value").unwrap();
	///    writer.commit().unwrap();
	/// }
	/// //now delete the value for the current user
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.delete_value("/cobia_rust_delete_value","test_value").unwrap();
	/// writer.commit().unwrap();
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn delete_value(&self, key_name: &str, value_name: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).deleteValue.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(key_name)
					.as_capechar_const()
					.cast_mut(),
				CapeStringImpl::from_string(value_name)
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

	/// Get a registrar object
	///
	/// Registrar object facilitate the registration of PMCs.
	///
	/// This function is not typically called directly; it is used
	/// in the self-registation entry point, which is typically
	/// generated using the pmc_entry_points! macro.

	pub fn get_pmc_registrar(&self) -> Result<CapeRegistrar, COBIAError> {
		let mut registrar: *mut C::ICapePMCRegistrar = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).getPMCRegistrar.unwrap())(
				(*self.interface).me,
				&mut registrar as *mut *mut C::ICapePMCRegistrar,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapeRegistrar {
				interface: registrar,
			})
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Unregister a PMC by uuid.
	///
	/// This function unregisters a PMC by uuid. 
	/// Changes to do not take effect until commit() is called.
	///
	/// This function is not typically called directly; it is used
	/// in the self-unregistation entry point, which is typically
	/// generated using the pmc_entry_points! macro.

	pub fn unregister_pmc(&self, uuid: &CapeUUID) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).unregisterPMC.unwrap())(
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

	/// Unregister a service for a PMC by uuid.
	///
	/// This function unregisters service for a PMC by uuid. 
	/// Changes to do not take effect until commit() is called.
	///
	/// This function is not typically called directly; it is used
	/// in the self-unregistation entry point, which is typically
	/// generated using the pmc_entry_points! macro.

	pub fn unregister_pmc_service(
		&self,
		uuid: &CapeUUID,
		service: CapePMCServiceType,
	) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).unregisterPMCService.unwrap())(
				(*self.interface).me,
				uuid as *const C::CapeUUID,
				service as i32,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Commit changes to the registry
	///
	/// This function commits changes to the registry. Changes are not
	/// written to the registry until this function is called.
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.create_key("/cobia_rust_commit").unwrap().
	/// set_string_value("test_value", "test_value").unwrap();
	/// writer.commit().unwrap();
	/// //now the key and value are in the registry
	/// let key=writer.get_key("/cobia_rust_commit").unwrap();
	/// let value=key.get_string_value("test_value",None).unwrap();
	/// assert_eq!(value,"test_value");
	/// //delete the key
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.delete_key("/cobia_rust_commit").unwrap();
	/// writer.commit().unwrap();
	/// cobia::cape_open_cleanup();

	pub fn commit(&self) -> Result<(), COBIAError> {
		let result = unsafe { ((*(*self.interface).vTbl).commit.unwrap())((*self.interface).me) };
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Revert changes to the registry
	///
	/// This function ignores all changes made to the registry 
	/// since the last commit without committing them.
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.create_key("/rust_cobia_test_key").unwrap().
	/// set_string_value("test_value", "test_value").unwrap();
	/// writer.revert().unwrap();
	/// //start over
	/// writer.create_key("/rust_cobia_test_key").unwrap().
	/// set_string_value("test_value", "1-2-3-test").unwrap();
	/// cobia::cape_open_cleanup();

	pub fn revert(&self) -> Result<(), COBIAError> {
		let result = unsafe { ((*(*self.interface).vTbl).revert.unwrap())((*self.interface).me) };
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Register types from IDL
	///
	/// This function registers types from IDL files. This function is not 
	/// typically called. It is called for example by the cobiaRegister
	/// registration tool.
	///
	/// # Arguments
	///
	/// * `idl_files` - A vector of strings containing the paths to the IDL files.

	pub fn register_types_from_idl<T: AsRef<str>>(
		&self,
		idl_files: &[T],
	) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).registerTypesFromIDL.unwrap())(
				(*self.interface).me,
				(&CapeArrayStringVec::from_slice(idl_files).as_cape_array_string_in() as *const C::ICapeArrayString).cast_mut()
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Register types from IDL
	///
	/// This function registers types from IDL files. This function is not 
	/// typically called. It is called for example by the cobiaRegister
	/// registration tool.
	///
	/// # Arguments
	///
	/// * `idl_files` - A vector of Paths to the IDL files.

	pub fn register_types_from_idl_paths(&self, idl_files: &[&Path]) -> Result<(), COBIAError> {
		let mut paths_as_string = Vec::<String>::new();
		paths_as_string.reserve(idl_files.len());
		for p in idl_files {
			paths_as_string.push(p.to_str().unwrap().to_string());
		}
		self.register_types_from_idl(&paths_as_string)
	}

	///Unregister types from IDL
	///
	/// This function unregisters types from IDL files by library ID.
	/// This function is not 
	/// typically called. It is called for example by the cobiaRegister
	/// registration tool.
	/// 
	/// # Arguments
	///
	/// * `library_id` - The library ID of the types to unregister.

	pub fn unregister_types(&self, library_id: &CapeUUID) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).unregisterTypes.unwrap())(
				(*self.interface).me,
				library_id as *const C::CapeUUID,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Register proxy interface provider
	///
	/// This function registers a proxy interface provider. This function is not
	/// typically called. It is called for example by the cobiaRegister
	/// registration tool.
	///
	/// # Arguments
	///
	/// * `library_id` - The library ID of the proxy interface provider.
	/// * `service_type` - The service type of the proxy interface provider.
	/// * `location` - The location of the proxy interface provider, typically a path to a shared library.

	pub fn register_proxy_interface_provider(
		&self,
		library_id: &CapeUUID,
		service_type: CapePMCServiceType,
		location: &str,
	) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl)
				.registerProxyInterfaceProvider
				.unwrap())(
				(*self.interface).me,
				library_id as *const C::CapeUUID,
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

	/// Unregister proxy interface provider
	///
	/// This function unregisters a proxy interface provider. This function is not
	/// typically called. It is called for example by the cobiaRegister
	/// registration tool.
	///
	/// # Arguments
	///
	/// * `library_id` - The library ID of the proxy interface provider.
	/// * `service_type` - The service type of the proxy interface provider.

	pub fn unregister_proxy_interface_provider(
		&self,
		library_id: &CapeUUID,
		service_type: CapePMCServiceType,
	) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl)
				.unregisterProxyInterfaceProvider
				.unwrap())(
				(*self.interface).me,
				library_id as *const C::CapeUUID,
				service_type as i32,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}
}

/// Release pointer
///
/// ICapeRegistryWriter derives from ICobiaBase, which contains
/// addReference() and release(). The Drop trait calls release.

impl Drop for CapeRegistryWriter {
	fn drop(&mut self) {
		unsafe {
			((*(*self.interface).vTbl).base.release.unwrap())((*self.interface).me);
		}
	}
}

/// Add pointer reference
///
/// ICapeRegistryWriter derives from ICobiaBase, which contains
/// addReference() and release(). The Clone trait calls addReference.

impl Clone for CapeRegistryWriter {
	fn clone(&self) -> Self {
		unsafe {
			((*(*self.interface).vTbl).base.addReference.unwrap())((*self.interface).me);
		}
		CapeRegistryWriter {
			interface: self.interface,
		}
	}
}
