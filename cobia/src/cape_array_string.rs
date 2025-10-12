use crate::{CapeArrayStringProviderIn,CapeArrayStringProviderOut,CapeStringIn,CapeStringOut,CapeArrayStringInFromProvider};
use std::fmt;
use std::marker::PhantomData;

use crate::COBIAError;
use crate::C;
use crate::cape_result_value::*;

/// CapeArrayStringIn wraps an ICapeArrayString interface pointer in a read-only manner
///
/// Given a reference to an ICapeArrayString interface pointer, this allows getting,
/// but not setting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayString input arguments.
///
/// A NULL interface pointer is treated as an empty array.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayStringIn) {
///		assert_eq!(a.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
/// }
/// 
/// let arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
/// test_content(&CapeArrayStringInFromProvider::from(&arr).as_cape_array_string_in());
/// ```

pub struct CapeArrayStringIn<'a> {
	data: *mut *mut C::ICapeString,
	size: C::CapeSize,
	interface : &'a *mut C::ICapeArrayString,
	_lifetime: PhantomData<&'a ()> //even though we do not refer to the interace after contruction, life time is bound to the interface, as each of the elements are
}

impl<'a> CapeArrayStringIn<'a> {
	/// Create a new CapeStringIn from an ICapeArrayString interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeArrayString interface
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	///	let i_cape_array_string=arr.as_cape_array_string_in();
	///	let mut i_cape_array_string_ptr=(&i_cape_array_string as *const C::ICapeArrayString).cast_mut(); //normally a pointer to the interface is received
	///	let sa = cobia::CapeArrayStringIn::new(&mut i_cape_array_string_ptr); //CapeArrayStringIn from *mut C::ICapeArrayString
	/// assert_eq!(sa.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// ```

	pub fn new(interface: &'a *mut C::ICapeArrayString) -> CapeArrayStringIn<'a> {
		if interface.is_null() {
			CapeArrayStringIn {
				data: std::ptr::null_mut(),
				size: 0,
				interface,
				_lifetime : std::default::Default::default()
			}
		} else {
			let mut data: *mut *mut C::ICapeString = std::ptr::null_mut();
			let mut size: C::CapeSize = 0;
			unsafe { (*(**interface).vTbl).get.unwrap()((**interface).me, &mut data, &mut size) };
			CapeArrayStringIn {
				data,
				size,
				interface,
				_lifetime : std::default::Default::default()
			}
		}
	}

	/// Return the size of the vector
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_size(a: &CapeArrayStringIn) {
	///		assert_eq!(a.size(), 2);
	/// }
	/// 
	/// let arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// test_size(&CapeArrayStringInFromProvider::from(&arr).as_cape_array_string_in());
	/// ```
	pub fn size(&self) -> usize {
		self.size as usize
	}

	/// Check if the array is empty
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_empty(a: &CapeArrayStringIn) {
	///		assert!(a.is_empty());
	/// }
	/// 
	/// let arr = cobia::CapeArrayStringVec::new();
	/// test_empty(&CapeArrayStringInFromProvider::from(&arr).as_cape_array_string_in());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.size == 0
	}

	/// Return the content of the string array as a string vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_content(a: &CapeArrayStringIn) {
	///		assert_eq!(a.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// }
	/// 
	/// let arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// test_content(&CapeArrayStringInFromProvider::from(&arr).as_cape_array_string_in());
	/// ```

	pub fn as_string_vec(&self) -> Vec<String> {
		let mut vec = Vec::new();
		vec.reserve(self.size as usize);
		for i in 0..self.size {
			let p=unsafe { *self.data.add(i as usize) };
			if p.is_null() {
				//this provided by the implementor of ICapeArrayString and should not be null
				vec.push(String::new());
			} else {
				vec.push(CapeStringIn::new(&p).as_string());
			}
		}
		vec
	}

	/// Get an element
	///
	/// # Arguments
	///
	/// * `index` - The index of the element to get
	/// 
	/// Note that neither Index and IndexMut is 
	/// provided for CapeArrayStringIn, because these interfaces
	/// yield a reference to the element, whereas the elements
	/// of CapeArrayStringIn are represented by an interface,
	/// which is conveniently wrapped into CapeStringIn. 
	/// 
	/// Note that the life time of the CapeStringIn is tied to the
	/// life time of the CapeArrayStringIn.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_element(a: &CapeArrayStringIn) {
	///		assert_eq!(a.at(1).unwrap().to_string(), "idealGasEntropy".to_string());
	/// }
	/// 
	/// let arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// test_element(&CapeArrayStringInFromProvider::from(&arr).as_cape_array_string_in());
	/// ```

	pub fn at(&self, index: usize) -> Result<CapeStringIn<'a>, COBIAError> {
		if index >= self.size as usize {
			return Err(COBIAError::Code(COBIAERR_NOSUCHITEM));
		}
		let p=unsafe { self.data.add(index) };
		if (unsafe { *p }).is_null() {
			//this provided by the implementor of ICapeArrayString and should not be null
			Err(COBIAError::Code(COBIAERR_NULLPOINTER))
		} else {
			Ok(CapeStringIn::new(unsafe { &mut *p} ))
		}
	}
}

pub struct CapeArrayStringInIterator<'a> {
	arr: &'a CapeArrayStringIn<'a>,
	index: usize,
}

impl<'a> Iterator for CapeArrayStringInIterator<'a> {
	type Item = CapeStringIn<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.arr.size as usize {
			let res = self.arr.at(self.index);
			self.index += 1;
			Some(res.unwrap())
		} else {
			None
		}
	}
}

impl<'a> CapeArrayStringIn<'a> {
	/// Return an iterator over the string array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: &CapeArrayStringIn) {
	///		let mut iter = a.iter();
	///		assert_eq!(iter.next().unwrap().to_string(), "idealGasEnthalpy".to_string());
	///		assert_eq!(iter.next().unwrap().to_string(), "idealGasEntropy".to_string());
	///		assert!(!iter.next().is_some());
	/// }
	/// 
	/// let arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// test_iter(&CapeArrayStringInFromProvider::from(&arr).as_cape_array_string_in());
	/// ```

	pub fn iter(&self) -> CapeArrayStringInIterator<'_> {
		CapeArrayStringInIterator {
			arr: &self,
			index: 0,
		}
	}
}

impl<'a> fmt::Display for CapeArrayStringIn<'a> {
	/// Display the content of the string array as a string vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_format(a: &CapeArrayStringIn) {
	///		assert_eq!(format!("{}", a), "[\"idealGasEnthalpy\", \"idealGasEntropy\"]");
	/// }
	/// 
	/// let arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// test_format(&CapeArrayStringInFromProvider::from(&arr).as_cape_array_string_in());
	/// ```

	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "[")?;
		for (count, v) in self.iter().enumerate() {
			if count != 0 {
				write!(f, ", ")?;
			}
			write!(f, "\"{}\"", v)?;
		}
		write!(f, "]")
	}
}

impl<'a> CapeArrayStringProviderIn for CapeArrayStringIn<'a> {
	fn as_cape_array_string_in(&self) -> C::ICapeArrayString {
		unsafe { **self.interface }
	}
}

/// CapeArrayStringOut wraps an ICapeArrayString interface pointer.
///
/// Given a reference to an ICapeArrayString interface pointer, this allows setting
///  and getting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayString output arguments.
///
/// NULL interface pointers are not allowed.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_content(a: &mut CapeArrayStringOut) {
///		a.put_array(&["idealGasEnthalpy", "idealGasEntropy"]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayStringVec::new();
/// set_content(&mut CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out());
/// assert_eq!(arr.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
/// ```

pub struct CapeArrayStringOut<'a> {
	interface: &'a mut *mut C::ICapeArrayString,
	data: *mut *mut C::ICapeString,
	size: C::CapeSize,
}

impl<'a> CapeArrayStringOut<'a> {
	/// Create a new CapeStringOut from an ICapeArrayString interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeArrayString interface
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	///	let i_cape_array_string=arr.as_cape_array_string_out();
	///	let mut i_cape_array_string_ptr=(&i_cape_array_string as *const C::ICapeArrayString).cast_mut(); //normally a pointer to the interface is received
	///	let sa = cobia::CapeArrayStringOut::new(&mut i_cape_array_string_ptr); //CapeArrayStringOut from *mut C::ICapeArrayString
	/// assert_eq!(sa.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// ```

	pub fn new(interface: &'a mut *mut C::ICapeArrayString) -> CapeArrayStringOut<'a> {
		let mut data: *mut *mut C::ICapeString = std::ptr::null_mut();
		let mut size: C::CapeSize = 0;
		unsafe {
			(*(**interface).vTbl).get.unwrap()((**interface).me, &mut data, &mut size);
		}
		CapeArrayStringOut {
			interface,
			data,
			size,
		}
	}

	/// Return the content of the string array as a string vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	///	let i_cape_array_string=arr.as_cape_array_string_out();
	///	let mut i_cape_array_string_ptr=(&i_cape_array_string as *const C::ICapeArrayString).cast_mut(); //normally a pointer to the interface is received
	///	let sa = cobia::CapeArrayStringOut::new(&mut i_cape_array_string_ptr); //CapeArrayStringOut from *mut C::ICapeArrayString
	/// assert_eq!(sa.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// ```

	pub fn as_string_vec(&self) -> Vec<String> {
		let mut vec = Vec::new();
		vec.reserve(self.size as usize);
		for i in 0..self.size {
			let p=unsafe { self.data.add(i as usize) };
			if p.is_null() {
				//this provided by the implementor of ICapeArrayString and should not be null
				vec.push(String::new());
			} else {
				vec.push(CapeStringOut::new(unsafe { &mut *p }).as_string());
			}
		}
		vec
	}

	/// Return the size of the vector
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_size(a: &mut CapeArrayStringOut) {
	///		assert_eq!(a.size(), 2);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// test_size(&mut CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out());
	/// ```
	pub fn size(&self) -> usize {
		self.size as usize
	}

	/// Check if the array is empty
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_empty(a: &mut CapeArrayStringOut) {
	///		assert!(a.is_empty());
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayStringVec::new();
	/// test_empty(&mut CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.size == 0
	}

	/// Set the content of the string array from a slice of string slices.
	///
	/// # Arguments
	///
	/// * `arr` - A slice or array of vector of strings or string slices
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_content(a: &mut CapeArrayStringOut) {
	///		a.put_array(&["idealGasEnthalpy", "idealGasEntropy"]).unwrap();
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayStringVec::new();
	/// set_content(&mut CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out());
	/// assert_eq!(arr.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// ```

	pub fn put_array<T: AsRef<str>>(&mut self, array: &[T]) -> Result<(), COBIAError> {
		let mut data:*mut *mut C::ICapeString=std::ptr::null_mut();
		let res = unsafe {
			(*(**self.interface).vTbl).setsize.unwrap()(
				(**self.interface).me,
				array.len() as C::CapeSize,
				&mut data,
			)
		};
		if res == COBIAERR_NOERROR {
			self.size = array.len() as C::CapeSize;
			self.data=data;
			for (i, s) in array.iter().enumerate() {
				let p=unsafe { self.data.add(i as usize) }; 
				if p.is_null() {
					//this provided by the implementor of ICapeArrayString and should not be null
					return Err(COBIAError::Code(COBIAERR_NULLPOINTER));
				}
				let el = CapeStringOut::new(unsafe { &mut *p });
				el.set_string((*s).as_ref())?;
			}
			Ok(())
		} else {
			Err(COBIAError::Code(res))
		}
	}

	/// Get an element
	///
	/// # Arguments
	///
	/// * `index` - The index of the element to get
	/// 
	/// Note that neither Index and IndexMut is 
	/// provided for CapeArrayStringOut, because these interfaces
	/// yield a reference to the element, whereas the elements
	/// of CapeArrayStringOut are represented by an interface,
	/// which is conveniently wrapped into CapeStringOut. 
	/// 
	/// Note that the life time of the CapeStringOut is tied to the
	/// life time of the CapeArrayStringOut.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_element(a: &mut CapeArrayStringOut) {
	///		assert_eq!(a.at(1).unwrap().to_string(), "idealGasEntropy".to_string());
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// test_element(&mut CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out());
	/// ```

	pub fn at(&self, index: usize) -> Result<CapeStringOut<'a>, COBIAError> {
		if index >= self.size as usize {
			return Err(COBIAError::Code(COBIAERR_NOSUCHITEM));
		}
		let p=unsafe { self.data.add(index) };
		if p.is_null() {
			//this provided by the implementor of ICapeArrayString and should not be null
			Err(COBIAError::Code(COBIAERR_NULLPOINTER))
		} else {
			Ok(CapeStringOut::new(unsafe { &mut *p} ))
		}
	}

	/// Resize
	///
	/// # Arguments
	///
	/// * `size` - The new size of the array
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_content(a: &mut CapeArrayStringOut) {
	///		a.resize(3).unwrap();
	///		a.at(0).unwrap().set_string("idealGasEnthalpy").unwrap();
	///		a.at(1).unwrap().set_string("idealGasEntropy").unwrap();
	///		a.at(2).unwrap().set_string("density").unwrap();
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayStringVec::new();
	/// set_content(&mut CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out());
	/// assert_eq!(arr.as_string_vec(), ["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string(), "density".to_string()]); //the values have been stored on the object that implements ICapeArrayString
	/// ```
	pub fn resize(&mut self, size: usize) -> Result<(), COBIAError> {
		let mut data:*mut *mut C::ICapeString=std::ptr::null_mut();
		let res = unsafe {
			(*(**self.interface).vTbl).setsize.unwrap()(
				(**self.interface).me,
				size as C::CapeSize,
				&mut data,
			)
		};
		if res == COBIAERR_NOERROR {
			self.size = size as C::CapeSize;
			self.data=data;
			Ok(())
		} else {
			Err(COBIAError::Code(res))
		}
	}

	/// Set an element
	///
	/// # Arguments
	///
	/// * `index` - The index of the element to set
	/// * `value` - The value to set
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_content(a: &mut CapeArrayStringOut) {
	///		a.resize(3).unwrap();
	///		a.at(0).unwrap().set_string("idealGasEnthalpy").unwrap();
	///		a.at(1).unwrap().set_string("idealGasEntropy").unwrap();
	///		a.at(2).unwrap().set_string("density").unwrap();
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayStringVec::new();
	/// set_content(&mut CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out());
	/// assert_eq!(arr.as_string_vec(), ["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string(), "density".to_string()]); //the values have been stored on the object that implements ICapeArrayString
	/// ```
	pub fn set_string<T: AsRef<str>>(&mut self, index: usize, value: T) -> Result<(), COBIAError> {
		if index >= self.size as usize {
			return Err(COBIAError::Code(COBIAERR_NOSUCHITEM));
		}
		let p=unsafe { self.data.add(index) };
		if p.is_null() {
			//this provided by the implementor of ICapeArrayString and should not be null
			return Err(COBIAError::Code(COBIAERR_NULLPOINTER));
		}
		let el = CapeStringOut::new(unsafe { &mut *p });
		el.set_string(value.as_ref())
	}

	/// Set the content of the string array from any object that implements CapeArrayStringProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayStringProviderIn
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut arr = cobia::CapeArrayStringVec::new();
	/// let mut arr1 = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out().set(&arr1);
	/// assert_eq!(arr.as_string_vec(), ["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]); //the values have been stored on the object that implements ICapeArrayString
	/// ```

	pub fn set<T:CapeArrayStringProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut string_array_in_from_provider = CapeArrayStringInFromProvider::from(array);
		let string_array=string_array_in_from_provider.as_cape_array_string_in();
		self.resize(string_array.size())?;
		for i in 0..string_array.size() {
			self.at(i)?.set(&string_array.at(i)?)?;
		}
		Ok(())
	}

}

pub struct CapeArrayStringIterator<'a> {
	arr: &'a CapeArrayStringOut<'a>,
	index: usize,
}

impl<'a> Iterator for CapeArrayStringIterator<'a> {
	type Item = CapeStringOut<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.arr.size as usize {
			let res = self.arr.at(self.index);
			self.index += 1;
			Some(res.unwrap())
		} else {
			None
		}
	}
}

impl<'a> CapeArrayStringOut<'a> {
	/// Return an iterator over the string array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: &mut CapeArrayStringOut) {
	///		let mut iter = a.iter();
	///		assert_eq!(iter.next().unwrap().to_string(), "idealGasEnthalpy".to_string());
	///		assert_eq!(iter.next().unwrap().to_string(), "idealGasEntropy".to_string());
	///		assert!(!iter.next().is_some());
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// test_iter(&mut CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out());
	/// ```

	pub fn iter(&self) -> CapeArrayStringIterator<'_> {
		CapeArrayStringIterator {
			arr: &self,
			index: 0,
		}
	}
}

impl<'a> fmt::Display for CapeArrayStringOut<'a> {
	/// Display the content of the string array as a string vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_format(a: &mut CapeArrayStringOut) {
	///		assert_eq!(format!("{}", a), "[\"idealGasEnthalpy\", \"idealGasEntropy\"]");
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// test_format(&mut CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out());
	/// ```

	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "[")?;
		for (count, v) in self.iter().enumerate() {
			if count != 0 {
				write!(f, ", ")?;
			}
			write!(f, "\"{}\"", v)?;
		}
		write!(f, "]")
	}
}

impl<'a> CapeArrayStringProviderOut  for CapeArrayStringOut<'a> {
	fn as_cape_array_string_out(&mut self) -> C::ICapeArrayString {
		unsafe { **self.interface }
	}
}