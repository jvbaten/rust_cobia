use crate::C;
use crate::*;

/// Private trait that provides the key to the registry key
/// used by CapeRegistryKeyReader trait
pub(crate) trait CapeRegistryKeyReaderKey {
	fn get_read_key(&self) -> *mut C::ICapeRegistryKey;
}

/// Public trait that provides methods to read from the registry key
///
/// This trait provides methods to read values from the registry key.
/// This is implemented by CapeRegistryKey as well as CapeRegistryKeyWriter.
///
/// # Example
///
/// ```
/// use cobia;
/// use cobia::prelude::*;
/// cobia::cape_open_initialize().unwrap();
/// let lib_key=cobia::CapeRegistryKey::from_path("/types/libraries/{8d1d724f-ab15-48e5-80e4-a612468e68d4}").unwrap(); //points to the CAPE-OPEN 1.2 type library
/// assert_eq!(lib_key.get_string_value("name",None).unwrap(), "CAPEOPEN_1_2".to_string()); //check its name
/// cobia::cape_open_cleanup();
/// ```

#[allow(private_bounds)]
pub trait CapeRegistryKeyReader: CapeRegistryKeyReaderKey {

	/// Get a list of all values names in the key
	///
	/// This method returns a list of all value names in the key.
	/// The values can be of different types: string, integer, UUID or empty.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types/libraries/{8d1d724f-ab15-48e5-80e4-a612468e68d4}").unwrap(); //points to the CAPE-OPEN 1.2 type library
	/// assert!(lib_key.get_values().unwrap().contains(&("name".to_string()))); //see that 'name' is amongst them
	/// cobia::cape_open_cleanup();
	/// ```

	fn get_values(&self) -> Result<Vec<String>, COBIAError> {
		let mut sa = CapeArrayStringVec::new();
		let iface = self.get_read_key();
		let result = unsafe {
			((*(*iface).vTbl).getValues.unwrap())((*iface).me,(&sa.as_cape_array_string_out() as *const C::ICapeArrayString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(sa.as_string_vec())
		} else {
			Err(COBIAError::Code(result))
		}
	}
	
	/// Get a list of all sub key names in the key
	///
	/// This method returns a list of all sub key names in the key.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types").unwrap(); //points to the CAPE-OPEN 1.2 type library
	/// let keys=lib_key.get_keys().unwrap();
	/// assert!(keys.contains(&("categories".to_string()))); //see that 'categories' is amongst them
	/// assert!(keys.contains(&("interfaces".to_string()))); //see that 'interfaces' is amongst them
	/// assert!(keys.contains(&("enumerations".to_string()))); //see that 'enumerations' is amongst them
	/// assert!(keys.contains(&("libraries".to_string()))); //see that 'libraries' is amongst them
	/// cobia::cape_open_cleanup();
	/// ```

	fn get_keys(&self) -> Result<Vec<String>, COBIAError> {
		let mut sa = CapeArrayStringVec::new();
		let iface = self.get_read_key();
		let result =
			unsafe { ((*(*iface).vTbl).getKeys.unwrap())((*iface).me, (&sa.as_cape_array_string_out() as *const C::ICapeArrayString).cast_mut()) };
		if result == COBIAERR_NOERROR {
			Ok(sa.as_string_vec())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get the type of a value
	///
	/// This method returns the type of a value in the key.
	/// The value can be of different types, such as string, integer, or UUID.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types/libraries/{8d1d724f-ab15-48e5-80e4-a612468e68d4}").unwrap(); //points to the CAPE-OPEN 1.2 type library
	/// assert_eq!(lib_key.get_value_type("name",None).unwrap(), cobia::CapeRegistryValueType::String); //check that 'name' is a string
	/// cobia::cape_open_cleanup();

	fn get_value_type(
		&self,
		value_name: &str,
		sub_key: Option<&str>,
	) -> Result<CapeRegistryValueType, COBIAError> {
		let mut val: i32 = 0;
		let iface = self.get_read_key();
		let string_constant;  
		let sub_key=match sub_key {
			Some(sub_key) => {
					string_constant=CapeStringImpl::from(sub_key); //must remain in scope
					string_constant.as_capechar_const()
				},
			None => std::ptr::null(),
		};
		let result = unsafe {
			((*(*iface).vTbl).getValueType.unwrap())(
				(*iface).me,
				CapeStringImpl::from_string(value_name).as_capechar_const(),
				sub_key,
				&mut val as *mut i32,
			)
		};
		if result == COBIAERR_NOERROR {
			match CapeRegistryValueType::from(val) {
				Some(v) => Ok(v),
				None => Err(COBIAError::Code(COBIAERR_REGISTRY_INVALIDVALUE)),
			}
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get a string value
	///
	/// This method returns a string value from the key.
	///
	/// # Arguments
	///
	/// * `value_name` - The name of the value to get
	/// * `sub_key` - The name of the sub key to get the value from. If None, the value is taken from the key itself.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types/interfaces/{12ebf184-f47a-4407-b52a-7fcc0a70451c}").unwrap(); //points to the CAPE-OPEN 1.2 ICapeIdentification interface
	/// assert_eq!(lib_key.get_string_value("name",None).unwrap(), "ICapeIdentification".to_string()); //check its name
	/// cobia::cape_open_cleanup();
	/// ```
	///
	/// Or, equivalently,
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types").unwrap(); //points to the types key
	/// assert_eq!(lib_key.get_string_value("name",Some("interfaces/{12ebf184-f47a-4407-b52a-7fcc0a70451c}")).unwrap(), "ICapeIdentification".to_string()); //check name of ICapeIdentification interface
	/// cobia::cape_open_cleanup();
	/// ```
	/// 
	/// This will not work, as the vlaue is not a string
	///
	/// ```should_panic
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types/interfaces/{}").unwrap(); //points to the CAPE-OPEN 1.2 type library
	/// assert_eq!(lib_key.get_string_value("version",None).unwrap(), "1.2".to_string()); //check its version
	/// let str=lib_key.get_string_value("library",None).unwrap(); //fails, this is a uuid
	/// cobia::cape_open_cleanup();
	/// ```

	fn get_string_value(
		&self,
		value_name: &str,
		sub_key: Option<&str>,
	) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let iface = self.get_read_key();
		let string_constant;
		let sub_key=match sub_key {
			Some(sub_key) => {
					string_constant=CapeStringImpl::from(sub_key); //must remain in scope
					string_constant.as_capechar_const()
			},
			None => std::ptr::null(),
		};
		let result = unsafe {
			((*(*iface).vTbl).getStringValue.unwrap())(
				(*iface).me,
				CapeStringImpl::from(value_name).as_capechar_const().cast_mut(),
				sub_key,
				(&s.as_cape_string_out() as *const C::ICapeString).cast_mut()
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get an integer value
	///
	/// This method returns an integer value from the key.
	///
	/// # Arguments
	///
	/// * `value_name` - The name of the value to get
	/// * `sub_key` - The name of the sub key to get the value from. If None, the value is taken from the key itself.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types/interfaces/{b135a443-2ed8-45ef-bb2d-e68d2e631c31}").unwrap(); //points to the CAPE-OPEN 1.2 ICapeCollection interface
	/// assert_eq!(lib_key.get_integer_value("numberOfTemplateArguments",None).unwrap(), 1); //check number of template arguments
	/// cobia::cape_open_cleanup();
	/// ```
	///
	/// Or, equivalently,
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types/interfaces").unwrap(); //points to the interfaces key
	/// assert_eq!(lib_key.get_integer_value("numberOfTemplateArguments",Some("{b135a443-2ed8-45ef-bb2d-e68d2e631c31}")).unwrap(), 1); //check number of template arguments of ICapeCollection interface
	/// cobia::cape_open_cleanup();
	/// ```

	fn get_integer_value(
		&self,
		value_name: &str,
		sub_key: Option<&str>,
	) -> Result<i32, COBIAError> {
		let mut i = 0i32;
		let iface = self.get_read_key();
		let string_constant;
		let sub_key=match sub_key {
			Some(sub_key) => {
					string_constant=CapeStringImpl::from(sub_key); //must remain in scope
					string_constant.as_capechar_const()
			},
			None => std::ptr::null(),
		};
		let result = unsafe {
			((*(*iface).vTbl).getIntegerValue.unwrap())(
				(*iface).me,
				CapeStringImpl::from(value_name).as_capechar_const().cast_mut(),
				sub_key,
				&mut i as *mut i32,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(i)
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get a UUID value
	///
	/// This method returns a UUID value from the key.
	///
	/// # Arguments
	///
	/// * `value_name` - The name of the value to get
	/// * `sub_key` - The name of the sub key to get the value from. If None, the value is taken from the key itself.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::cape_open_1_2;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types/interfaces/{12ebf184-f47a-4407-b52a-7fcc0a70451c}").unwrap(); //points to the CAPE-OPEN 1.2 ICapeIdentification interface
	/// assert_eq!(lib_key.get_uuid_value("library",None).unwrap(), cobia::cape_open_1_2::LIBRARY_ID); //check library id
	/// cobia::cape_open_cleanup();
	/// ```

	fn get_uuid_value(&self, value_name: &str, sub_key: Option<&str>) -> Result<CapeUUID, COBIAError> {
		let mut uuid = CapeUUID {
			data: [0; 16],
		};
		let iface = self.get_read_key();
		let string_constant;
		let sub_key=match sub_key {
			Some(sub_key) => {
					string_constant=CapeStringImpl::from(sub_key); //must remain in scope
					string_constant.as_capechar_const()
			},
			None => std::ptr::null(),
		};
		let result = unsafe {
			((*(*iface).vTbl).getUUIDValue.unwrap())(
				(*iface).me,
				CapeStringImpl::from(value_name).as_capechar_const().cast_mut(),
				sub_key,
				&mut uuid as *mut CapeUUID,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(uuid)
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get a sub key
	///
	/// This method returns a sub key from the key.
	///
	/// # Arguments
	///
	/// * `key_name` - The name of the sub key to get
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types/interfaces").unwrap(); //points to the interfaces key
	/// let icape_collection_key=lib_key.get_sub_key("{b135a443-2ed8-45ef-bb2d-e68d2e631c31}").unwrap(); //points to the ICapeCollection interface
	/// assert_eq!(icape_collection_key.get_string_value("name",None).unwrap(), "ICapeCollection".to_string()); //check its name
	/// cobia::cape_open_cleanup();
	/// ```

	fn get_sub_key(&self, key_name: &str) -> Result<CapeRegistryKey, COBIAError> {
		let mut key: *mut C::ICapeRegistryKey = std::ptr::null_mut();
		let iface = self.get_read_key();
		let result = unsafe {
			((*(*iface).vTbl).getSubKey.unwrap())(
				(*iface).me,
				CapeStringImpl::from(key_name).as_capechar_const(),
				&mut key as *mut *mut C::ICapeRegistryKey,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapeRegistryKey { interface: key })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Check whether a particular value is in the registry for all users or just the current user
	///
	/// This method checks whether a particular value is in the registry for all users or just the current user.
	///
	/// # Arguments
	///
	/// * `value_name` - The name of the value to check
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types/interfaces/{12ebf184-f47a-4407-b52a-7fcc0a70451c}").unwrap(); //points to the CAPE-OPEN 1.2 ICapeIdentification interface
	/// println!("The name for  CAPE-OPEN 1.2 ICapeIdentification is in the {} part of the registry", 
	///             if lib_key.is_all_users("name").unwrap() {"all users"} else {"current user"} ); 
	/// cobia::cape_open_cleanup();
	/// ```

	fn is_all_users(&self, value_name: &str) -> Result<bool, COBIAError> {
		let mut all_users = 0u32;
		let iface = self.get_read_key();
		let result = unsafe {
			((*(*iface).vTbl).isAllUsers.unwrap())(
				(*iface).me,
				CapeStringImpl::from(value_name).as_capechar_const(),
				&mut all_users as *mut u32,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(all_users != 0)
		} else {
			Err(COBIAError::Code(result))
		}
	}

}

/// A class that provides access to a COBIA registry key in a read-only manner
///
/// This class provides access to a COBIA registry key in a read-only manner; for
/// writing to the COBIA registry, consider using CapeRegistryKeyWriter.
///
/// # Example
///
/// ```
/// use cobia;
/// use cobia::prelude::*;
/// cobia::cape_open_initialize().unwrap();
/// let lib_key=cobia::CapeRegistryKey::from_path("/types/libraries/{8d1d724f-ab15-48e5-80e4-a612468e68d4}").unwrap(); //points to the CAPE-OPEN 1.2 type library
/// assert_eq!(lib_key.get_string_value("name",None).unwrap(), "CAPEOPEN_1_2".to_string()); //check its name
/// cobia::cape_open_cleanup();
/// ```

pub struct CapeRegistryKey {
	pub(crate) interface: *mut C::ICapeRegistryKey,
}

impl CapeRegistryKey {

	/// Get a reference to the root key
	///
	/// This method returns a reference to the root key.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let root_key=cobia::CapeRegistryKey::new().unwrap(); //points to the root key
	/// //note the absense of a leading slash in the sub-key to the root key
	/// assert_eq!(root_key.get_string_value("name",Some("types/libraries/{8d1d724f-ab15-48e5-80e4-a612468e68d4}")).unwrap(), "CAPEOPEN_1_2".to_string()); //check name of CAPE-OPEN 1.2 type library
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn new() -> Result<CapeRegistryKey, COBIAError> {
		let mut key: *mut C::ICapeRegistryKey = std::ptr::null_mut();
		let result = unsafe {
			C::capeGetRegistryKey(
				std::ptr::null_mut(),
				&mut key as *mut *mut C::ICapeRegistryKey,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapeRegistryKey { interface: key })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get a reference to a key from a path
	///
	/// This method returns a reference to a key from a path.
	///
	/// # Arguments
	///
	/// * `loc` - The path to the key to get
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let lib_key=cobia::CapeRegistryKey::from_path("/types/libraries/{8d1d724f-ab15-48e5-80e4-a612468e68d4}").unwrap(); //points to the CAPE-OPEN 1.2 type library
	/// assert_eq!(lib_key.get_string_value("name",None).unwrap(), "CAPEOPEN_1_2".to_string()); //check its name
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn from_path(loc: &str) -> Result<CapeRegistryKey, COBIAError> {
		let mut key: *mut C::ICapeRegistryKey = std::ptr::null_mut();
		let result = unsafe {
			C::capeGetRegistryKey(
				CapeStringImpl::from(loc).as_capechar_const(),
				&mut key as *mut *mut C::ICapeRegistryKey,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapeRegistryKey { interface: key })
		} else {
			Err(COBIAError::Code(result))
		}
	}
}

impl CapeRegistryKeyReaderKey for CapeRegistryKey {
	fn get_read_key(&self) -> *mut C::ICapeRegistryKey {
		self.interface
	}
}

impl CapeRegistryKeyReader for CapeRegistryKey {}

/// Release pointer
///
/// ICapeRegistryKey derives from ICobiaBase, which contains
/// addReference() and release(). The Drop trait calls release.

impl Drop for CapeRegistryKey {
	fn drop(&mut self) {
		unsafe {
			((*(*self.interface).vTbl).base.release.unwrap())((*self.interface).me);
		}
	}
}

/// Add pointer reference
///
/// ICapeRegistryKey derives from ICobiaBase, which contains
/// addReference() and release(). The Clone trait calls addReference.

impl Clone for CapeRegistryKey {
	fn clone(&self) -> Self {
		unsafe {
			((*(*self.interface).vTbl).base.addReference.unwrap())((*self.interface).me);
		}
		CapeRegistryKey {
			interface: self.interface,
		}
	}
}
