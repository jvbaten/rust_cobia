use std::fmt;
use crate::C;
use crate::*;

/// CapeValueIn wraps a reference to an ICapeValue interface pointer.
///
/// Given a reference to an ICapeValue interface pointer, this allows getting,
/// but not setting, the value.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeValue input arguments.
///
/// This class takes a  reference to the interface pointer.
///
/// A NULL pointer is treated as an empty value.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(v: &CapeValueIn) {
///     assert_eq!(v.get_string().unwrap(), "my value".to_string());
/// }
/// 
/// let val = cobia::CapeValueImpl::from_string("my value".into());
/// test_content(&CapeValueInFromProvider::from(&val).as_cape_value_in())
/// ```

pub struct CapeValueIn<'a> {
	interface: &'a *mut C::ICapeValue,
}

impl<'a> CapeValueIn<'a> {

	/// Create v new CapeValueIn from an ICapeValue interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeValue interface
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let val = cobia::CapeValueImpl::from_integer(-8);
	///	let i_cape_value=val.as_cape_value_in();
	///	let mut i_cape_value_ptr=(&i_cape_value as *const C::ICapeValue).cast_mut(); //normally a pointer to the interface is received
	///	let v = cobia::CapeValueIn::new(&mut i_cape_value_ptr); //CapeValueIn from *mut C::ICapeValue
	/// assert_eq!(v.get_integer().unwrap(), -8);
	/// ```

	pub fn new(interface: &'a *mut C::ICapeValue) -> Self {
		Self {
			interface,
		}
	}

	/// Get the value type
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_type(v: &CapeValueIn) {
	///     assert_eq!(v.get_type().unwrap(), cobia::CapeValueType::String);
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_string("hydrogen".to_string());
	/// test_type(&CapeValueInFromProvider::from(&val).as_cape_value_in())
	/// ```

	pub fn get_type(&self) -> Result<CapeValueType,COBIAError> {
		if self.interface.is_null() {
			return Ok(CapeValueType::Empty);
		}
		let tp= unsafe {(*(**self.interface).vTbl).getValueType.unwrap()((**self.interface).me)};
		match CapeValueType::from(tp as i32) {
			Some(v) => Ok(v),
			None => Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Get the value as a string
	///
	/// Get the value as a string. If the value is not a string, an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_string(v: &CapeValueIn) {
	///     assert_eq!(v.get_string().unwrap(), "hydrogen".to_string());
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_string("hydrogen".to_string());
	/// test_string(&CapeValueInFromProvider::from(&val).as_cape_value_in())
	/// ```

	pub fn get_string(&self) -> Result<String,COBIAError> {
		if self.interface.is_null() {
			return Err(COBIAError::Code(COBIAERR_NOSUCHITEM));
		}
		let mut data: *const C::CapeCharacter = std::ptr::null();
		let mut size: C::CapeSize = 0;
		let res= unsafe {(*(**self.interface).vTbl).getStringValue.unwrap()((**self.interface).me,&mut data,&mut size)};
		if res==COBIAERR_NOERROR {
			Ok(unsafe {CapeStringImpl::from_raw_data(data,size)}.as_string())
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Get the value as an integer
	///
	/// Get the value as an integer. If the value is not an integer, an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_integer(v: &CapeValueIn) {
	///     assert_eq!(v.get_integer().unwrap(), 8);
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_integer(8);
	/// test_integer(&CapeValueInFromProvider::from(&val).as_cape_value_in())
	/// ```

	pub fn get_integer(&self) -> Result<i32,COBIAError> {
		if self.interface.is_null() {
			return Err(COBIAError::Code(COBIAERR_NOSUCHITEM));
		}
		let mut value: CapeInteger = 0;
		let res= unsafe {(*(**self.interface).vTbl).getIntegerValue.unwrap()((**self.interface).me,&mut value)};
		if res==COBIAERR_NOERROR {
			Ok(value)
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Get the value as a boolean
	///
	/// Get the value as a boolean. If the value is not a boolean, an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_boolean(v: &CapeValueIn) {
	///     assert_eq!(v.get_boolean().unwrap(), true);
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_boolean(true);
	/// test_boolean(&CapeValueInFromProvider::from(&val).as_cape_value_in())
	/// ```

	pub fn get_boolean(&self) -> Result<bool,COBIAError> {
		if self.interface.is_null() {
			return Err(COBIAError::Code(COBIAERR_NOSUCHITEM));
		}
		let mut value: CapeBoolean = 0;
		let res= unsafe {(*(**self.interface).vTbl).getBooleanValue.unwrap()((**self.interface).me,&mut value)};
		if res==COBIAERR_NOERROR {
			Ok(value!=0)
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Get the value as a real
	///
	/// Get the value as a real. If the value is not a real, an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_real(v: &CapeValueIn) {
	///     assert_eq!(v.get_real().unwrap(), 2.01568);
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_real(2.01568);
	/// test_real(&CapeValueInFromProvider::from(&val).as_cape_value_in())
	/// ```

	pub fn get_real(&self) -> Result<f64,COBIAError> {
		if self.interface.is_null() {
			return Err(COBIAError::Code(COBIAERR_NOSUCHITEM));
		}
		let mut value: CapeReal = 0.0;
		let res= unsafe {(*(**self.interface).vTbl).getRealValue.unwrap()((**self.interface).me,&mut value)};
		if res==COBIAERR_NOERROR {
			Ok(value)
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

}

impl<'a> fmt::Display for CapeValueIn<'a> {

	/// Display the content of the value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_format(v: &CapeValueIn) {
	///     assert_eq!(format!("{}", v), "\"1333-74-0\"");
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_string("1333-74-0".into());
	/// test_format(&CapeValueInFromProvider::from(&val).as_cape_value_in())
	/// ```

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.get_type() {
			Ok(CapeValueType::String) => {
				match self.get_string() {
					Ok(s) => write!(f, "\"{}\"",s),
					_ => write!(f, "<invalid string>")
				}
			},
			Ok(CapeValueType::Integer) => {
				match self.get_integer() {
					Ok(i) => write!(f, "{}",i),
					_ => write!(f, "<invalid integer>")
				}
			},
			Ok(CapeValueType::Boolean) => {
				match self.get_boolean() {
					Ok(b) => {
						if b {
							write!(f, "true")
						} else {
							write!(f, "false")
						}
					},
					_ => write!(f, "<invalid boolean>")
				}
			},
			Ok(CapeValueType::Real) => {
				match self.get_real() {
					Ok(r) => write!(f, "{}",r),
					_ => write!(f, "<invalid real>")
				}
			},
			Ok(CapeValueType::Empty) => write!(f, "<empty>"),
			_ => write!(f, "<invalid value type>")
		}
	}
}

impl<'a> CapeValueProviderIn for CapeValueIn<'a> {
	fn as_cape_value_in(&self) -> C::ICapeValue {
		unsafe { **self.interface }
	}
}

/// CapeValueOut wraps an ICapeValue interface pointer.
///
/// Given a reference to an ICapeValue interface pointer, this allows setting
///  and getting the value.
///
/// A reference to a NULL pointer is not allowed here.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeValue ouput arguments.
///
/// This class takes a mutable reference to the interface pointer, as
/// it should be the only class that is in use at a time to change the 
/// data behind the interface (as the data pointer is cached)
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_value(v: &mut CapeValueOut) {
///     v.set_string("my value").unwrap();
/// }
/// 
/// let mut val = cobia::CapeValueImpl::new();
/// set_value(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
/// assert_eq!(val.value(), CapeValueContent::String("my value".into()));
/// ```

pub struct CapeValueOut<'a> {
	interface: &'a mut *mut C::ICapeValue,
}

impl<'a> CapeValueOut<'a> {

	/// Create v new CapeValueOut from an ICapeValue interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeValue interface
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut val = cobia::CapeValueImpl::from_integer(-8);
	///	let i_cape_value=val.as_cape_value_out();
	///	let mut i_cape_value_ptr=(&i_cape_value as *const C::ICapeValue).cast_mut(); //normally a pointer to the interface is received
	///	let v = cobia::CapeValueOut::new(&mut i_cape_value_ptr); //CapeValueOut from *mut C::ICapeValue
	/// assert_eq!(v.get_integer().unwrap(), -8);
	/// ```

	pub fn new(interface: &'a mut *mut C::ICapeValue) -> Self {
		Self {
			interface,
		}
	}

	/// Get the value type
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_type(v: &mut CapeValueOut) {
	///     assert_eq!(v.get_type().unwrap(), cobia::CapeValueType::String);
	/// }
	/// 
	/// let mut val = cobia::CapeValueImpl::from_string("hydrogen".into());
	/// check_type(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
	/// ```

	pub fn get_type(&self) -> Result<CapeValueType,COBIAError> {
		let tp= unsafe {(*(**self.interface).vTbl).getValueType.unwrap()((**self.interface).me)};
		match CapeValueType::from(tp as i32) {
			Some(v) => Ok(v),
			None => Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Get the value as a string
	///
	/// Get the value as a string. If the value is not a string, an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_string(v: &mut CapeValueOut) {
	///     assert_eq!(v.get_string().unwrap(), "hydrogen".to_string());
	/// }
	/// 
	/// let mut val = cobia::CapeValueImpl::from_string("hydrogen".into());	
	/// check_string(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
	/// ```

	pub fn get_string(&self) -> Result<String,COBIAError> {
		let mut data: *const C::CapeCharacter = std::ptr::null();
		let mut size: C::CapeSize = 0;
		let res= unsafe {(*(**self.interface).vTbl).getStringValue.unwrap()((**self.interface).me,&mut data,&mut size)};
		if res==COBIAERR_NOERROR {
			Ok(unsafe {CapeStringImpl::from_raw_data(data,size)}.as_string())
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Get the value as an integer
	///
	/// Get the value as an integer. If the value is not an integer, an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_integer(v: &mut CapeValueOut) {
	///     assert_eq!(v.get_integer().unwrap(), 8);
	/// }
	/// 
	/// let mut val = cobia::CapeValueImpl::from_integer(8);
	/// check_integer(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
	/// ```

	pub fn get_integer(&self) -> Result<i32,COBIAError> {
		let mut value: CapeInteger = 0;
		let res= unsafe {(*(**self.interface).vTbl).getIntegerValue.unwrap()((**self.interface).me,&mut value)};
		if res==COBIAERR_NOERROR {
			Ok(value)
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Get the value as a boolean
	///
	/// Get the value as a boolean. If the value is not a boolean, an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_boolean(v: &mut CapeValueOut) {
	///     assert_eq!(v.get_boolean().unwrap(), true);
	/// }
	/// 
	/// let mut val = cobia::CapeValueImpl::from_boolean(true);
	/// check_boolean(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
	/// ```

	pub fn get_boolean(&self) -> Result<bool,COBIAError> {
		let mut value: CapeBoolean = 0;
		let res= unsafe {(*(**self.interface).vTbl).getBooleanValue.unwrap()((**self.interface).me,&mut value)};
		if res==COBIAERR_NOERROR {
			Ok(value!=0)
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Get the value as a real
	///
	/// Get the value as a real. If the value is not a real, an error is returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_real(v: &mut CapeValueOut) {
	///     assert_eq!(v.get_real().unwrap(), 2.01568);
	/// }
	/// 
	/// let mut val = cobia::CapeValueImpl::from_real(2.01568);
	/// check_real(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
	/// ```

	pub fn get_real(&self) -> Result<f64,COBIAError> {
		let mut value: CapeReal = 0.0;
		let res= unsafe {(*(**self.interface).vTbl).getRealValue.unwrap()((**self.interface).me,&mut value)};
		if res==COBIAERR_NOERROR {
			Ok(value)
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Set the value as a string
	///
	/// # Arguments
	///
	/// * `value` - The string value to set
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_value(v: &mut CapeValueOut) {
	///     v.set_string("my value").unwrap();
	/// }
	/// 
	/// let mut val = cobia::CapeValueImpl::new();
	/// set_value(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
	/// assert_eq!(val.value(), CapeValueContent::String("my value".into()));
	/// ```

	pub fn set_string<T: AsRef<str>>(&self, value: T) -> Result<(),COBIAError> {
		let val = CapeStringImpl::from_string(value);
		let (ptr, sz) = val.as_capechar_const_with_length();
		let res= unsafe {(*(**self.interface).vTbl).setStringValue.unwrap()((**self.interface).me,ptr,sz)};
		if res==COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Set the value as an integer
	///
	/// # Arguments
	///
	/// * `value` - The integer value to set
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_value(v: &mut CapeValueOut) {
	///     v.set_integer(8).unwrap();
	/// }
	/// 
	/// let mut val = cobia::CapeValueImpl::new();
	/// set_value(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
	/// assert_eq!(val.value(), CapeValueContent::Integer(8));
	/// ```

	pub fn set_integer(&self, value: CapeInteger) -> Result<(),COBIAError> {
		let res= unsafe {(*(**self.interface).vTbl).setIntegerValue.unwrap()((**self.interface).me,value)};
		if res==COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Set the value as a boolean
	///
	/// # Arguments
	///
	/// * `value` - The boolean value to set
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_value(v: &mut CapeValueOut) {
	///     v.set_boolean(true).unwrap()
	/// }
	/// 
	/// let mut val = cobia::CapeValueImpl::new();
	/// set_value(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
	/// assert_eq!(val.value(), CapeValueContent::Boolean(true));
	/// ```

	pub fn set_boolean(&self, value: bool) -> Result<(),COBIAError> {
		let res= unsafe {(*(**self.interface).vTbl).setBooleanValue.unwrap()((**self.interface).me,value as CapeBoolean)};
		if res==COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Set the value as a real
	///
	/// # Arguments
	///
	/// * `value` - The real value to set
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_value(v: &mut CapeValueOut) {
	///     v.set_real(2.01568).unwrap();
	/// }
	/// 
	/// let mut val = cobia::CapeValueImpl::new();
	/// set_value(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
	/// assert_eq!(val.value(), cobia::cape_value_impl::CapeValueContent::Real(2.01568));
	/// ```

	pub fn set_real(&self, value: CapeReal) -> Result<(),COBIAError> {
		let res= unsafe {(*(**self.interface).vTbl).setRealValue.unwrap()((**self.interface).me,value)};
		if res==COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Set the value to empty
	///
	/// Set the value to empty. This is equivalent to setting the value to None in rust.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_value(v: &mut CapeValueOut) {
	///     v.set_empty().unwrap();
	/// }
	/// 
	/// let mut val = cobia::CapeValueImpl::new();
	/// set_value(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
	/// assert_eq!(val.value(), cobia::cape_value_impl::CapeValueContent::Empty);
	/// ```

	pub fn set_empty(&self) -> Result<(),COBIAError> {
		let res= unsafe {(*(**self.interface).vTbl).clear.unwrap()((**self.interface).me)};
		if res==COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
		}
	}

	/// Set the value from a CapeValueProviderIn
	///
	/// Set the value from a CapeValueProviderIn. This allows setting the value from any type that implements the CapeValueProviderIn trait.
	///
	/// # Arguments
	///
	/// * `value` - A reference to a type that implements the CapeValueProviderIn trait
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut val = cobia::CapeValueImpl::new();
	/// let val1 = cobia::CapeValueImpl::from_string("hydrogen".into());
	/// CapeValueOutFromProvider::from(&mut val).as_cape_value_out().set(&val1);
	/// assert_eq!(val.value(), cobia::cape_value_impl::CapeValueContent::String("hydrogen".into()));
	/// ```

	pub fn set<T: CapeValueProviderIn>(&mut self, value: &T) -> Result<(),COBIAError> {
		let mut value_in_from_provider = CapeValueInFromProvider::from(value);
		let value=value_in_from_provider.as_cape_value_in();
		match value.get_type() {
			Ok(CapeValueType::String) => self.set_string(value.get_string()?)?,
			Ok(CapeValueType::Integer) => self.set_integer(value.get_integer()?)?,
			Ok(CapeValueType::Boolean) => self.set_boolean(value.get_boolean()?)?,
			Ok(CapeValueType::Real) => self.set_real(value.get_real()?)?,
			Ok(CapeValueType::Empty) => self.set_empty()?,
			_ => return Err(COBIAError::Code(COBIAERR_NOSUCHITEM)),
		}
		Ok(())
	}

}

impl<'a> fmt::Display for CapeValueOut<'a> {

	/// Display the content of the value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_format(v: &mut CapeValueOut) {
	///     assert_eq!(format!("{}", v), "\"1333-74-0\"");
	/// }
	/// 
	/// let mut val = cobia::CapeValueImpl::from_string("1333-74-0".into());
	/// test_format(&mut CapeValueOutFromProvider::from(&mut val).as_cape_value_out());
	/// ```

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.get_type() {
			Ok(CapeValueType::String) => {
				match self.get_string() {
					Ok(s) => write!(f, "\"{}\"",s),
					_ => write!(f, "<invalid string>")
				}
			},
			Ok(CapeValueType::Integer) => {
				match self.get_integer() {
					Ok(i) => write!(f, "{}",i),
					_ => write!(f, "<invalid integer>")
				}
			},
			Ok(CapeValueType::Boolean) => {
				match self.get_boolean() {
					Ok(b) => {
						if b {
							write!(f, "true")
						} else {
							write!(f, "false")
						}
					},
					_ => write!(f, "<invalid boolean>")
				}
			},
			Ok(CapeValueType::Real) => {
				match self.get_real() {
					Ok(r) => write!(f, "{}",r),
					_ => write!(f, "<invalid real>")
				}
			},
			Ok(CapeValueType::Empty) => write!(f, "<empty>"),
			_ => write!(f, "<invalid value type>")
		}
	}
}

impl<'a> CapeValueProviderOut for CapeValueOut<'a> {
	fn as_cape_value_out(&mut self) -> C::ICapeValue {
		unsafe { **self.interface }
	}
}