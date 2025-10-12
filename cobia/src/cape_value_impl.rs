use crate::C;
use crate::*;

/// CapeValueImpl content
#[derive(Debug,PartialEq,Clone)]
pub enum CapeValueContent {
	Empty,
	String(std::string::String),
	Integer(CapeInteger),
	Boolean(bool),
	Real(CapeReal),
}

/// Default CapeValue implementation
///
/// ICapeValue is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation.
///
/// This class can be used as input and output argument. 
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

#[derive (Clone)]
pub struct CapeValueImpl {
	value: CapeValueContent,
	str_val: CapeStringImpl, //used to internally store string values, for which the pointer is returned to the external caller
}

impl CapeValueImpl {

	/// Create a new, empty, CapeValueImpl
	///
	/// Creates a new empty CapeValueImpl
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
	pub fn new() -> Self {
		Self {
			value: CapeValueContent::Empty,
			str_val: CapeStringImpl::new()
		}
	}

	/// Create a new CapeValueImpl from string
	///
	/// Creates a new CapeValueImpl from the given string
	///
	/// # Arguments
	///
	/// * `value` - The initial value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_value(v: &CapeValueIn) {
	///     assert_eq!(v.get_string().unwrap(), "C2H6".to_string());
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_str("C2H6");
	/// check_value(&CapeValueInFromProvider::from(&val).as_cape_value_in());
	/// ```

	pub fn from_str(value: &str) -> Self {
		Self {
			value: CapeValueContent::String(value.to_string()),
			str_val: CapeStringImpl::new()
		}
	}

	/// Create a new CapeValueImpl from string
	///
	/// Creates a new CapeValueImpl from the given string
	///
	/// # Arguments
	///
	/// * `value` - The initial value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_value(v: &CapeValueIn) {
	///     assert_eq!(v.get_string().unwrap(), "C2H6".to_string());
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_string("C2H6".to_string());
	/// check_value(&CapeValueInFromProvider::from(&val).as_cape_value_in());
	/// ```

	pub fn from_string(value: String) -> Self {
		Self {
			value: CapeValueContent::String(value),
			str_val: CapeStringImpl::new()
		}
	}

	/// Create a new CapeValueImpl from content
	///
	/// Creates a new CapeValueImpl from the given CapeValueContent
	///
	/// # Arguments
	///
	/// * `value` - The initial value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_value(v: &CapeValueIn) {
	///     assert_eq!(v.get_string().unwrap(), "n-butane".to_string());
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_content(cobia::CapeValueContent::String("n-butane".into()));
	/// check_value(&CapeValueInFromProvider::from(&val).as_cape_value_in());
	/// ```

	pub fn from_content(value: CapeValueContent) -> Self {
		Self {
			value,
			str_val: CapeStringImpl::new()
		}
	}

	/// Create a new CapeValueImpl from integer
	///
	/// Creates a new CapeValueImpl from the given integer
	///
	/// # Arguments
	///
	/// * `value` - The initial value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_value(v: &CapeValueIn) {
	///     assert_eq!(v.get_integer().unwrap(),5);
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_integer(5);
	/// check_value(&CapeValueInFromProvider::from(&val).as_cape_value_in());
	/// ```

	pub fn from_integer(value: CapeInteger) -> Self {
		Self {
			value: CapeValueContent::Integer(value),
			str_val: CapeStringImpl::new()
		}
	}

	/// Create a new CapeValueImpl from boolean
	///
	/// Creates a new CapeValueImpl from the given boolean
	///
	/// # Arguments
	///
	/// * `value` - The initial value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_value(v: &CapeValueIn) {
	///     assert_eq!(v.get_boolean().unwrap(),false);
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_boolean(false);
	/// check_value(&CapeValueInFromProvider::from(&val).as_cape_value_in());
	/// ```

	pub fn from_boolean(value: bool) -> Self {
		Self {
			value: CapeValueContent::Boolean(value),
			str_val: CapeStringImpl::new()
		}
	}

	/// Create a new CapeValueImpl from real
	///
	/// Creates a new CapeValueImpl from the given real
	///
	/// # Arguments
	///
	/// * `value` - The initial value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_value(v: &CapeValueIn) {
	///     assert_eq!(v.get_real().unwrap(),3.0);
	/// }
	/// 
	/// let val = cobia::CapeValueImpl::from_real(3.0);
	/// check_value(&CapeValueInFromProvider::from(&val).as_cape_value_in());
	/// ```

	pub fn from_real(value: CapeReal) -> Self {
		Self {
			value: CapeValueContent::Real(value),
			str_val: CapeStringImpl::new()
		}
	}

	/// Set to empty
	///
	/// Change the type to empty
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut val = cobia::CapeValueImpl::from_real(3.0);
	/// val.reset();
	/// assert_eq!(val.value(), CapeValueContent::Empty);
	/// ```
	pub fn reset(&mut self) {
		self.value=CapeValueContent::Empty;
	}

	/// Set to string
	///
	/// Change the type to the given string
	///
	/// # Arguments
	///
	/// * `value` - The new value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut val = cobia::CapeValueImpl::new();
	/// val.set_string("H2O".into());
	/// assert_eq!(val.value(), CapeValueContent::String("H2O".into()));
	/// ```

	pub fn set_string(&mut self,value: String) {
		self.value = CapeValueContent::String(value);
	}

	/// Set to string
	///
	/// Change the type to the given string
	///
	/// # Arguments
	///
	/// * `value` - The new value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut val = cobia::CapeValueImpl::new();
	/// val.set_str("H2O");
	/// assert_eq!(val.value(), CapeValueContent::String("H2O".into()));
	/// ```

	pub fn set_str<T: AsRef<str>>(&mut self,value: T) {
		self.value = CapeValueContent::String(value.as_ref().into());
	}

	/// Set to integer
	///
	/// Change the type to the given integer
	///
	/// # Arguments
	///
	/// * `value` - The new value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut val = cobia::CapeValueImpl::new();
	/// val.set_integer(5);
	/// assert_eq!(val.value(), CapeValueContent::Integer(5));
	/// ```

	pub fn set_integer(&mut self,value: CapeInteger) {
		self.value=CapeValueContent::Integer(value);
	}

	/// Set to boolean
	///
	/// Change the type to the given boolean
	///
	/// # Arguments
	///
	/// * `value` - The new value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut val = cobia::CapeValueImpl::new();
	/// val.set_boolean(true);
	/// assert_eq!(val.value(), CapeValueContent::Boolean(true));
	/// val.set_boolean(false);
	/// assert_eq!(val.value(), CapeValueContent::Boolean(false));
	/// ```

	pub fn set_boolean(&mut self,value: bool) {
		self.value=CapeValueContent::Boolean(value);
	}

	/// Set to real
	///
	/// Change the type to the given real
	///
	/// # Arguments
	///
	/// * `value` - The initial value
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut val = cobia::CapeValueImpl::new();
	/// val.set_real(3.1);
	/// assert_eq!(val.value(), CapeValueContent::Real(3.1));
	/// ```

	pub fn set_real(&mut self,value: CapeReal) {
		self.value=CapeValueContent::Real(value);
	}

	/// Set the content of the value from any object that implements CapeValueProviderIn.
	///
	/// # Arguments
	/// * `val` - An object that implements CapeValueProviderIn
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// let mut val = cobia::CapeValueImpl::new();
	/// let val1 = cobia::CapeValueImpl::from_str("test");
	/// val.set(&val1);
	/// assert_eq!(val.value(), cobia::CapeValueContent::String("test".into()));
	///
	/// fn set_value(val: &mut cobia::CapeValueImpl,v: &cobia::CapeValueIn) {
	///     val.set(v);
	/// }
	/// 
	/// let val2 = cobia::CapeValueImpl::from_str("test2");
	/// set_value(&mut val,&cobia::CapeValueInFromProvider::from(&val2).as_cape_value_in());
	/// assert_eq!(val.value(), cobia::CapeValueContent::String("test2".into()));
	/// ```
	pub fn set<T:CapeValueProviderIn>(&mut self,val:&T) -> Result<(), COBIAError> {
		let mut value_in_from_provider = CapeValueInFromProvider::from(val);
		let value=value_in_from_provider.as_cape_value_in();
		match value.get_type()? {
			CapeValueType::String => {self.set_string(value.get_string()?)},
			CapeValueType::Integer => {self.set_integer(value.get_integer()?)},
			CapeValueType::Boolean => {self.set_boolean(value.get_boolean()?);},
			CapeValueType::Real => {self.set_real(value.get_real()?)},
			CapeValueType::Empty => {self.reset()},
		}
		Ok(())
	}

	/// Get a reference to the value
	///
	/// Returns a reference to the value
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let val = cobia::CapeValueImpl::from_str("test");
	/// assert_eq!(val.value_ref(),&cobia::CapeValueContent::String("test".into()));
	/// ```

	pub fn value_ref(&self) -> &CapeValueContent {
		&self.value
	}

	/// Get a mutable reference to the value
	///
	/// Returns a mutable reference to the value
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let mut val = cobia::CapeValueImpl::from_integer(2);
	/// *val.value_ref_mut()=cobia::CapeValueContent::Boolean(false);
	/// assert_eq!(val.value_ref(),&cobia::CapeValueContent::Boolean(false));
	/// ```

	pub fn value_ref_mut(&mut self) -> &mut CapeValueContent {
		&mut self.value
	}

	/// Get a clone of the value
	///
	/// Returns a clone of the value
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let val = cobia::CapeValueImpl::from_integer(2);
	/// assert_eq!(val.value(),cobia::CapeValueContent::Integer(2));
	/// ```
	 
	pub fn value(&self) -> CapeValueContent {
		self.value.clone()
	}

	/// Interface member function

	extern "C" fn get_value_type(me: *mut ::std::os::raw::c_void) -> C::CapeValueType {
		let p = me as *mut Self;
		let myself: &mut Self = unsafe { &mut *p };
		match myself.value {
			CapeValueContent::Empty => CapeValueType::Empty as C::CapeValueType,
			CapeValueContent::String(_) => CapeValueType::String as C::CapeValueType,
			CapeValueContent::Integer(_) => CapeValueType::Integer as C::CapeValueType,
			CapeValueContent::Boolean(_) => CapeValueType::Boolean as C::CapeValueType,
			CapeValueContent::Real(_) => CapeValueType::Real as C::CapeValueType,
		}	
	}

	/// Interface member function

	extern "C" fn get_string_value(me: *mut ::std::os::raw::c_void,data: *mut *const CapeCharacter,size: *mut C::CapeSize) -> CapeResult {
		let p = me as *mut Self;
		let myself: &mut Self = unsafe { &mut *p };
		match &myself.value {
			CapeValueContent::String(s) => {
				myself.str_val.set_string(s); 
				let (d, s)=myself.str_val.as_capechar_const_with_length();
				unsafe {
					*data = d;
					*size = s;
				}
				COBIAERR_NOERROR
			}
			_ => COBIAERR_NOSUCHITEM
		}
	}

	/// Interface member function

	extern "C" fn get_integer_value(me: *mut ::std::os::raw::c_void,value: *mut CapeInteger) -> CapeResult {
		let p = me as *mut Self;
		let myself: &mut Self = unsafe { &mut *p };
		match &myself.value {
			CapeValueContent::Integer(i) => {
				unsafe {
					*value = *i;
				}
				COBIAERR_NOERROR
			}
			_ => COBIAERR_NOSUCHITEM
		}
	}

	/// Interface member function

	extern "C" fn get_boolean_value(me: *mut ::std::os::raw::c_void,value: *mut CapeBoolean) -> CapeResult {
		let p = me as *mut Self;
		let myself: &mut Self = unsafe { &mut *p };
		match &myself.value {
			CapeValueContent::Boolean(b) => {
				unsafe {
					*value = *b as CapeBoolean;
				}
				COBIAERR_NOERROR
			}
			_ => COBIAERR_NOSUCHITEM
		}
	}

	/// Interface member function

	extern "C" fn get_real_value(me: *mut ::std::os::raw::c_void,value: *mut CapeReal) -> CapeResult {
		let p = me as *mut Self;
		let myself: &mut Self = unsafe { &mut *p };
		match &myself.value {
			CapeValueContent::Real(r) => {
				unsafe {
					*value = *r;
				}
				COBIAERR_NOERROR
			}
			_ => COBIAERR_NOSUCHITEM
		}
	}

	/// Interface member function

	extern "C" fn set_string_value(me: *mut ::std::os::raw::c_void,data: *const CapeCharacter,size: C::CapeSize) -> CapeResult {
		let p = me as *mut Self;
		let myself: &mut Self = unsafe { &mut *p };
		myself.str_val = unsafe { CapeStringImpl::from_raw_data(data,size) };
		myself.value=CapeValueContent::String(myself.str_val.as_string());
		COBIAERR_NOERROR
	}

	/// Interface member function

	extern "C" fn set_integer_value(me: *mut ::std::os::raw::c_void,value: CapeInteger) -> CapeResult {
		let p = me as *mut Self;
		let myself: &mut Self = unsafe { &mut *p };
		myself.value = CapeValueContent::Integer(value);
		COBIAERR_NOERROR
	}

	/// Interface member function

	extern "C" fn set_boolean_value(me: *mut ::std::os::raw::c_void,value: CapeBoolean) -> CapeResult {
		let p = me as *mut Self;
		let myself: &mut Self = unsafe { &mut *p };
		myself.value = CapeValueContent::Boolean(value!=0);
		COBIAERR_NOERROR
	}

	/// Interface member function

	extern "C" fn set_real_value(me: *mut ::std::os::raw::c_void,value: CapeReal) -> CapeResult {
		let p = me as *mut Self;
		let myself: &mut Self = unsafe { &mut *p };
		myself.value = CapeValueContent::Real(value);
		COBIAERR_NOERROR
	}

	/// Interface member function

	extern "C" fn clear(me: *mut ::std::os::raw::c_void) -> CapeResult {
		let p = me as *mut Self;
		let myself: &mut Self = unsafe { &mut *p };
		myself.value = CapeValueContent::Empty;
		COBIAERR_NOERROR
	}

	/// Interface v-table

	const VTABLE: C::ICapeValue_VTable = C::ICapeValue_VTable {
		getValueType: Some(Self::get_value_type),
		getStringValue: Some(Self::get_string_value),
		getIntegerValue: Some(Self::get_integer_value),
		getBooleanValue: Some(Self::get_boolean_value),
		getRealValue: Some(Self::get_real_value),
		setStringValue: Some(Self::set_string_value),
		setIntegerValue: Some(Self::set_integer_value),
		setBooleanValue: Some(Self::set_boolean_value),
		setRealValue: Some(Self::set_real_value),
		clear: Some(Self::clear),
	};

}


impl CapeValueProviderIn for CapeValueImpl {
	/// Convert to ICapeValue
	///
	/// Returns a reference to the ICapeValue interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let val = cobia::CapeValueImpl::from_integer(2);
	///	let i_cape_value=val.as_cape_value_in();
	///	let mut i_cape_value_ptr=(&i_cape_value as *const C::ICapeValue).cast_mut(); //normally a pointer to the interface is received
	///	let v = cobia::CapeValueIn::new(&mut i_cape_value_ptr); //CapeValueIn from *mut C::ICapeValue
	/// assert_eq!(v.get_integer().unwrap(), 2);
	/// ```

	fn as_cape_value_in(&self) -> C::ICapeValue {
		C::ICapeValue {
			vTbl:(&Self::VTABLE as *const C::ICapeValue_VTable).cast_mut(),
			me:(self as *const Self).cast_mut() as *mut ::std::os::raw::c_void
		}
	}
}

impl CapeValueProviderOut for CapeValueImpl {
	/// Convert to ICapeValue
	///
	/// Returns a mutable reference to the ICapeValue interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut val = cobia::CapeValueImpl::new();
	///	let i_cape_value=val.as_cape_value_out();
	///	let mut i_cape_value_ptr=(&i_cape_value as *const C::ICapeValue).cast_mut(); //normally a pointer to the interface is received
	///	let mut v = cobia::CapeValueOut::new(&mut i_cape_value_ptr); //CapeValueOut from *mut C::ICapeValue
	/// v.set_real(2.5).unwrap();
	/// assert_eq!(val.value(), cobia::cape_value_impl::CapeValueContent::Real(2.5));
	/// ```

	fn as_cape_value_out(&mut self) -> C::ICapeValue {
		C::ICapeValue {
			vTbl:(&Self::VTABLE as *const C::ICapeValue_VTable).cast_mut(),
			me:(self as *const Self).cast_mut() as *mut ::std::os::raw::c_void
		}
	}
}

impl std::default::Default for CapeValueImpl {
	fn default() -> Self {
		Self::new()
	}
}

impl<T: CapeValueProviderIn> PartialEq<T> for CapeValueImpl {
	/// Partial equality
	///
	/// Checks if the content of the CapeValueVec is equal to the content of another object that implements CapeValueProviderIn.
	///
	/// # Arguments
	///
	/// * `other` - An object that implements CapeValueProviderIn
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let val1 = cobia::CapeValueImpl::from_integer(2);
	/// let val2 = cobia::CapeValueImpl::from_integer(2);
	/// let val3 = cobia::CapeValueImpl::from_boolean(true);
	/// assert!(val1 == val2);
	/// assert!(val1 != val3);
	/// ```
	fn eq(&self, other: &T) -> bool {
		let mut provider=CapeValueInFromProvider::from(other);
		let other=provider.as_cape_value_in(); 
		//compare the values
		match self.value {
			CapeValueContent::Empty => {
				if other.get_type().unwrap() != CapeValueType::Empty {
					return false;
				}
			},
			CapeValueContent::String(ref s) => {
				if other.get_type().unwrap() != CapeValueType::String || other.get_string().unwrap() != *s {
					return false;
				}
			},
			CapeValueContent::Integer(i) => {
				if other.get_type().unwrap() != CapeValueType::Integer || other.get_integer().unwrap() != i {
					return false;
				}
			},
			CapeValueContent::Boolean(b) => {
				if other.get_type().unwrap() != CapeValueType::Boolean || other.get_boolean().unwrap() != b {
					return false;
				}
			},
			CapeValueContent::Real(r) => {
				if other.get_type().unwrap() != CapeValueType::Real || other.get_real().unwrap() != r {
					return false;
				}
			},
		}
		true
	}
}