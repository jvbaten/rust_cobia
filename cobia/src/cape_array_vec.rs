use crate::C;
use crate::*;
use std::mem;

/// Vector based implementation of ICapeArray
///
/// Base class for several vector based ICapeArray implementations.
///
/// Any ICapeArray (e.g. ICapeArrayReal) is passed as data container
/// between CAPE-OPEN functions. It is up to the caller to provide the 
/// interface, and its implementation. This class provides a default
/// impementation, that uses std::vec::Vec as the data container.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_content(arr: &mut CapeArrayRealOut) {
///		arr.put_array(&[4.5,6.5]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[4.5,6.5]);
/// set_content(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
/// assert_eq!(arr.as_vec(), &vec![4.5,6.5]);
/// ```

#[allow(private_bounds)]
#[derive (Clone)]
pub struct CapeArrayVec<Element:Copy+Clone,InterfaceType> {
	vec: Vec<Element>,
	interface_type: std::marker::PhantomData<InterfaceType>,
}

#[allow(private_bounds)]
impl<Element:Copy+Clone,InterfaceType> CapeArrayVec<Element,InterfaceType> {

	/// Create a new CapeArrayVec
	///
	/// Creates a new empty CapeArrayVec
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealVec::new();
	/// assert_eq!(arr.as_vec().len(), 0);
	/// ```
	pub fn new() -> Self {
		Self {
			vec: Vec::new(),
			interface_type: std::default::Default::default(),
		}
	}

	/// Return a vector
	///
	/// Returns a reference to the vector of type T.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealVec::from_slice(&[2.5,4.5]);
	/// assert_eq!(arr.as_vec(), &vec![2.5,4.5]);
	/// ```

	pub fn as_vec(&self) -> &Vec<Element> {
		&self.vec
	}

	/// Return a mutable vector
	///
	/// Returns a mutable reference to the vector of type T.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayRealVec::from_vec(vec![2.5,4.5]);
	/// arr.as_mut_vec().push(6.5);
	/// assert_eq!(arr.as_vec(), &vec![2.5,4.5,6.5]);
	/// ```

	pub fn as_mut_vec(&mut self) -> &mut Vec<Element> {
		&mut self.vec
	}

	/// Get size
	///
	/// Returns the size of the array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealVec::from_slice(&[2.5,4.5]);
	/// assert_eq!(arr.size(), 2);
	/// ```

	pub fn size(&self) -> usize {
		self.as_vec().len()	
	}

	/// Initialize from slice
	///
	/// Creates a new CapeArrayVec from a slice.
	///
	/// # Arguments
	///
	/// * `slice` - A vector or array or slice of values to be converted to a CapeArrayVec - values are copied
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealVec::from_slice(&[2.5,4.5]);
	/// assert_eq!(arr.as_vec(), &vec![2.5,4.5]);
	/// ```

	pub fn from_slice(slice: &[Element]) -> Self {
		let mut a = Self::new();
		a.as_mut_vec().extend_from_slice(slice);
		a
	}

	/// Check if empty
	///
	/// Returns true if the array is empty.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealVec::new();
	/// assert!(arr.is_empty());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.as_vec().is_empty()
	}

	/// Initialize from vector
	///
	/// Creates a new CapeArrayVec from a vector, taking ownwership of the data
	///
	/// # Arguments
	///
	/// * `vec` - A vector slice of values to be converted to a CapeArrayVec
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealVec::from_slice(&[2.5,4.5]);
	/// assert_eq!(arr.as_vec(), &vec![2.5,4.5]);
	/// ```

	pub fn from_vec(mut vec: std::vec::Vec<Element>) -> Self {
		let mut a = Self::new();
		mem::swap(a.as_mut_vec(), &mut vec);
		a
	}

}

impl<Element:Copy+Clone,InterfacType> Default for CapeArrayVec<Element,InterfacType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Element:Copy+Clone,InterfacType> std::ops::Index<usize> for CapeArrayVec<Element,InterfacType> {

	type Output = Element;

	/// Indexing
	///
	/// Returns a reference to the element at the given index.
	///
	/// # Arguments
	///
	/// * `index` - The index of the element to be returned
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealVec::from_slice(&[10.1,10.2,10.3]);
	/// assert_eq!(arr[1], 10.2);
	/// ```

	fn index(&self, index: usize) -> &Self::Output {
		&self.vec[index]
	}

}

impl<Element:Copy+Clone,Interface> std::ops::IndexMut<usize> for CapeArrayVec<Element,Interface> {
	/// Indexing
	///
	/// Returns a mutable reference to the element at the given index.
	///
	/// # Arguments
	///
	/// * `index` - The index of the element to be returned
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[10.1,10.2,10.3]);
	/// arr[1]=2.2;
	/// assert_eq!(arr[1], 2.2);
	/// ```

	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.vec[index]
	}
}


impl<Element:Copy+Clone,InterfacType> AsRef<[Element]> for CapeArrayVec<Element,InterfacType> {
    fn as_ref(&self) -> &[Element] {
        self.vec.as_ref()
    }
}

/// Vector based CapeArrayRealOut implementation
///
/// ICapeArrayReal is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `Vec<C::CapeReal>`.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_content(arr: &mut CapeArrayRealOut) {
///		arr.put_array(&[4.5,6.5]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[4.5,6.5]);
/// set_content(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
/// assert_eq!(arr.as_vec(), &vec![4.5,6.5]);
/// ```
pub type CapeArrayRealVec = CapeArrayVec<CapeReal,C::ICapeArrayReal>;

impl CapeArrayRealVec {

	/// Interface member function

	extern "C" fn get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut CapeReal,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = arr.vec.as_ptr() as *mut CapeReal;
			*size = arr.vec.len() as C::CapeSize;
		}
	}
	
	/// Resize
	///
	/// Resize the array to the given size. If the size is larger than the current size, the new elements are set to `CapeReal::NAN`.
	///
	/// # Arguments
	///
	/// * `size` - The new size of the array
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[2.5,4.5]);
	/// arr.resize(4);
	/// assert_eq!(arr.size(), 4);
	/// assert!(arr[2].is_nan());
	/// ```
	pub fn resize(&mut self, size: usize) {
		self.vec.resize(size, CapeReal::NAN);
	}
	
	/// Set the content of the real array from any object that implements CapeArrayRealProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayRealProviderIn
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// let mut arr = cobia::CapeArrayRealVec::new();
	/// let arr1 = cobia::CapeArrayRealVec::from_slice(&[2.5,4.5]);
	/// arr.set(&arr1);
	/// assert_eq!(arr.as_vec(), &vec![2.5,4.5]);
	/// ```
	pub fn set<T:CapeArrayRealProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut real_array_in_from_provider = CapeArrayRealInFromProvider::from(array);
		let real_array=real_array_in_from_provider.as_cape_array_real_in();
		self.vec.clear();
		self.vec.extend_from_slice(real_array.as_slice());
		Ok(())
	}

	/// Interface member function

	extern "C" fn setsize(
		me: *mut ::std::os::raw::c_void,
		size: C::CapeSize,
		data: *mut *mut CapeReal,
	) -> CapeResult {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		arr.resize(size as usize);
		unsafe {
			*data = arr.vec.as_ptr() as *mut CapeReal;
		}
		COBIAERR_NOERROR
	}

	/// Interface v-table

	const VTABLE: C::ICapeArrayReal_VTable = C::ICapeArrayReal_VTable {
		get: Some(Self::get),
		setsize: Some(Self::setsize),
	};

}

impl CapeArrayRealProviderIn for CapeArrayRealVec {
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
	///     assert_eq!(a.as_vec(), vec![2.5,4.5]);
	/// }
	/// 
	/// let arr = cobia::CapeArrayRealVec::from_slice(&[2.5,4.5]);
	/// test_content(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in());
	/// ```

	fn as_cape_array_real_in(&self) -> C::ICapeArrayReal {
		C::ICapeArrayReal {
			me:(self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayRealVec::VTABLE as *const C::ICapeArrayReal_VTable).cast_mut(),
		}
	}
}

impl CapeArrayRealProviderOut for CapeArrayRealVec {
	/// Convert to ICapeArrayReal
	///
	/// Returns a mutable reference to the ICapeArrayReal interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[2.5,4.5]);
	///	let i_cape_array_real=arr.as_cape_array_real_out();
	///	let mut i_cape_array_real_ptr=(&i_cape_array_real as *const C::ICapeArrayReal).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayRealOut::new(&mut i_cape_array_real_ptr); //CapeArrayRealOut from *mut C::ICapeArrayReal
	/// assert_eq!(a.as_vec(), vec![2.5,4.5]);
	/// ```

	fn as_cape_array_real_out(&mut self) -> C::ICapeArrayReal {
		C::ICapeArrayReal {
			me:(self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayRealVec::VTABLE as *const C::ICapeArrayReal_VTable).cast_mut(),
		}
	}
}

impl<T:CapeArrayRealProviderIn> PartialEq<T> for CapeArrayRealVec {
	/// Partial equality
	///
	/// Checks if the CapeArrayRealVec is equal to another CapeArrayRealProviderIn.
	///
	/// # Arguments
	///
	/// * `other` - The other CapeArrayRealProviderIn to compare with
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr1 = cobia::CapeArrayRealVec::from_slice(&[2.5,4.5]);
	/// let arr2 = cobia::CapeArrayRealVec::from_slice(&[2.5,4.5]);
	/// let arr3 = cobia::CapeArrayRealVec::from_slice(&[2.0,4.5]);
	/// assert!(arr1 == arr2);
	/// assert!(arr1 != arr3);
	/// ```
	fn eq(&self, other: &T) -> bool {
		let mut provider=CapeArrayRealInFromProvider::from(other);
		let other=provider.as_cape_array_real_in(); 
		self.as_vec() == other.as_slice()
	}
}

/// Vector based CapeArrayIntegerOut implementation
///
/// ICapeArrayInteger is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `Vec<C::CapeInteger>`.
///
/// # Examples
///
///
/// ```
/// use cobia::*;
///
/// fn set_content(a: &mut CapeArrayIntegerOut) {
///     a.put_array(&[2,8,10]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayIntegerVec::new();
/// set_content(&mut CapeArrayIntegerOutFromProvider::from(&mut arr).as_cape_array_integer_out());
/// assert_eq!(arr.as_vec(), &vec![2,8,10]);
/// ```
pub type CapeArrayIntegerVec = CapeArrayVec<CapeInteger,C::ICapeArrayInteger>;

impl CapeArrayIntegerVec {

	/// Interface member function

	extern "C" fn get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut CapeInteger,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = arr.vec.as_ptr() as *mut CapeInteger;
			*size = arr.vec.len() as C::CapeSize;
		}
	}

	/// Interface member function

	extern "C" fn setsize(
		me: *mut ::std::os::raw::c_void,
		size: C::CapeSize,
		data: *mut *mut CapeInteger,
	) -> CapeResult {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		arr.resize(size as usize);
		unsafe {
			*data = arr.vec.as_ptr() as *mut CapeInteger;
		}
		COBIAERR_NOERROR
	}

	/// Resize
	///
	/// Resize the array to the given size. If the size is larger than the current size, the new elements are set to `0`.
	///
	/// # Arguments
	///
	/// * `size` - The new size of the array
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayIntegerVec::from_slice(&[2,4]);
	/// arr.resize(4);
	/// assert_eq!(arr.as_vec(), &vec![2 as cobia::CapeInteger,4 as cobia::CapeInteger,0 as cobia::CapeInteger,0 as cobia::CapeInteger]);
	/// ```
	pub fn resize(&mut self, size: usize) {
		self.vec.resize(size, 0);
	}

	/// Set the content of the integer array from any object that implements CapeArrayIntegerProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayIntegerProviderIn
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// let mut arr = cobia::CapeArrayIntegerVec::new();
	/// let arr1 = cobia::CapeArrayIntegerVec::from_slice(&[8,9,7]);
	/// arr.set(&arr1);
	/// assert_eq!(arr.as_vec(), &vec![8,9,7]);
	/// ```
	pub fn set<T:CapeArrayIntegerProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut integer_array_in_from_provider = CapeArrayIntegerInFromProvider::from(array);
		let integer_array=integer_array_in_from_provider.as_cape_array_integer_in();
		self.vec.clear();
		self.vec.extend_from_slice(integer_array.as_slice());
		Ok(())
	}

	/// Interface v-table

	const VTABLE: C::ICapeArrayInteger_VTable = C::ICapeArrayInteger_VTable {
		get: Some(Self::get),
		setsize: Some(Self::setsize),
	};

}

impl CapeArrayIntegerProviderIn for CapeArrayIntegerVec {
	/// Convert to ICapeArrayInteger
	///
	/// Returns a reference to the ICapeArrayInteger interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayIntegerVec::from_slice(&[9,10,2]);
	///	let i_cape_array_integer=arr.as_cape_array_integer_out();
	///	let mut i_cape_array_integer_ptr=(&i_cape_array_integer as *const C::ICapeArrayInteger).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayIntegerOut::new(&mut i_cape_array_integer_ptr); //CapeArrayIntegerOut from *mut C::ICapeArrayInteger
	/// assert_eq!(a.as_vec(), vec![9,10,2]);
	/// ```

	fn as_cape_array_integer_in(&self) -> C::ICapeArrayInteger {
		C::ICapeArrayInteger {
			vTbl: (&CapeArrayIntegerVec::VTABLE as *const C::ICapeArrayInteger_VTable).cast_mut(),
			me:(self as *const Self).cast_mut() as *mut ::std::os::raw::c_void
		}
	}
}


impl CapeArrayIntegerProviderOut for CapeArrayIntegerVec {
	/// Convert to ICapeArrayInteger
	///
	/// Returns a mutable reference to the ICapeArrayInteger interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayIntegerVec::from_slice(&[9,10,2]);
	///	let i_cape_array_integer=arr.as_cape_array_integer_out();
	///	let mut i_cape_array_integer_ptr=(&i_cape_array_integer as *const C::ICapeArrayInteger).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayIntegerOut::new(&mut i_cape_array_integer_ptr); //CapeArrayIntegerOut from *mut C::ICapeArrayInteger
	/// assert_eq!(a.as_vec(), vec![9,10,2]);
	/// ```

	fn as_cape_array_integer_out(&mut self) -> C::ICapeArrayInteger {
		C::ICapeArrayInteger {
			vTbl: (&CapeArrayIntegerVec::VTABLE as *const C::ICapeArrayInteger_VTable).cast_mut(),
			me:(self as *const Self).cast_mut() as *mut ::std::os::raw::c_void
		}
	}
}

impl<T:CapeArrayIntegerProviderIn> PartialEq<T> for CapeArrayIntegerVec {
	/// Partial equality
	///
	/// Checks if the CapeArrayIntegerVec is equal to another CapeArrayIntegerProviderIn.
	///
	/// # Arguments
	///
	/// * `other` - The other CapeArrayIntegerProviderIn to compare with
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr1 = cobia::CapeArrayIntegerVec::from_slice(&[2,4]);
	/// let arr2 = cobia::CapeArrayIntegerVec::from_slice(&[2,4]);
	/// let arr3 = cobia::CapeArrayIntegerVec::from_slice(&[2,3,4]);
	/// assert!(arr1 == arr2);
	/// assert!(arr1 != arr3);
	/// ```
	fn eq(&self, other: &T) -> bool {
		let mut provider=CapeArrayIntegerInFromProvider::from(other);
		let other=provider.as_cape_array_integer_in(); 
		self.as_vec() == other.as_slice()
	}
}

/// Vector based CapeArrayByteOut implementation
///
/// ICapeArrayByte is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `Vec<C::CapeByte>`.
///
/// # Examples
///
///
/// ```
/// use cobia::*;
///
/// fn set_content(a: &mut CapeArrayByteOut) {
///		a.put_array(&[2u8,4u8]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayByteVec::from_slice(&[2u8,4u8]);
/// set_content(&mut CapeArrayByteOutFromProvider::from(&mut arr).as_cape_array_byte_out());
/// assert_eq!(arr.as_vec(), &vec![2u8,4u8]);
/// ```
pub type CapeArrayByteVec = CapeArrayVec<CapeByte,C::ICapeArrayByte>;

impl CapeArrayByteVec {

	/// Interface member function

	extern "C" fn get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut CapeByte,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = arr.vec.as_ptr() as *mut CapeByte;
			*size = arr.vec.len() as C::CapeSize;
		}
	}

	/// Interface member function

	extern "C" fn setsize(
		me: *mut ::std::os::raw::c_void,
		size: C::CapeSize,
		data: *mut *mut CapeByte,
	) -> C::CapeResult {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		arr.resize(size as usize);
		unsafe {
			*data = arr.vec.as_ptr() as *mut CapeByte;
		}
		COBIAERR_NOERROR
	}

	/// Resize
	///
	/// Resize the array to the given size. If the size is larger than the current size, the new elements are set to `0`.
	///
	/// # Arguments
	///
	/// * `size` - The new size of the array
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayByteVec::from_slice(&[2u8,4u8]);
	/// arr.resize(4);
	/// assert_eq!(arr.as_vec(), &vec![2u8,4u8,0,0]);
	/// ```
	pub fn resize(&mut self, size: usize) {
		self.vec.resize(size, 0);
	}

	/// Set the content of the byte array from any object that implements CapeArrayByteProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayByteProviderIn
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// let mut arr = cobia::CapeArrayByteVec::new();
	/// let arr1 = cobia::CapeArrayByteVec::from_slice(&[8,9,7]);
	/// arr.set(&arr1);
	/// assert_eq!(arr.as_vec(), &vec![8,9,7]);
	/// ```
	pub fn set<T:CapeArrayByteProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut byte_array_in_from_provider = CapeArrayByteInFromProvider::from(array);
		let byte_array=byte_array_in_from_provider.as_cape_array_byte_in();
		self.vec.clear();
		self.vec.extend_from_slice(byte_array.as_slice());
		Ok(())
	}

	/// Interface v-table

	const VTABLE: C::ICapeArrayByte_VTable = C::ICapeArrayByte_VTable {
		get: Some(Self::get),
		setsize: Some(Self::setsize),
	};

}

impl CapeArrayByteProviderIn for CapeArrayByteVec {
	/// Convert to ICapeArrayByte
	///
	/// Returns a reference to the ICapeArrayByte interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayByteVec::from_slice(&[9u8,10u8,2u8]);
	///	let i_cape_array_byte=arr.as_cape_array_byte_in();
	///	let mut i_cape_array_byte_ptr=(&i_cape_array_byte as *const C::ICapeArrayByte).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayByteIn::new(&mut i_cape_array_byte_ptr); //CapeArrayByteIn from *mut C::ICapeArrayByte
	/// assert_eq!(a.as_vec(), vec![9u8,10u8,2u8]);
	/// ```

	fn as_cape_array_byte_in(&self) -> C::ICapeArrayByte {
		C::ICapeArrayByte {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayByteVec::VTABLE as *const C::ICapeArrayByte_VTable).cast_mut(),
		}
	}
}

impl CapeArrayByteProviderOut for CapeArrayByteVec {
	/// Convert to ICapeArrayByte
	///
	/// Returns a mutable reference to the ICapeArrayByte interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayByteVec::from_slice(&[9u8,10u8,2u8]);
	///	let i_cape_array_byte=arr.as_cape_array_byte_out();
	///	let mut i_cape_array_byte_ptr=(&i_cape_array_byte as *const C::ICapeArrayByte).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayByteOut::new(&mut i_cape_array_byte_ptr); //CapeArrayByteOut from *mut C::ICapeArrayByte
	/// assert_eq!(a.as_vec(), vec![9u8,10u8,2u8]);
	/// ```

	fn as_cape_array_byte_out(&mut self) -> C::ICapeArrayByte {
		C::ICapeArrayByte {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayByteVec::VTABLE as *const C::ICapeArrayByte_VTable).cast_mut(),
		}
	}
}

impl<T:CapeArrayByteProviderIn> PartialEq<T> for CapeArrayByteVec {
	/// Partial equality
	///
	/// Checks if the CapeArrayByteVec is equal to another CapeArrayByteProviderIn.
	///
	/// # Arguments
	///
	/// * `other` - The other CapeArrayByteProviderIn to compare with
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr1 = cobia::CapeArrayByteVec::from_slice(&[2u8,4u8]);
	/// let arr2 = cobia::CapeArrayByteVec::from_slice(&[2u8,4u8]);
	/// let arr3 = cobia::CapeArrayByteVec::from_slice(&[2u8,3u8,4u8]);
	/// assert!(arr1 == arr2);
	/// assert!(arr1 != arr3);
	/// ```
	fn eq(&self, other: &T) -> bool {
		let mut provider=CapeArrayByteInFromProvider::from(other);
		let other=provider.as_cape_array_byte_in(); 
		self.as_vec() == other.as_slice()
	}
}

/// Vector based CapeArrayBooleanOut implementation
///
/// ICapeArrayBoolean is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `Vec<C::CapeBoolean>`.
///
/// # Examples
///
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayBooleanIn) {
///     assert_eq!(a.as_bool_vec(), vec![true,false]);
/// }
/// 
/// let arr = cobia::CapeArrayBooleanVec::from_bool_slice(&[true,false]);
/// test_content(&CapeArrayBooleanInFromProvider::from(&arr).as_cape_array_boolean_in())
/// ```
pub type CapeArrayBooleanVec = CapeArrayVec<CapeBoolean,C::ICapeArrayBoolean>;

impl CapeArrayBooleanVec {

	/// Interface member function

	extern "C" fn get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut CapeBoolean,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = arr.vec.as_ptr() as *mut CapeBoolean;
			*size = arr.vec.len() as C::CapeSize;
		}
	}

	/// Resize
	///
	/// Resize the array to the given size. If the size is larger than the current size, the new elements are set to `false`.
	///
	/// # Arguments
	///
	/// * `size` - The new size of the array
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayBooleanVec::from_bool_slice(&[true,false]);
	/// arr.resize(4);
	/// assert_eq!(arr.as_vec(), &vec![true as cobia::CapeBoolean,false as cobia::CapeBoolean,false as cobia::CapeBoolean,false as cobia::CapeBoolean]);
	/// ```
	pub fn resize(&mut self, size: usize) {
		self.vec.resize(size, 0);
	}

	/// Interface member function

	extern "C" fn setsize(
		me: *mut ::std::os::raw::c_void,
		size: C::CapeSize,
		data: *mut *mut CapeBoolean,
	) -> CapeResult {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		arr.resize(size as usize);
		unsafe {
			*data = arr.vec.as_ptr() as *mut CapeBoolean;
		}
		COBIAERR_NOERROR
	}

	/// Set the content of the boolean array from any object that implements CapeArrayBooleanProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayBooleanProviderIn
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// let mut arr = cobia::CapeArrayBooleanVec::new();
	/// let arr1 = cobia::CapeArrayBooleanVec::from_slice(&[8,9,7]);
	/// arr.set(&arr1);
	/// assert_eq!(arr.as_vec(), &vec![8,9,7]);
	/// ```
	pub fn set<T:CapeArrayBooleanProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut boolean_array_in_from_provider = CapeArrayBooleanInFromProvider::from(array);
		let boolean_array=boolean_array_in_from_provider.as_cape_array_boolean_in();
		self.vec.clear();
		self.vec.extend_from_slice(boolean_array.as_slice());
		Ok(())
	}

	/// Interface v-table

	const VTABLE: C::ICapeArrayBoolean_VTable = C::ICapeArrayBoolean_VTable {
		get: Some(Self::get),
		setsize: Some(Self::setsize),
	};

}

impl CapeArrayBooleanProviderIn for CapeArrayBooleanVec {
	/// Convert to ICapeArrayBoolean
	///
	/// Returns a reference to the ICapeArrayBoolean interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayBooleanVec::from_bool_slice(&[false,true,false]);
	///	let i_cape_array_boolean=arr.as_cape_array_boolean_in();
	///	let mut i_cape_array_boolean_ptr=(&i_cape_array_boolean as *const C::ICapeArrayBoolean).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayBooleanIn::new(&mut i_cape_array_boolean_ptr); //CapeArrayBooleanIn from *mut C::ICapeArrayBoolean
	/// assert_eq!(a.as_bool_vec(), vec![false,true,false]);
	/// ```

	fn as_cape_array_boolean_in(&self) -> C::ICapeArrayBoolean {
		C::ICapeArrayBoolean {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayBooleanVec::VTABLE as *const C::ICapeArrayBoolean_VTable).cast_mut(),
		}
	}
}

impl CapeArrayBooleanProviderOut for CapeArrayBooleanVec {
	/// Convert to ICapeArrayBoolean
	///
	/// Returns a mutable reference to the ICapeArrayBoolean interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayBooleanVec::from_bool_slice(&[false,true,false]);
	///	let i_cape_array_boolean=arr.as_cape_array_boolean_in();
	///	let mut i_cape_array_boolean_ptr=(&i_cape_array_boolean as *const C::ICapeArrayBoolean).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayBooleanIn::new(&mut i_cape_array_boolean_ptr); //CapeArrayBooleanIn from *mut C::ICapeArrayBoolean
	/// assert_eq!(a.as_bool_vec(), vec![false,true,false]);
	/// ```

	fn as_cape_array_boolean_out(&mut self) -> C::ICapeArrayBoolean {
		C::ICapeArrayBoolean {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayBooleanVec::VTABLE as *const C::ICapeArrayBoolean_VTable).cast_mut(),
		}
	}
}

impl CapeArrayBooleanVec {

	/// Initialize from bool slice
	///
	/// Creates a new CapeArrayTypeImpl from a bool slice.
	///
	/// # Arguments
	///
	/// * `slice` - A vector or array or slice of values to be converted to a CapeArrayTypeImpl - values are copied
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = CapeArrayBooleanVec::from_bool_slice(&[true,false,false]);
	/// assert_eq!(arr.as_vec(), &vec![true as CapeBoolean,false as CapeBoolean,false as CapeBoolean]);
	/// ```

	pub fn from_bool_slice(slice: &[bool]) -> Self {
		let mut a = Self::new();
		let v=a.as_mut_vec();
		v.reserve(slice.len());
		for i in 0..slice.len() {
			v.push(slice[i] as CapeBoolean);
		}
		a
	}

}

impl<T:CapeArrayBooleanProviderIn> PartialEq<T> for CapeArrayBooleanVec {
	/// Partial equality
	///
	/// Checks if the CapeArrayBooleanVec is equal to another CapeArrayBooleanProviderIn.
	///
	/// # Arguments
	///
	/// * `other` - The other CapeArrayBooleanProviderIn to compare with
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr1 = cobia::CapeArrayBooleanVec::from_bool_slice(&[true,false]);
	/// let arr2 = cobia::CapeArrayBooleanVec::from_bool_slice(&[true,false]);
	/// let arr3 = cobia::CapeArrayBooleanVec::from_bool_slice(&[false,false]);
	/// assert!(arr1 == arr2);
	/// assert!(arr1 != arr3);
	/// ```
	fn eq(&self, other: &T) -> bool {
		let mut provider=CapeArrayBooleanInFromProvider::from(other);
		let other=provider.as_cape_array_boolean_in(); 
		self.as_vec() == other.as_slice()
	}
}


/// Vector based CapeArrayEnumerationOut implementation
///
/// ICapeArrayEnmeration is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `Vec<Element>`.
///
/// All CAPE-OPEN and COBIA enumeration types are represented as CapeEnumeration
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn resize(a: &mut CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
///    a.resize(4).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
/// resize(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
/// assert_eq!(arr.size(), 4); 
/// ```
pub type CapeArrayEnumerationVec<Element> = CapeArrayVec<Element,C::ICapeArrayEnumeration>;

type CapeArrayEnumerationVecRaw = CapeArrayVec<C::CapeEnumeration,C::ICapeArrayEnumeration>;

impl CapeArrayEnumerationVecRaw {

	/// Interface member function

	extern "C" fn get_raw(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut C::CapeEnumeration,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = arr.vec.as_ptr() as *mut C::CapeEnumeration;
			*size = arr.vec.len() as C::CapeSize;
		}
	}

	/// Interface member function

	extern "C" fn setsize_raw(
		me: *mut ::std::os::raw::c_void,
		size: C::CapeSize,
		data: *mut *mut C::CapeEnumeration,
	) -> CapeResult {
		let p = me as *mut Self;
		let arr: &mut Self = unsafe { &mut *p };
		arr.vec.resize(size as usize, 0);
		unsafe {
			*data = arr.vec.as_ptr() as *mut C::CapeEnumeration;
		}
		COBIAERR_NOERROR
	}

	/// Interface v-table

	const VTABLE_RAW: C::ICapeArrayEnumeration_VTable = C::ICapeArrayEnumeration_VTable {
		get: Some(Self::get_raw),
		setsize: Some(Self::setsize_raw),
	};

}

impl<Element:Copy+Clone> CapeArrayEnumerationProviderIn for CapeArrayEnumerationVec<Element> {
	/// Convert to ICapeArrayEnumeration
	///
	/// Returns a reference to the ICapeArrayEnumeration interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayEnumerationVec::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	///	let i_cape_array_enumeration=arr.as_cape_array_enumeration_in();
	///	let mut i_cape_array_enumeration_ptr=(&i_cape_array_enumeration as *const C::ICapeArrayEnumeration).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayEnumerationIn::<cobia::CapePMCServiceType>::new(&mut i_cape_array_enumeration_ptr); //CapeArrayEnumerationIn from *mut C::ICapeArrayEnumeration
	/// assert_eq!(a.as_vec(), vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// ```

	fn as_cape_array_enumeration_in(&self) -> C::ICapeArrayEnumeration {
		C::ICapeArrayEnumeration {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayEnumerationVecRaw::VTABLE_RAW as *const C::ICapeArrayEnumeration_VTable).cast_mut(),
		}
	}
}

impl<Element:Copy+Clone> CapeArrayEnumerationVec<Element> {

	/// Resize
	///
	/// Resize the array to the given size. If the size is larger than the current size, the new elements are set specified value.
	///
	/// # Arguments
	///
	/// * `size` - The new size of the array
	/// * `value` - The value to set the new elements to
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayEnumerationVec::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// arr.resize(4,cobia::CapePMCServiceType::Inproc32);
	/// assert_eq!(arr.as_vec(), &vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64,cobia::CapePMCServiceType::Inproc32,cobia::CapePMCServiceType::Inproc32]);
	/// ```
	pub fn resize(&mut self, size: usize, value: Element) {
		self.vec.resize(size, value);
	}

}

impl<Element:Copy+Clone> CapeArrayEnumerationProviderOut for CapeArrayEnumerationVec<Element> {
	/// Convert to ICapeArrayEnumeration
	///
	/// Returns a mutable reference to the ICapeArrayEnumeration interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayEnumerationVec::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	///	let i_cape_array_enumeration=arr.as_cape_array_enumeration_out();
	///	let mut i_cape_array_enumeration_ptr=(&i_cape_array_enumeration as *const C::ICapeArrayEnumeration).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayEnumerationOut::<cobia::CapePMCServiceType>::new(&mut i_cape_array_enumeration_ptr); //CapeArrayEnumerationOut from *mut C::ICapeArrayEnumeration
	/// assert_eq!(a.as_vec(), vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// ```

	fn as_cape_array_enumeration_out(&mut self) -> C::ICapeArrayEnumeration {
		C::ICapeArrayEnumeration {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&CapeArrayEnumerationVecRaw::VTABLE_RAW as *const C::ICapeArrayEnumeration_VTable).cast_mut(),
		}
	}
}
