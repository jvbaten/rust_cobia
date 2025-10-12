use crate::C;
use crate::*;

/// A writable COBIA registry key
///
/// This struct is used to write to a COBIA registry key. It is obtained
/// from a [`CapeRegistryWriter`](crate::CapeRegistryWriter) and can be used to 
/// create subkeys and write values. It is valid only for the life
/// span of the CapeRegistryWriter.
///
/// Values are not written to the registry until CapeRegistryWriter::commit() is
/// called.

pub struct CapeRegistryKeyWriter<'a> {
	pub(crate) interface: *mut C::ICapeRegistryKeyWriter,
	pub(crate) writer: &'a CapeRegistryWriter //cannot use the key after the writer is deleted....
}

impl<'a> CapeRegistryKeyWriter<'a> {

	/// Create a subkey
	///
	/// Create a subkey with the given name. The subkey is returned as a
	/// CapeRegistryKeyWriter. The subkey can be used to write values and
	/// create further subkeys. The subkey is not written to the registry
	/// until CapeRegistryWriter::commit() is called.
	///
	/// # Arguments
	///
	/// * `key_name` - The name of the subkey to create
	///
	/// # Returns
	///
	/// A CapeRegistryKeyWriter for the new subkey
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// {
	///   //create a key, then a sub key, and put a value in there
	///   let writer = CapeRegistryWriter::new(false).unwrap();
	///   let key=writer.create_key("/cobia_rust_create_sub_key").unwrap();
	///   let sub_key=key.create_sub_key(&writer,"sub_key").unwrap();
	///   sub_key.set_string_value("test_value", "test_value").unwrap();
	///   writer.commit().unwrap();
	/// }
	/// //read the value back
	/// let reader = CapeRegistryKey::from_path("/cobia_rust_create_sub_key/sub_key").unwrap();
	/// assert_eq!(reader.get_string_value("test_value",None).unwrap(), "test_value".to_string());
	/// // now delete the entire key
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.delete_key("/cobia_rust_create_sub_key").unwrap();
	/// writer.commit().unwrap();
	/// //validate that it was deleted
	/// let reader = CapeRegistryKey::from_path("/cobia_rust_create_sub_key");
	/// assert!(reader.is_err());
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn create_sub_key(&self, writer: &'a CapeRegistryWriter, key_name: &str) -> Result<CapeRegistryKeyWriter<'_>, COBIAError> {
		let mut key: *mut C::ICapeRegistryKeyWriter = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).createSubKey.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(key_name)
					.as_capechar_const()
					.cast_mut(),
				&mut key as *mut *mut C::ICapeRegistryKeyWriter,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapeRegistryKeyWriter { interface: key, writer })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Delete a subkey
	///
	/// Delete a subkey with the given name. The result is not written to the
	/// registry until CapeRegistryWriter::commit() is called.
	///
	/// # Arguments
	///
	/// * `key_name` - The name of the subkey to delete
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// {
	///   //create a key, then a sub key, and put a value in there
	///   let writer = CapeRegistryWriter::new(false).unwrap();
	///   let key=writer.create_key("/cobia_rust_delete_sub_key").unwrap();
	///   let sub_key=key.create_sub_key(&writer,"sub_key").unwrap();
	///   sub_key.set_string_value("test_value", "test_value").unwrap();
	///   writer.commit().unwrap();
	/// }
	/// //read the value back
	/// let reader = CapeRegistryKey::from_path("/cobia_rust_delete_sub_key/sub_key").unwrap();
	/// assert_eq!(reader.get_string_value("test_value",None).unwrap(), "test_value".to_string());
	/// // now delete the sub key
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// let key=writer.create_key("/cobia_rust_delete_sub_key").unwrap();
	/// key.delete_sub_key("sub_key").unwrap();
	/// writer.commit().unwrap();
	/// //validate that it was deleted
	/// let reader = CapeRegistryKey::from_path("/cobia_rust_delete_sub_key/sub_key");
	/// assert!(reader.is_err());
	/// // now delete the entire key
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.delete_key("/cobia_rust_delete_sub_key").unwrap();
	/// writer.commit().unwrap();
	/// //validate that it was deleted
	/// let reader = CapeRegistryKey::from_path("/cobia_rust_delete_sub_key");
	/// assert!(reader.is_err());
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn delete_sub_key(&self, key_name: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).deleteSubKey.unwrap())(
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

	/// Delete a value
	///
	/// Delete a value with the given name. The result is not written to the
	/// registry until CapeRegistryWriter::commit() is called.
	///
	/// # Arguments
	///
	/// * `key_name` - The name of the subkey to delete; if None, the current key is used
	/// * `value_name` - The name of the value to delete
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// {
	///   //create a key, and put a value in there
	///   let writer = CapeRegistryWriter::new(false).unwrap();
	///   let key=writer.create_key("/cobia_rust_delete_value").unwrap();
	///   key.set_string_value("test_string_value", "I like COBIA").unwrap();
	///   writer.commit().unwrap();
	/// }
	/// //read the value back
	/// let reader = CapeRegistryKey::from_path("/cobia_rust_delete_value").unwrap();
	/// assert_eq!(reader.get_string_value("test_string_value",None).unwrap(), "I like COBIA".to_string());
	/// // now delete the value
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// let key=writer.create_key("/cobia_rust_delete_value").unwrap();
	/// key.delete_value(None,"test_string_value").unwrap();
	/// writer.commit().unwrap();
	/// //validate that it was deleted
	/// let reader = CapeRegistryKey::from_path("/cobia_rust_delete_value").unwrap();
	/// let value = reader.get_string_value("test_string_value",None);
	/// assert!(value.is_err());
	/// // now delete the entire key
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.delete_key("/cobia_rust_delete_value").unwrap();
	/// writer.commit().unwrap();
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn delete_value(&self, key_name: Option<&str>, value_name: &str) -> Result<(), COBIAError> {
		let key_name = {
			if let Some(key_name) = key_name {
				CapeStringImpl::from_string(key_name).as_capechar_const()
			} else {
				std::ptr::null()
			}
		};
		let result = unsafe {
			((*(*self.interface).vTbl).deleteValue.unwrap())(
				(*self.interface).me,
				key_name.cast_mut(),
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

	/// Write a string value
	///
	/// Write a string value with the given name. The result is not written to the
	/// registry until CapeRegistryWriter::commit() is called.
	///
	/// # Arguments
	///
	/// * `value_name` - The name of the value to write
	/// * `value` - The value to write
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// {
	///   //create a key, and put a value in there
	///   let writer = CapeRegistryWriter::new(false).unwrap();
	///   let key=writer.create_key("/cobia_rust_set_string_value").unwrap();
	///   key.set_string_value("another_string_value", "CAPE-OPEN is the defacto interop standard for chemical engineering software components").unwrap();
	///   writer.commit().unwrap();
	/// }
	/// //read the value back
	/// let reader = CapeRegistryKey::from_path("/cobia_rust_set_string_value").unwrap();
	/// assert_eq!(reader.get_string_value("another_string_value",None).unwrap(), "CAPE-OPEN is the defacto interop standard for chemical engineering software components".to_string());
	/// // now delete the entire key
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.delete_key("/cobia_rust_set_string_value").unwrap();
	/// writer.commit().unwrap();
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn set_string_value<T1: AsRef<str>,T2: AsRef<str>>(&self, value_name: T1, value: T2) -> Result<(), COBIAError> {
		let value_name=value_name.as_ref();
		let value=value.as_ref();
		let result = unsafe {
			((*(*self.interface).vTbl).putStringValue.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(value_name)
					.as_capechar_const()
					.cast_mut(),
				CapeStringImpl::from_string(value)
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

	/// Write an integer value
	///
	/// Write an integer value with the given name. The result is not written to the
	/// registry until CapeRegistryWriter::commit() is called.
	///
	/// # Arguments
	///
	/// * `value_name` - The name of the value to write
	/// * `value` - The value to write
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// {
	///   //create a key, and put a value in there
	///   let writer = CapeRegistryWriter::new(false).unwrap();
	///   let key=writer.create_key("/cobia_rust_put_integer_value").unwrap();
	///   key.put_integer_value("my_integer_value", 14).unwrap();
	///   writer.commit().unwrap();
	/// }
	/// //read the value back
	/// assert_eq!(CapeRegistryKey::from_path("/cobia_rust_put_integer_value").unwrap().
	///		get_integer_value("my_integer_value",None).unwrap(), 14);
	/// // now delete the entire key
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.delete_key("/cobia_rust_put_integer_value").unwrap();
	/// writer.commit().unwrap();
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn put_integer_value(&self, value_name: &str, value: i32) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putIntegerValue.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(value_name)
					.as_capechar_const()
					.cast_mut(),
				value,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Write an uuid value
	///
	/// Write an uuid value with the given name. The result is not written to the
	/// registry until CapeRegistryWriter::commit() is called.
	///
	/// # Arguments
	///
	/// * `value_name` - The name of the value to write
	/// * `value` - The value to write
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let test_uuid = CapeUUID::new(); //generate unique uuid
	/// {
	///   //create a key, and put a value in there
	///   let writer = CapeRegistryWriter::new(false).unwrap();
	///   let key=writer.create_key("/cobia_rust_put_uuid_value").unwrap();
	///   key.put_uuid_value("new_uuid", &test_uuid).unwrap();
	///   writer.commit().unwrap();
	/// }
	/// //read the value back
	/// assert_eq!(CapeRegistryKey::from_path("/cobia_rust_put_uuid_value").unwrap().
	///		get_uuid_value("new_uuid",None).unwrap(), test_uuid);
	/// // now delete the entire key
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.delete_key("/cobia_rust_put_uuid_value").unwrap();
	/// writer.commit().unwrap();
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn put_uuid_value(&self, value_name: &str, value: &CapeUUID) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putUUIDValue.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(value_name)
					.as_capechar_const()
					.cast_mut(),
				value as *const C::CapeUUID,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Write an empty value
	///
	/// Write an uuid value with the given name. The result is not written to the
	/// registry until CapeRegistryWriter::commit() is called.
	///
	/// Empty values are typically used to denote an option that is either
	/// set (value present) or not set (value not present). 
	///
	/// # Arguments
	///
	/// * `value_name` - The name of the value to write
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let test_uuid = CapeUUID::new(); //generate unique uuid
	/// {
	///   //create a key, and put a value in there
	///   let writer = CapeRegistryWriter::new(false).unwrap();
	///   let key=writer.create_key("/cobia_rust_put_empty_value").unwrap();
	///   key.put_empty_value("test_empty_value").unwrap();
	///   writer.commit().unwrap();
	/// }
	/// //check existence and type
	/// assert_eq!(CapeRegistryKey::from_path("/cobia_rust_put_empty_value").unwrap().
	///		get_value_type("test_empty_value",None).unwrap(), cobia::CapeRegistryValueType::Empty);
	/// // now delete the entire key
	/// let writer = CapeRegistryWriter::new(false).unwrap();
	/// writer.delete_key("/cobia_rust_put_empty_value").unwrap();
	/// writer.commit().unwrap();
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn put_empty_value(&self, value_name: &str) -> Result<(), COBIAError> {
		let result = unsafe {
			((*(*self.interface).vTbl).putEmptyValue.unwrap())(
				(*self.interface).me,
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
}

impl<'a> CapeRegistryKeyReaderKey for CapeRegistryKeyWriter<'a> {
	fn get_read_key(&self) -> *mut C::ICapeRegistryKey {
		//cast
		let p = self.interface as *mut C::ICapeRegistryKeyWriter;
		unsafe {
			std::mem::transmute::<*mut C::ICapeRegistryKeyWriter, *mut C::ICapeRegistryKey>(p)
		}
	}
}

impl<'a> CapeRegistryKeyReader for CapeRegistryKeyWriter<'a> {}

/// Release pointer
///
/// ICapeRegistryKeyWriter derives from ICobiaBase, which contains
/// addReference() and release(). The Drop trait calls release.

impl<'a> Drop for CapeRegistryKeyWriter<'a> {
	fn drop(&mut self) {
		unsafe {
			((*(*self.interface).vTbl).base.base.release.unwrap())((*self.interface).me);
		}
	}
}

/// Add pointer reference
///
/// ICapeRegistryKeyWriter derives from ICobiaBase, which contains
/// addReference() and release(). The Clone trait calls addReference.

impl<'a> Clone for CapeRegistryKeyWriter<'a> {
	fn clone(&self) -> Self {
		unsafe {
			((*(*self.interface).vTbl).base.base.addReference.unwrap())((*self.interface).me);
		}
		CapeRegistryKeyWriter {
			interface: self.interface,
			writer: self.writer
		}
	}
}
