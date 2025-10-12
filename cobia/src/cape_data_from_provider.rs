use crate::*;

/// CapeStringInFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeString as input,
/// the caller provides an object that implements `CapeStringProviderIn`, 
/// for example `CapeStringImpl`.
///
/// The CapeStringInFromProvider returns an `C::ICapeString` interface, which
/// has a small life span, enough to make sure that the pointer to this 
/// interface is valid. This is done inside wrapper classes such as 
/// `capeopen_1_2::CapeIdentification`, 
///
/// When implementing a function that gets called, and takes a CapeString
/// as input, it received a `&CapeStringIn` typed argument, which is 
/// constructed from the reference to an `C::ICapeString` interface pointer.
///
/// Typically a function call receives the `C::ICapeString` interface 
/// from the caller, and from this, the `CapeStringIn` is constructed by 
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the 
/// `CapeStringProviderIn` trait, allocate the pointer, point to it, and 
/// construct the `CapeStringIn` object from a reference to that pointer.
///
/// The `CapeStringInFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let string = CapeStringImpl::from_string("Hello");
/// fn StringFromCapeStringIn(string:&CapeStringIn) -> String {
///     string.as_string()
/// }
/// let value=StringFromCapeStringIn(&CapeStringInFromProvider::from(&string).as_cape_string_in()); //this is how string is passed as &CapeStringIn argument
/// assert_eq!(value,"Hello".to_string());
/// ```

pub struct CapeStringInFromProvider {
	interface: C::ICapeString,
	interface_ptr: *mut C::ICapeString,
}

impl CapeStringInFromProvider {

	pub fn from<T:CapeStringProviderIn>(provider:&T) -> Self {
		Self {
			interface: provider.as_cape_string_in(),
			interface_ptr: std::ptr::null_mut(),
		}
	}

	pub fn as_cape_string_in(&mut self) -> CapeStringIn<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeString;
		CapeStringIn::new(& self.interface_ptr)
	}

}

/// CapeStringOutFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeString as output,
/// the caller provides an object that implements `CapeStringProviderOut`, 
/// for example `CapeStringImpl`.
///
/// The `CapeStringOutFromProvider` returns an `C::ICapeString` interface, which
/// has a small life span, enough to make sure that the pointer to this 
/// interface is valid. This is done inside wrapper classes such as 
/// `capeopen_1_2::CapeIdentification`, 
///
/// When implementing a function that gets called, and takes a CapeString
/// as output, it received a `&mut CapeStringOut` typed argument, which is 
/// constructed from the reference to an `C::ICapeString` interface pointer.
///
/// Typically a function call receives the C::ICapeString interface 
/// from the caller, and from this, the `CapeStringOut` is constructed by 
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the 
/// `CapeStringProviderOut` trait, allocate the pointer, point to it, and 
/// construct the `CapeStringOut` object from a reference to that pointer.
///
/// The `CapeStringOutFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let mut string = CapeStringImpl::new();
/// fn SetCapeStringOut(string:&CapeStringOut) {
///     string.set_string("Hello");
/// }
/// SetCapeStringOut(&mut CapeStringOutFromProvider::from(&mut string).as_cape_string_out()); //this is how string is passed as &mut CapeStringOut argument
/// assert_eq!(string.as_string(),"Hello".to_string());
/// ```

pub struct CapeStringOutFromProvider {
	interface: C::ICapeString,
	interface_ptr: *mut C::ICapeString,
}

impl CapeStringOutFromProvider {

	pub fn from<T:CapeStringProviderOut>(provider:&mut T) -> Self {
		Self {
			interface: provider.as_cape_string_out(),
			interface_ptr: std::ptr::null_mut(),
		}
	}

	pub fn as_cape_string_out(&mut self) -> CapeStringOut<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeString;
		CapeStringOut::new(& mut self.interface_ptr)
	}

}

/// CapeValueInFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeValue as input,
/// the caller provides an object that implements `CapeValueProviderIn`,
/// for example `CapeValueImpl`.
///
/// The `CapeValueInFromProvider` returns an `C::ICapeValue` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapePersistWriter`.
///
/// When implementing a function that gets called, and takes a CapeValue
/// as input, it received a `&CapeValueIn` typed argument, which is
/// constructed from the reference to an `C::ICapeValue` interface pointer.
///
/// Typically a function call receives the `C::ICapeValue` interface
/// from the caller, and from this, the `CapeValueIn` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeValueProviderIn` trait, allocate the pointer, point to it, and
/// construct the `CapeValueIn` object from a reference to that pointer.
///
/// The `CapeValueInFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let value = CapeValueImpl::from_real(3.14);
/// fn ValueFromCapeValueIn(value:&CapeValueIn) -> f64 {
///     assert!(value.get_type().unwrap()==cobia::CapeValueType::Real);
///     value.get_real().unwrap()
/// }
/// let value=ValueFromCapeValueIn(&CapeValueInFromProvider::from(&value).as_cape_value_in()); //this is how value is passed as &CapeValueIn argument
/// assert_eq!(value,3.14);
/// ```

pub struct CapeValueInFromProvider {
	interface: C::ICapeValue,
	interface_ptr: *mut C::ICapeValue,
}

impl CapeValueInFromProvider {
	pub fn from<T:CapeValueProviderIn>(provider:&T) -> Self {
		Self {
			interface: provider.as_cape_value_in(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_value_in(&mut self) -> CapeValueIn<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeValue;
		CapeValueIn::new(& self.interface_ptr)
	}
}

/// CapeValueOutFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeValue as output,
/// the caller provides an object that implements `CapeValueProviderOut`,
/// for example `CapeValueImpl`.
///
/// The `CapeValueOutFromProvider` returns an `C::ICapeValue` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapeCOSEUtilities`.
///
/// When implementing a function that gets called, and takes a CapeValue
/// as output, it received a `&mut CapeValueOut` typed argument, which is
/// constructed from the reference to an `C::ICapeValue` interface pointer.
///
/// Typically a function call receives the `C::ICapeValue` interface
/// from the caller, and from this, the `CapeValueOut` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeValueProviderOut` trait, allocate the pointer, point to it, and
/// construct the `CapeValueOut` object from a reference to that pointer.
///
/// The `CapeValueOutFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let mut value = CapeValueImpl::new();
/// fn SetValueOut(value:&CapeValueOut) {
///     value.set_integer(8);
/// }
/// SetValueOut(&mut CapeValueOutFromProvider::from(&mut value).as_cape_value_out()); //this is how value is passed as &mut CapeValueOut argument
/// assert_eq!(value.value(),CapeValueContent::Integer(8));
/// ```

pub struct CapeValueOutFromProvider {
	interface: C::ICapeValue,
	interface_ptr: *mut C::ICapeValue,
}

impl CapeValueOutFromProvider {
	pub fn from<T:CapeValueProviderOut>(provider:&mut T) -> Self {
		Self {
			interface: provider.as_cape_value_out(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_value_out(&mut self) -> CapeValueOut<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeValue;
		CapeValueOut::new(& mut self.interface_ptr)
	}
}

/// CapeArrayStringInFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayString as input,
/// the caller provides an object that implements `CapeArrayStringProviderIn`,
/// for example `CapeArrayStringVec`.
///
/// The `CapeArrayStringInFromProvider` returns an `C::ICapeArrayString` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapeThermoMaterial`.
///
/// When implementing a function that gets called, and takes a CapeArrayString
/// as input, it received a `&CapeArrayStringIn` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayString` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayString` interface
/// from the caller, and from this, the `CapeArrayStringIn` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayStringProviderIn` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayStringIn` object from a reference to that pointer.
///
/// The `CapeArrayStringInFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let array = CapeArrayStringVec::from_slice(&["Hello","World"]);
/// fn ArrayFromCapeArrayStringIn(array:&CapeArrayStringIn) -> Vec<String> {
///     array.as_string_vec()
/// }
/// let value=ArrayFromCapeArrayStringIn(&CapeArrayStringInFromProvider::from(&array).as_cape_array_string_in()); //this is how array is passed as &CapeArrayStringIn argument
/// assert_eq!(value,vec!["Hello".to_string(),"World".to_string()]);
/// ```

pub struct CapeArrayStringInFromProvider {
	interface: C::ICapeArrayString,
	interface_ptr: *mut C::ICapeArrayString,
}

impl CapeArrayStringInFromProvider {
	pub fn from<T:CapeArrayStringProviderIn>(provider:&T) -> Self {
		Self {
			interface: provider.as_cape_array_string_in(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_string_in(&mut self) -> CapeArrayStringIn<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayString;
		CapeArrayStringIn::new(& self.interface_ptr)
	}
}

/// CapeArrayStringOutFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayString as output,
/// the caller provides an object that implements `CapeArrayStringProviderOut`,
/// for example `CapeArrayStringVec`.
///
/// The `CapeArrayStringOutFromProvider` returns an `C::ICapeArrayString` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapeThermoMaterial`.
///
/// When implementing a function that gets called, and takes a CapeArrayString
/// as output, it received a `&mut CapeArrayStringOut` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayString` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayString` interface
/// from the caller, and from this, the `CapeArrayStringOut` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayStringProviderOut` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayStringOut` object from a reference to that pointer.
///
/// The `CapeArrayStringOutFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let mut array = CapeArrayStringVec::new();
/// fn SetArrayOut(array:&mut CapeArrayStringOut) {
///     array.put_array(&["Hello","World"]);
/// }
/// SetArrayOut(&mut CapeArrayStringOutFromProvider::from(&mut array).as_cape_array_string_out()); //this is how array is passed as &mut CapeArrayStringOut argument
/// assert_eq!(array.as_string_vec(),vec!["Hello".to_string(),"World".to_string()]);
/// ```

pub struct CapeArrayStringOutFromProvider {
	interface: C::ICapeArrayString,
	interface_ptr: *mut C::ICapeArrayString,
}

impl CapeArrayStringOutFromProvider {
	pub fn from<T:CapeArrayStringProviderOut>(provider:&mut T) -> Self {
		Self {
			interface: provider.as_cape_array_string_out(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_string_out(&mut self) -> CapeArrayStringOut<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayString;
		CapeArrayStringOut::new(& mut self.interface_ptr)
	}
}

/// CapeArrayIntegerInFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayInteger as input,
/// the caller provides an object that implements `CapeArrayIntegerProviderIn`,
/// for example `CapeArrayIntegerVec`.
///
/// The `CapeArrayIntegerInFromProvider` returns an `C::ICapeArrayInteger` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapeArrayRealParameter`.
///
/// When implementing a function that gets called, and takes a CapeArrayInteger
/// as input, it received a `&CapeArrayIntegerIn` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayInteger` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayInteger` interface
/// from the caller, and from this, the `CapeArrayIntegerIn` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayIntegerProviderIn` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayIntegerIn` object from a reference to that pointer.
///
/// The `CapeArrayIntegerInFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let array = CapeArrayIntegerVec::from_slice(&[1,2,3]);
/// fn ArrayFromCapeArrayIntegerIn(array:&CapeArrayIntegerIn) -> Vec<i32> {
///     array.as_vec()
/// }
/// let value=ArrayFromCapeArrayIntegerIn(&CapeArrayIntegerInFromProvider::from(&array).as_cape_array_integer_in()); //this is how array is passed as &CapeArrayIntegerIn argument
/// assert_eq!(value,vec![1,2,3]);
/// ```

pub struct CapeArrayIntegerInFromProvider {
	interface: C::ICapeArrayInteger,
	interface_ptr: *mut C::ICapeArrayInteger,
}

impl CapeArrayIntegerInFromProvider {
	pub fn from<T:CapeArrayIntegerProviderIn>(provider:&T) -> Self {
		Self {
			interface: provider.as_cape_array_integer_in(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_integer_in(&mut self) -> CapeArrayIntegerIn<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayInteger;
		CapeArrayIntegerIn::new(& self.interface_ptr)
	}
}

/// CapeArrayIntegerOutFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayInteger as output,
/// the caller provides an object that implements `CapeArrayIntegerProviderOut`,
/// for example `CapeArrayIntegerVec`.
///
/// The `CapeArrayIntegerOutFromProvider` returns an `C::ICapeArrayInteger` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapeArrayIntegerParameter`.
///
/// When implementing a function that gets called, and takes a CapeArrayInteger
/// as output, it received a `&mut CapeArrayIntegerOut` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayInteger` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayInteger` interface
/// from the caller, and from this, the `CapeArrayIntegerOut` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayIntegerProviderOut` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayIntegerOut` object from a reference to that pointer.
///
/// The `CapeArrayIntegerOutFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let mut array = CapeArrayIntegerVec::new();
/// fn SetArrayOut(array:&mut CapeArrayIntegerOut) {
///     array.put_array(&[1,2,3]);
/// }
/// SetArrayOut(&mut CapeArrayIntegerOutFromProvider::from(&mut array).as_cape_array_integer_out()); //this is how array is passed as &mut CapeArrayIntegerOut argument
/// assert_eq!(array.as_vec(),&vec![1,2,3]);
/// ```
	
pub struct CapeArrayIntegerOutFromProvider {
	interface: C::ICapeArrayInteger,
	interface_ptr: *mut C::ICapeArrayInteger,
}

impl CapeArrayIntegerOutFromProvider {
	pub fn from<T:CapeArrayIntegerProviderOut>(provider:&mut T) -> Self {
		Self {
			interface: provider.as_cape_array_integer_out(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_integer_out(&mut self) -> CapeArrayIntegerOut<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayInteger;
		CapeArrayIntegerOut::new(& mut self.interface_ptr)
	}
}

/// CapeArrayRealInFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayReal as input,
/// the caller provides an object that implements `CapeArrayRealProviderIn`,
/// for example `CapeArrayRealVec`.
///
/// The `CapeArrayRealInFromProvider` returns an `C::ICapeArrayReal` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapeThermoMaterial`.
///
/// When implementing a function that gets called, and takes a CapeArrayReal
/// as input, it received a `&CapeArrayRealIn` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayReal` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayReal` interface
/// from the caller, and from this, the `CapeArrayRealIn` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayRealProviderIn` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayRealIn` object from a reference to that pointer.
///
/// The `CapeArrayRealInFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let array = CapeArrayRealVec::from_slice(&[1.0,2.0,3.0]);
/// fn ArrayFromCapeArrayRealIn(array:&CapeArrayRealIn) -> Vec<f64> {
///     array.as_vec()
/// }
/// let value=ArrayFromCapeArrayRealIn(&CapeArrayRealInFromProvider::from(&array).as_cape_array_real_in()); //this is how array is passed as &CapeArrayRealIn argument
/// assert_eq!(value,vec![1.0,2.0,3.0]);
/// ```

pub struct CapeArrayRealInFromProvider {
	interface: C::ICapeArrayReal,
	interface_ptr: *mut C::ICapeArrayReal,
}

impl CapeArrayRealInFromProvider {
	pub fn from<T:CapeArrayRealProviderIn>(provider:&T) -> Self {
		Self {
			interface: provider.as_cape_array_real_in(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_real_in(&mut self) -> CapeArrayRealIn<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayReal;
		CapeArrayRealIn::new(& self.interface_ptr)
	}
}

/// CapeArrayRealOutFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayReal as output,
/// the caller provides an object that implements `CapeArrayRealProviderOut`,
/// for example `CapeArrayRealVec`.
///
/// The `CapeArrayRealOutFromProvider` returns an `C::ICapeArrayReal` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapeThermoMaterial`.
///
/// When implementing a function that gets called, and takes a CapeArrayReal
/// as output, it received a `&mut CapeArrayRealOut` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayReal` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayReal` interface
/// from the caller, and from this, the `CapeArrayRealOut` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayRealProviderOut` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayRealOut` object from a reference to that pointer.
///
/// The `CapeArrayRealOutFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let mut array = CapeArrayRealVec::new();
/// fn SetArrayOut(array:&mut CapeArrayRealOut) {
///     array.put_array(&[1.0,2.0,3.0]);
/// }
/// SetArrayOut(&mut CapeArrayRealOutFromProvider::from(&mut array).as_cape_array_real_out()); //this is how array is passed as &mut CapeArrayRealOut argument
/// assert_eq!(array.as_vec(),&vec![1.0,2.0,3.0]);
/// ```

pub struct CapeArrayRealOutFromProvider {
	interface: C::ICapeArrayReal,
	interface_ptr: *mut C::ICapeArrayReal,
}

impl CapeArrayRealOutFromProvider {
	pub fn from<T:CapeArrayRealProviderOut>(provider:&mut T) -> Self {
		Self {
			interface: provider.as_cape_array_real_out(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_real_out(&mut self) -> CapeArrayRealOut<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayReal;
		CapeArrayRealOut::new(& mut self.interface_ptr)
	}
}

/// CapeArrayBooleanInFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayBoolean as input,
/// the caller provides an object that implements `CapeArrayBooleanProviderIn`,
/// for example `CapeArrayBooleanVec`.
///
/// The `CapeArrayBooleanInFromProvider` returns an `C::ICapeArrayBoolean` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapeArrayBooleanParameter`.
///
/// When implementing a function that gets called, and takes a CapeArrayBoolean
/// as input, it received a `&CapeArrayBooleanIn` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayBoolean` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayBoolean` interface
/// from the caller, and from this, the `CapeArrayBooleanIn` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayBooleanProviderIn` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayBooleanIn` object from a reference to that pointer.
///
/// The `CapeArrayBooleanInFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let array = CapeArrayBooleanVec::from_slice(&[true as cobia::CapeBoolean,false as cobia::CapeBoolean,true as cobia::CapeBoolean]);
/// fn ArrayFromCapeArrayBooleanIn(array:&CapeArrayBooleanIn) -> Vec<cobia::CapeBoolean> {
///     array.as_vec()
/// }
/// let value=ArrayFromCapeArrayBooleanIn(&CapeArrayBooleanInFromProvider::from(&array).as_cape_array_boolean_in()); //this is how array is passed as &CapeArrayBooleanIn argument
/// assert_eq!(value,vec![true as cobia::CapeBoolean,false as cobia::CapeBoolean,true as cobia::CapeBoolean]);
/// ```

pub struct CapeArrayBooleanInFromProvider {
	interface: C::ICapeArrayBoolean,
	interface_ptr: *mut C::ICapeArrayBoolean,
}

impl CapeArrayBooleanInFromProvider {
	pub fn from<T:CapeArrayBooleanProviderIn>(provider:&T) -> Self {
		Self {
			interface: provider.as_cape_array_boolean_in(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_boolean_in(&mut self) -> CapeArrayBooleanIn<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayBoolean;
		CapeArrayBooleanIn::new(& self.interface_ptr)
	}
}

/// CapeArrayBooleanOutFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayBoolean as output,
/// the caller provides an object that implements `CapeArrayBooleanProviderOut`,
/// for example `CapeArrayBooleanVec`.
///
/// The `CapeArrayBooleanOutFromProvider` returns an `C::ICapeArrayBoolean` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapeArrayBooleanParameter`.
///
/// When implementing a function that gets called, and takes a CapeArrayBoolean
/// as output, it received a `&mut CapeArrayBooleanOut` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayBoolean` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayBoolean` interface
/// from the caller, and from this, the `CapeArrayBooleanOut` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayBooleanProviderOut` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayBooleanOut` object from a reference to that pointer.
///
/// The `CapeArrayBooleanOutFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let mut array = CapeArrayBooleanVec::new();
/// fn SetArrayOut(array:&mut CapeArrayBooleanOut) {
///     array.put_array(&[true as cobia::CapeBoolean,false as cobia::CapeBoolean,true as cobia::CapeBoolean]);
/// }
/// SetArrayOut(&mut CapeArrayBooleanOutFromProvider::from(&mut array).as_cape_array_boolean_out()); //this is how array is passed as &mut CapeArrayBooleanOut argument
/// assert_eq!(array.as_vec(),&vec![true as cobia::CapeBoolean,false as cobia::CapeBoolean,true as cobia::CapeBoolean]);
/// ```

pub struct CapeArrayBooleanOutFromProvider {
	interface: C::ICapeArrayBoolean,
	interface_ptr: *mut C::ICapeArrayBoolean,
}

impl CapeArrayBooleanOutFromProvider {
	pub fn from<T:CapeArrayBooleanProviderOut>(provider:&mut T) -> Self {
		Self {
			interface: provider.as_cape_array_boolean_out(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_boolean_out(&mut self) -> CapeArrayBooleanOut<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayBoolean;
		CapeArrayBooleanOut::new(& mut self.interface_ptr)
	}
}

/// CapeArrayByteInFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayByte as input,
/// the caller provides an object that implements `CapeArrayByteProviderIn`,
/// for example `CapeArrayByteVec`.
///
/// The `CapeArrayByteInFromProvider` returns an `C::ICapeArrayByte` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapePersistWriter`.
///
/// When implementing a function that gets called, and takes a CapeArrayByte
/// as input, it received a `&CapeArrayByteIn` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayByte` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayByte` interface
/// from the caller, and from this, the `CapeArrayByteIn` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayByteProviderIn` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayByteIn` object from a reference to that pointer.
///
/// The `CapeArrayByteInFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let array = CapeArrayByteVec::from_slice(&[1,2,3]);
/// fn ArrayFromCapeArrayByteIn(array:&CapeArrayByteIn) -> Vec<u8> {
///     array.as_vec()
/// }
/// let value=ArrayFromCapeArrayByteIn(&CapeArrayByteInFromProvider::from(&array).as_cape_array_byte_in()); //this is how array is passed as &CapeArrayByteIn argument
/// assert_eq!(value,vec![1,2,3]);
/// ```

pub struct CapeArrayByteInFromProvider {
	interface: C::ICapeArrayByte,
	interface_ptr: *mut C::ICapeArrayByte,
}

impl CapeArrayByteInFromProvider {
	pub fn from<T:CapeArrayByteProviderIn>(provider:&T) -> Self {
		Self {
			interface: provider.as_cape_array_byte_in(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_byte_in(&mut self) -> CapeArrayByteIn<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayByte;
		CapeArrayByteIn::new(& self.interface_ptr)
	}
}

/// CapeArrayByteOutFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayByte as output,
/// the caller provides an object that implements `CapeArrayByteProviderOut`,
/// for example `CapeArrayByteVec`.
///
/// The `CapeArrayByteOutFromProvider` returns an `C::ICapeArrayByte` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapePersistReader`.
///
/// When implementing a function that gets called, and takes a CapeArrayByte
/// as output, it received a `&mut CapeArrayByteOut` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayByte` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayByte` interface
/// from the caller, and from this, the `CapeArrayByteOut` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayByteProviderOut` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayByteOut` object from a reference to that pointer.
///
/// The `CapeArrayByteOutFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let mut array = CapeArrayByteVec::new();
/// fn SetArrayOut(array:&mut CapeArrayByteOut) {
///     array.put_array(&[1,2,3]);
/// }
/// SetArrayOut(&mut CapeArrayByteOutFromProvider::from(&mut array).as_cape_array_byte_out()); //this is how array is passed as &mut CapeArrayByteOut argument
/// assert_eq!(array.as_vec(),&vec![1,2,3]);
/// ```

pub struct CapeArrayByteOutFromProvider {
	interface: C::ICapeArrayByte,
	interface_ptr: *mut C::ICapeArrayByte,
}

impl CapeArrayByteOutFromProvider {
	pub fn from<T:CapeArrayByteProviderOut>(provider:&mut T) -> Self {
		Self {
			interface: provider.as_cape_array_byte_out(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_byte_out(&mut self) -> CapeArrayByteOut<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayByte;
		CapeArrayByteOut::new(& mut self.interface_ptr)
	}
}

/// CapeArrayValueInFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayValue as input,
/// the caller provides an object that implements `CapeArrayValueProviderIn`,
/// for example `CapeArrayValueVec`.
///
/// The `CapeArrayValueInFromProvider` returns an `C::ICapeArrayValue` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapePersistWriter`.
///
/// When implementing a function that gets called, and takes a CapeArrayValue
/// as input, it received a `&CapeArrayValueIn` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayValue` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayValue` interface
/// from the caller, and from this, the `CapeArrayValueIn` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayValueProviderIn` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayValueIn` object from a reference to that pointer.
///
/// The `CapeArrayValueInFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let array = CapeArrayValueVec::from_slice(&[CapeValueContent::Integer(1),CapeValueContent::Integer(2),CapeValueContent::Integer(3)]);
/// fn ArrayFromCapeArrayValueIn(array:&CapeArrayValueIn) -> Vec<CapeValueContent> {
///     array.as_value_vec().unwrap()
/// }
/// let value=ArrayFromCapeArrayValueIn(&CapeArrayValueInFromProvider::from(&array).as_cape_array_value_in()); //this is how array is passed as &CapeArrayValueIn argument
/// assert_eq!(value,vec![CapeValueContent::Integer(1),CapeValueContent::Integer(2),CapeValueContent::Integer(3)]);
/// ```

pub struct CapeArrayValueInFromProvider {
	interface: C::ICapeArrayValue,
	interface_ptr: *mut C::ICapeArrayValue,
}

impl CapeArrayValueInFromProvider {
	pub fn from<T:CapeArrayValueProviderIn>(provider:&T) -> Self {
		Self {
			interface: provider.as_cape_array_value_in(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_value_in(&mut self) -> CapeArrayValueIn<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayValue;
		CapeArrayValueIn::new(& self.interface_ptr)
	}
}

/// CapeArrayValueOutFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayValue as output,
/// the caller provides an object that implements `CapeArrayValueProviderOut`,
/// for example `CapeArrayValueVec`.
///
/// The `CapeArrayValueOutFromProvider` returns an `C::ICapeArrayValue` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapePersistReader`.
///
/// When implementing a function that gets called, and takes a CapeArrayValue
/// as output, it received a `&mut CapeArrayValueOut` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayValue` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayValue` interface
/// from the caller, and from this, the `CapeArrayValueOut` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayValueProviderOut` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayValueOut` object from a reference to that pointer.
///
/// The `CapeArrayValueOutFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let mut array = CapeArrayValueVec::new();
/// fn SetArrayOut(array:&mut CapeArrayValueOut) {
///     array.put_array(&[CapeValueContent::Integer(1),CapeValueContent::Integer(2),CapeValueContent::Integer(3)]);
/// }
/// SetArrayOut(&mut CapeArrayValueOutFromProvider::from(&mut array).as_cape_array_value_out()); //this is how array is passed as &mut CapeArrayValueOut argument
/// assert_eq!(array.as_value_vec(),vec![CapeValueContent::Integer(1),CapeValueContent::Integer(2),CapeValueContent::Integer(3)]);
/// ```

pub struct CapeArrayValueOutFromProvider {
	interface: C::ICapeArrayValue,
	interface_ptr: *mut C::ICapeArrayValue,
}

impl CapeArrayValueOutFromProvider {
	pub fn from<T:CapeArrayValueProviderOut>(provider:&mut T) -> Self {
		Self {
			interface: provider.as_cape_array_value_out(),
			interface_ptr: std::ptr::null_mut(),
		}
	}
	pub fn as_cape_array_value_out(&mut self) -> CapeArrayValueOut<'_> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayValue;
		CapeArrayValueOut::new(& mut self.interface_ptr)
	}
}

/// CapeArrayEnumerationInFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayEnumeration as input,
/// the caller provides an object that implements `CapeArrayEnumerationProviderIn`,
/// for example `CapeArrayEnumerationVec`.
///
/// The `CapeArrayEnumerationInFromProvider` returns an `C::ICapeArrayEnumeration` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapeArrayRealParameter`.
///
/// When implementing a function that gets called, and takes a CapeArrayEnumeration
/// as input, it received a `&CapeArrayEnumerationIn` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayEnumeration` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayEnumeration` interface
/// from the caller, and from this, the `CapeArrayEnumerationIn` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayEnumerationProviderIn` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayEnumerationIn` object from a reference to that pointer.
///
/// The `CapeArrayEnumerationInFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let array = cobia::CapeArrayEnumerationVec::from_slice(&[CapePMCServiceType::Inproc64,CapePMCServiceType::COM64]);
/// fn ArrayFromCapeArrayEnumerationIn(array:&CapeArrayEnumerationIn<CapePMCServiceType>) -> Vec<CapePMCServiceType> {
///     array.as_vec()
/// }
/// let value=ArrayFromCapeArrayEnumerationIn(&CapeArrayEnumerationInFromProvider::from(&array).as_cape_array_enumeration_in()); //this is how array is passed as &CapeArrayEnumerationIn argument
/// assert_eq!(value,vec![CapePMCServiceType::Inproc64,CapePMCServiceType::COM64]);
/// ```

pub struct CapeArrayEnumerationInFromProvider<Element:Copy+Clone> {
	interface: C::ICapeArrayEnumeration,
	interface_ptr: *mut C::ICapeArrayEnumeration,
	element_type : std::marker::PhantomData<Element>,
}

impl<Element:Copy+Clone> CapeArrayEnumerationInFromProvider<Element> {
	pub fn from<T:CapeArrayEnumerationProviderIn>(provider:&T) -> Self {
		Self {
			interface: provider.as_cape_array_enumeration_in(),
			interface_ptr: std::ptr::null_mut(),
			element_type:std::default::Default::default(),
		}
	}
	pub fn as_cape_array_enumeration_in(&mut self) -> CapeArrayEnumerationIn<'_,Element> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayEnumeration;
		CapeArrayEnumerationIn::new(& self.interface_ptr)
	}
}

/// CapeArrayEnumerationOutFromProvider
///
/// When calling a CAPE-OPEN method that takes a CapeArrayEnumeration as output,
/// the caller provides an object that implements `CapeArrayEnumerationProviderOut`,
/// for example `CapeArrayEnumerationVec`.
///
/// The `CapeArrayEnumerationOutFromProvider` returns an `C::ICapeArrayEnumeration` interface, which
/// has a small life span, enough to make sure that the pointer to this
/// interface is valid. This is done inside wrapper classes such as
/// `capeopen_1_2::CapeArrayEnumerationParameter`.
///
/// When implementing a function that gets called, and takes a CapeArrayEnumeration
/// as output, it received a `&mut CapeArrayEnumerationOut` typed argument, which is
/// constructed from the reference to an `C::ICapeArrayEnumeration` interface pointer.
///
/// Typically a function call receives the `C::ICapeArrayEnumeration` interface
/// from the caller, and from this, the `CapeArrayEnumerationOut` is constructed by
/// the `cape_object_implementation` macro.
///
/// In the rare case that one wants to call an internal CAPE-OPEN function
/// directly, one needs to provide the class that implements the
/// `CapeArrayEnumerationProviderOut` trait, allocate the pointer, point to it, and
/// construct the `CapeArrayEnumerationOut` object from a reference to that pointer.
///
/// The `CapeArrayEnumerationOutFromProvider` class does all this.
///
/// # Example
/// ```
/// use cobia::*;
/// let mut array : CapeArrayEnumerationVec<CapePMCServiceType> = CapeArrayEnumerationVec::new();
/// fn SetArrayOut(array:&mut CapeArrayEnumerationOut<CapePMCServiceType>) {
///     array.put_array(&[CapePMCServiceType::Inproc64,CapePMCServiceType::COM64]);
/// }
/// SetArrayOut(&mut CapeArrayEnumerationOutFromProvider::from(&mut array).as_cape_array_enumeration_out()); //this is how array is passed as &mut CapeArrayEnumerationOut argument
/// assert_eq!(array.as_vec(),&vec![CapePMCServiceType::Inproc64,CapePMCServiceType::COM64]);
/// ```
	
pub struct CapeArrayEnumerationOutFromProvider<Element:Copy+Clone> {
	interface: C::ICapeArrayEnumeration,
	interface_ptr: *mut C::ICapeArrayEnumeration,
	element_type : std::marker::PhantomData<Element>,
}

impl<Element:Copy+Clone> CapeArrayEnumerationOutFromProvider<Element> {
	pub fn from<T:CapeArrayEnumerationProviderOut>(provider:&mut T) -> Self {
		Self {
			interface: provider.as_cape_array_enumeration_out(),
			interface_ptr: std::ptr::null_mut(),
			element_type:std::default::Default::default(),
		}
	}
	pub fn as_cape_array_enumeration_out(&mut self) -> CapeArrayEnumerationOut<'_,Element> {
		self.interface_ptr=&mut self.interface as *mut C::ICapeArrayEnumeration;
		CapeArrayEnumerationOut::new(& mut self.interface_ptr)
	}
}