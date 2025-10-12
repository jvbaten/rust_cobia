use crate::C;
use crate::*;

/// Scalar based CapeArrayRealOut implementation
///
/// ICapeArrayReal is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a scalar CapeReal.
///
/// This class can be used as input and output argument. However, it should 
/// only be used as an output argument in case a scalar value is expected, such
/// as for getting the density from a material object, as any attempt to 
/// resize the content to anything else but 1 will result in an error.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_content(a: &mut CapeArrayRealOut) {
///		a.resize(1);
///		a[0]=2.5;
/// }
/// 
/// let mut arr = cobia::CapeArrayRealScalar::new();
/// set_content(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
/// assert_eq!(arr.value(), 2.5);
/// ```

#[derive (Clone)]
pub struct CapeArrayRealScalar {
	value: CapeReal,
}

impl CapeArrayRealScalar {

	/// Create a new CapeArrayRealScalar
	///
	/// Creates a new empty CapeArrayRealScalar
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let arr = cobia::CapeArrayRealScalar::new();
	/// assert!(cobia::CapeReal::is_nan(arr.value()));
	/// ```
	pub fn new() -> Self {
		Self {
			value: CapeReal::NAN,
		}
	}

	/// Create a new CapeArrayRealScalar with initial value
	///
	/// Creates a new empty CapeArrayRealScalar and sets the value
	///
	/// # Arguments
	///
	/// * `value` - The initial value
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let arr = cobia::CapeArrayRealScalar::from(3.14);
	/// assert_eq!(arr.value(),3.14);
	/// ```
	pub fn from(value:CapeReal) -> Self {
		Self {
			value,
		}
	}

	/// Return value
	///
	/// Returns the value.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// let arr = cobia::CapeArrayRealScalar::from(10.0);
	/// assert_eq!(arr.value(),10.0);
	/// ```

	pub fn value(&self) -> CapeReal {
		self.value
	}

	/// Set the value
	///
	/// Sets the value
	///
	/// # Arguments
	///
	/// * `value` - The value to be set
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// let mut arr = cobia::CapeArrayRealScalar::from(10.0);
	/// arr.set(11.0);
	/// assert_eq!(arr.value(),11.0);
	/// ```

	pub fn set(&mut self,value:CapeReal) {
		self.value=value;
	}

	/// Interface member function

	extern "C" fn get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut CapeReal,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = &mut arr.value as *mut CapeReal;
			*size = 1;
		}
	}

	/// Interface member function

	extern "C" fn setsize(
		me: *mut ::std::os::raw::c_void,
		size: C::CapeSize,
		data: *mut *mut CapeReal,
	) -> CapeResult {
		if size!=1 {
			COBIAERR_INVALIDARGUMENT //only a size of 1 is allowed
		} else {
			let p = me as *mut Self;
			let arr: &mut Self = unsafe { &mut *p };
			unsafe {
				*data = &mut arr.value as *mut CapeReal;
			}
			COBIAERR_NOERROR
		}
	}

	/// Interface v-table

	const VTABLE: C::ICapeArrayReal_VTable = C::ICapeArrayReal_VTable {
		get: Some(Self::get),
		setsize: Some(Self::setsize),
	};

}

impl AsMut<CapeReal> for CapeArrayRealScalar {

	/// Return mutable reference to the scalar value
	///
	/// Returns a mutable reference to the scalar value.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// let mut arr = cobia::CapeArrayRealScalar::from(10.0);
	/// *arr.as_mut()=11.0;
	/// assert_eq!(arr.value(),11.0);
	/// ```

	fn as_mut(&mut self) -> &mut CapeReal {
		&mut self.value
	}

}

impl CapeArrayRealProviderIn for CapeArrayRealScalar {
	/// Convert to ICapeArrayReal
	///
	/// Returns a reference to the ICapeArrayReal interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_content(a: &CapeArrayRealIn) {
	///     assert_eq!(a[0], 0.0);
	/// }
	/// 
	/// let arr = cobia::CapeArrayRealScalar::from(0.0);
	/// test_content(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in());
	/// ```

	fn as_cape_array_real_in(&self) -> C::ICapeArrayReal {
		C::ICapeArrayReal {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&Self::VTABLE as *const C::ICapeArrayReal_VTable).cast_mut()
		}
	}
}

impl CapeArrayRealProviderOut for CapeArrayRealScalar {
	/// Convert to ICapeArrayReal
	///
	/// Returns a mutable reference to the ICapeArrayReal interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_content(a: &mut CapeArrayRealOut) {
	///		a.resize(1);
	///		a[0]=2.5;
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealScalar::new();
	/// set_content(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
	/// assert_eq!(arr.value(), 2.5);
	/// ```

	fn as_cape_array_real_out(&mut self) -> C::ICapeArrayReal {
		C::ICapeArrayReal {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&Self::VTABLE as *const C::ICapeArrayReal_VTable).cast_mut()
		}
	}
}
