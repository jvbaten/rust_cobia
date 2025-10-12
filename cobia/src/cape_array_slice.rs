use crate::C;
use crate::*;

/// Slice based implementation of ICapeArray
///
/// Base class for several slice based ICapeArray implementations.
///
/// Any ICapeArray (e.g. ICapeArrayReal) is passed as data container
/// between CAPE-OPEN functions. It is up to the caller to provide the 
/// interface, and its implementation. This class provides a default
/// impementation, that uses a reference to the slice its data
///
/// These data types can only be used as input argument, not as output
/// argument, as the reference to the slice is non mutable.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayRealIn) {
///		assert_eq!(a.as_slice(), &[2.5,4.5]);
/// }
/// 
/// let arr = cobia::CapeArrayRealSlice::new(&[2.5,4.5]);
/// test_content(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in());
/// ```

#[allow(private_bounds)]
pub struct CapeArraySlice<'a,Element:Copy+Clone> {
	slice: &'a [Element],
}

#[allow(private_bounds)]
impl<'a,Element:Copy+Clone> CapeArraySlice<'a,Element> {

	/// New from slice
	///
	/// Creates a new CapeArraySlice from a slice.
	///
	/// # Arguments
	///
	/// * `slice` - A vector or slice or slice of values to be used as CapeArraySlice - values are referenced
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealSlice::new(&[2.5,4.5]);
	/// assert_eq!(arr.as_slice(), &vec![2.5,4.5]);
	/// ```

	pub fn new(slice: &'a [Element]) -> Self {
		CapeArraySlice {
			slice,
		}
	}

	/// Return a slice
	///
	/// Returns a reference to the slice of type T.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealSlice::new(&[2.5,4.5]);
	/// assert_eq!(arr.as_slice(), &vec![2.5,4.5]);
	/// ```

	pub fn as_slice(&self) -> &'a [Element] {
		&self.slice
	}

	/// Get size
	///
	/// Returns the size of the slice.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealSlice::new(&[2.5,4.5]);
	/// assert_eq!(arr.size(), 2);
	/// ```

	pub fn size(&self) -> usize {
		self.as_slice().len()	
	}

}

impl<'a,Element:Copy+Clone> AsRef<[Element]> for CapeArraySlice<'a,Element> {

	/// Return a reference to the slice
	///
	/// Returns a reference to the slice `[Element]`.
    fn as_ref(&self) -> &[Element] {
        self.slice
    }
}


/// Slice based CapeArrayRealIn implementation
///
/// ICapeArrayReal is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `&[CapeReal]`.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayRealIn) {
///		assert_eq!(a.as_slice(), &[2.5,4.5]);
/// }
/// 
/// let arr = cobia::CapeArrayRealSlice::new(&[2.5,4.5]);
/// test_content(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in());
/// ```
pub type CapeArrayRealSlice<'a> = CapeArraySlice<'a,CapeReal>;

impl<'a> CapeArrayRealSlice<'a> {

	/// Interface member function

	extern "C" fn get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut CapeReal,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = arr.slice.as_ptr() as *mut CapeReal;
			*size = arr.slice.len() as C::CapeSize;
		}
	}

	/// Interface member function

	extern "C" fn setsize(
		_: *mut ::std::os::raw::c_void,
		_: C::CapeSize,
		_: *mut *mut CapeReal,
	) -> CapeResult {
		COBIAERR_DENIED //this is an input argument, read only
	}

	/// Interface v-table

	const VTABLE: C::ICapeArrayReal_VTable = C::ICapeArrayReal_VTable {
		get: Some(Self::get),
		setsize: Some(Self::setsize),
	};

}

impl<'a> CapeArrayRealProviderIn for CapeArrayRealSlice<'a> {
	/// Convert to ICapeArrayReal
	///
	/// Returns a reference to the ICapeArrayReal interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealSlice::new(&[2.5,4.5]);
	///	let mut i_cape_array_real=arr.as_cape_array_real_in();
	///	let i_cape_array_real_ptr=(&i_cape_array_real as *const C::ICapeArrayReal).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayRealIn::new(&i_cape_array_real_ptr); //CapeArrayRealIn from *mut C::ICapeArrayReal
	/// assert_eq!(a.as_vec(), vec![2.5,4.5]);
	/// ```

	fn as_cape_array_real_in(&self) -> C::ICapeArrayReal {
		C::ICapeArrayReal {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayRealSlice::VTABLE as *const C::ICapeArrayReal_VTable).cast_mut(),
		}
	}
}

/// Slice based CapeArrayIntegerIn implementation
///
/// ICapeArrayInteger is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `&[CapeInteger]`
///
/// # Examples
///
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayIntegerIn) {
///		assert_eq!(a.as_vec(), vec![2,4]);
/// }
/// 
/// let arr = cobia::CapeArrayIntegerSlice::new(&[2,4]);
/// test_content(&CapeArrayIntegerInFromProvider::from(&arr).as_cape_array_integer_in());
/// ```
pub type CapeArrayIntegerSlice<'a> = CapeArraySlice<'a,CapeInteger>;

impl<'a> CapeArrayIntegerSlice<'a> {

	/// Interface member function

	extern "C" fn get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut CapeInteger,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = arr.slice.as_ptr() as *mut CapeInteger;
			*size = arr.slice.len() as C::CapeSize;
		}
	}

	/// Interface member function

	extern "C" fn setsize(
		_: *mut ::std::os::raw::c_void,
		_: C::CapeSize,
		_: *mut *mut CapeInteger,
	) -> CapeResult {
		COBIAERR_DENIED //this is an input argument, read only
	}

	/// Interface v-table

	const VTABLE: C::ICapeArrayInteger_VTable = C::ICapeArrayInteger_VTable {
		get: Some(Self::get),
		setsize: Some(Self::setsize),
	};

}

impl<'a> CapeArrayIntegerProviderIn for CapeArrayIntegerSlice<'a> {
	/// Convert to ICapeArrayInteger
	///
	/// Returns a reference to the ICapeArrayInteger interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayIntegerSlice::new(&[9,10,2]);
	///	let i_cape_array_integer=arr.as_cape_array_integer_in();
	///	let mut i_cape_array_integer_ptr=(&i_cape_array_integer as *const C::ICapeArrayInteger).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayIntegerIn::new(&mut i_cape_array_integer_ptr); //CapeArrayIntegerIn from *mut C::ICapeArrayInteger
	/// assert_eq!(a.as_vec(), vec![9,10,2]);
	/// ```

	fn as_cape_array_integer_in(&self) -> C::ICapeArrayInteger {
		C::ICapeArrayInteger {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayIntegerSlice::VTABLE as *const C::ICapeArrayInteger_VTable).cast_mut(),
		}
	}
}

/// Slice based CapeArrayByteIn implementation
///
/// ICapeArrayByte is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `&[CapeByte]`.
///
/// # Examples
///
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayByteIn) {
///		assert_eq!(a.as_vec(), vec![2u8,4u8]);
/// }
/// 
/// let arr = cobia::CapeArrayByteSlice::new(&[2u8,4u8]);
/// test_content(&CapeArrayByteInFromProvider::from(&arr).as_cape_array_byte_in());
/// ```
pub type CapeArrayByteSlice<'a> = CapeArraySlice<'a,CapeByte>;

impl<'a> CapeArrayByteSlice<'a> {

	/// Interface member function

	extern "C" fn get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut CapeByte,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = arr.slice.as_ptr() as *mut CapeByte;
			*size = arr.slice.len() as C::CapeSize;
		}
	}

	/// Interface member function

	extern "C" fn setsize(
		_: *mut ::std::os::raw::c_void,
		_: C::CapeSize,
		_: *mut *mut CapeByte,
	) -> C::CapeResult {
		COBIAERR_DENIED //this is an input argument, read only
	}

	/// Interface v-table

	const VTABLE: C::ICapeArrayByte_VTable = C::ICapeArrayByte_VTable {
		get: Some(Self::get),
		setsize: Some(Self::setsize),
	};

}

impl<'a> CapeArrayByteProviderIn for CapeArrayByteSlice<'a> {
	/// Convert to ICapeArrayByte
	///
	/// Returns a reference to the ICapeArrayByte interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayByteSlice::new(&[9u8,10u8,2u8]);
	///	let i_cape_array_byte=arr.as_cape_array_byte_in();
	///	let mut i_cape_array_byte_ptr=(&i_cape_array_byte as *const C::ICapeArrayByte).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayByteIn::new(&mut i_cape_array_byte_ptr); //CapeArrayByteIn from *mut C::ICapeArrayByte
	/// assert_eq!(a.as_vec(), vec![9u8,10u8,2u8]);
	/// ```

	fn as_cape_array_byte_in(&self) -> C::ICapeArrayByte {
		C::ICapeArrayByte {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayByteSlice::VTABLE as *const C::ICapeArrayByte_VTable).cast_mut(),
		}
	}
}

/// Slice based CapeArrayBooleanIn implementation
///
/// ICapeArrayBoolean is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `&[CapeBoolean]`.
///
/// # Examples
///
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayBooleanIn) {
///		assert_eq!(a.as_bool_vec(), vec![true,false]);
/// }
/// 
/// let arr = cobia::CapeArrayBooleanSlice::new(&[1,0]);
/// test_content(&CapeArrayBooleanInFromProvider::from(&arr).as_cape_array_boolean_in());
/// ```
pub type CapeArrayBooleanSlice<'a> = CapeArraySlice<'a,CapeBoolean>;

impl<'a> CapeArrayBooleanSlice<'a> {

	/// Interface member function

	extern "C" fn get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut CapeBoolean,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = arr.slice.as_ptr() as *mut CapeBoolean;
			*size = arr.slice.len() as C::CapeSize;
		}
	}

	/// Interface member function

	extern "C" fn setsize(
		_: *mut ::std::os::raw::c_void,
		_: C::CapeSize,
		_: *mut *mut CapeBoolean,
	) -> CapeResult {
		COBIAERR_DENIED //this is an input argument, read only
	}

	/// Interface v-table

	const VTABLE: C::ICapeArrayBoolean_VTable = C::ICapeArrayBoolean_VTable {
		get: Some(Self::get),
		setsize: Some(Self::setsize),
	};

}

impl<'a> CapeArrayBooleanProviderIn for CapeArrayBooleanSlice<'a> {
	/// Convert to ICapeArrayBoolean
	///
	/// Returns a reference to the ICapeArrayBoolean interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayBooleanSlice::new(&[0,1,0]);
	///	let i_cape_array_boolean=arr.as_cape_array_boolean_in();
	///	let mut i_cape_array_boolean_ptr=(&i_cape_array_boolean as *const C::ICapeArrayBoolean).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayBooleanIn::new(&mut i_cape_array_boolean_ptr); //CapeArrayBooleanIn from *mut C::ICapeArrayBoolean
	/// assert_eq!(a.as_bool_vec(), vec![false,true,false]);
	/// ```

	fn as_cape_array_boolean_in(&self) -> C::ICapeArrayBoolean {
		C::ICapeArrayBoolean {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayBooleanSlice::VTABLE as *const C::ICapeArrayBoolean_VTable).cast_mut(),
		}
	}
}

/// Slice based CapeArrayEnumerationIn implementation
///
/// ICapeArrayEnmeration is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `&[Element]`.
///
/// All CAPE-OPEN and COBIA enumeration types are represented as CapeEnumeration
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayEnumerationIn<CapePMCServiceType>) {
///		assert_eq!(a.as_vec(), vec![CapePMCServiceType::Inproc64,CapePMCServiceType::COM64]);
/// }
/// 
/// let arr = cobia::CapeArrayEnumerationSlice::new(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
/// test_content(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
/// ```

pub type CapeArrayEnumerationSlice<'a,Element> = CapeArraySlice<'a,Element>;

type CapeArrayEnumerationSliceRaw<'a> = CapeArraySlice<'a,C::CapeEnumeration>;

impl<'a> CapeArrayEnumerationSliceRaw<'a> {

	/// Interface member function

	extern "C" fn get_raw(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut C::CapeEnumeration,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = arr.slice.as_ptr() as *mut C::CapeEnumeration;
			*size = arr.slice.len() as C::CapeSize;
		}
	}

	/// Interface member function

	extern "C" fn setsize_raw(
		_: *mut ::std::os::raw::c_void,
		_: C::CapeSize,
		_: *mut *mut C::CapeEnumeration,
	) -> CapeResult {
		COBIAERR_DENIED //this is an input argument, read only
	}

	/// Interface v-table

	const VTABLE_RAW: C::ICapeArrayEnumeration_VTable = C::ICapeArrayEnumeration_VTable {
		get: Some(Self::get_raw),
		setsize: Some(Self::setsize_raw),
	};

}

impl<'a,Element:Copy+Clone> CapeArrayEnumerationProviderIn for CapeArrayEnumerationSlice<'a,Element> {
	/// Convert to ICapeArrayEnumeration
	///
	/// Returns a reference to the ICapeArrayEnumeration interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayEnumerationSlice::new(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	///	let i_cape_array_enumeration=arr.as_cape_array_enumeration_in();
	///	let mut i_cape_array_enumeration_ptr=(&i_cape_array_enumeration as *const C::ICapeArrayEnumeration).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayEnumerationIn::<cobia::CapePMCServiceType>::new(&mut i_cape_array_enumeration_ptr); //CapeArrayEnumerationIn from *mut C::ICapeArrayEnumeration
	/// assert_eq!(a.as_vec(), vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// ```

	fn as_cape_array_enumeration_in(&self) -> C::ICapeArrayEnumeration {
		C::ICapeArrayEnumeration {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayEnumerationSliceRaw::VTABLE_RAW as *const C::ICapeArrayEnumeration_VTable).cast_mut(),
		}
	}
}
