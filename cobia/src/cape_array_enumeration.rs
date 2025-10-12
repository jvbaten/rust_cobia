use std::fmt;
use crate::cape_array::CapeArrayRefIterator;
use crate::{COBIAError,CapeArrayEnumerationProviderIn,CapeArrayEnumerationProviderOut};
use crate::C;
use crate::cape_result_value::*;

/// CapeArrayEnumerationIn wraps an ICapeArrayEnumeration interface pointer.
///
/// Given a reference to an ICapeArrayEnumeration interface pointer, this allows setting
///  and getting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayEnumeration input arguments.
///
/// This class takes a mutable reference to the interface pointer, as
/// it should be the only class that is in use at a time to change the 
/// data behind the interface (as the data pointer is cached)
///
/// A NULL pointer is treated as an empty array.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayEnumerationIn<cobia::CapePMCServiceType>) {
///     assert_eq!(a.as_vec(), vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
/// }
/// 
/// let arr = cobia::CapeArrayEnumerationVec::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
/// test_content(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
/// ```

pub struct CapeArrayEnumerationIn<'a,Element:Copy+Clone> {
	interface: &'a *mut C::ICapeArrayEnumeration,
	data: *mut Element,
	size: C::CapeSize,
}

impl<'a,Element:Copy+Clone> CapeArrayEnumerationIn<'a,Element> {
	/// Create a new CapeArrayEnumerationIn from an ICapeArrayEnumeration interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeArrayEnumeration interface
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	///	let i_cape_array_enumeration=arr.as_cape_array_enumeration_in();
	///	let mut i_cape_array_enumeration_ptr=(&i_cape_array_enumeration as *const C::ICapeArrayEnumeration).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayEnumerationIn::<cobia::CapePMCServiceType>::new(&mut i_cape_array_enumeration_ptr); //CapeArrayEnumerationIn from *mut C::ICapeArrayEnumeration
	/// assert_eq!(a.as_vec(), vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// ```

	pub fn new(interface: &'a *mut C::ICapeArrayEnumeration) -> Self {
		if interface.is_null() {
			return Self {
				interface,
				data: std::ptr::null_mut(),
				size: 0,
			};
		}
		let mut data: *mut Element = std::ptr::null_mut();
		let mut size: C::CapeSize = 0;
		unsafe { (*(**interface).vTbl).get.unwrap()((**interface).me,&mut data as *mut *mut Element as *mut *mut i32,&mut size as *mut C::CapeSize) };
		Self {
			interface,
			data,
			size,
		}
	}

	/// Return the size of the array
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_size(a: &CapeArrayEnumerationIn<cobia::CapePMCServiceType>) {
	///     assert_eq!(a.size(), 2);
	/// }
	/// 
	/// let arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_size(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
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
	/// fn test_empty(a: &CapeArrayEnumerationIn<cobia::CapePMCServiceType>) {
	///     assert!(a.is_empty());
	/// }
	/// 
	/// let arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::new();
	/// test_empty(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.size == 0
	}
	
	/// Return the content of the array as a vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_content(a: &CapeArrayEnumerationIn<cobia::CapePMCServiceType>) {
	///     assert_eq!(a.as_vec(), vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// }
	/// 
	/// let arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_content(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
	/// ```

	pub fn as_vec(&self) -> Vec<Element> {
		let slice = unsafe { std::slice::from_raw_parts(self.data, self.size as usize) };
		slice.to_vec()
	}

	/// Return the content of the real array as a real slice.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_content(a: &CapeArrayEnumerationIn<cobia::CapePMCServiceType>) {
	///     assert_eq!(a.as_slice(), &[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// }
	/// 
	/// let arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_content(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
	/// ```

	pub fn as_slice(&self) -> &[Element] {
		let slice = unsafe { std::slice::from_raw_parts(self.data, self.size as usize) };
		slice
	}

	/// Return an iterator for the array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: &CapeArrayEnumerationIn<cobia::CapePMCServiceType>) {
	///		let mut iter = a.iter();
	///		assert_eq!(iter.next().unwrap(), cobia::CapePMCServiceType::Inproc64);
	///		assert_eq!(iter.next().unwrap(), cobia::CapePMCServiceType::COM64);
	///		assert!(!iter.next().is_some());
	/// }
	/// 
	/// let arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_iter(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
	/// ```

	pub fn iter(&self) -> CapeArrayRefIterator<'_,Element> {
		CapeArrayRefIterator {
			data: &self.as_slice(),
			index: 0
		}
	}

}

impl<'a,Element:Copy+Clone> std::ops::Index<usize> for CapeArrayEnumerationIn<'a,Element> {
	type Output = Element;

	/// Indexing
	///
	/// Returns a reference to the string at the given index.
	///
	/// # Arguments
	///
	/// * `index` - The index of the string to be returned
	///
	/// # Examples
	///
	/// ```rust
	/// use cobia::*;
	///
	/// fn test_item(a: &CapeArrayEnumerationIn<cobia::CapePMCServiceType>) {
	///		assert_eq!(a[1], cobia::CapePMCServiceType::COM64);
	/// }
	/// 
	/// let arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_item(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
	/// ```

	fn index(&self, index: usize) -> &Self::Output {
		if index>=(self.size as usize) {
			panic!("index out of bounds");
		}
		unsafe { &*self.data.add(index) as &Element }
	}

}

impl<'a,Element:Copy+Clone+std::fmt::Display> fmt::Display for CapeArrayEnumerationIn<'a,Element> {

	/// Display the content of the real array as a real vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_format(a: &CapeArrayEnumerationIn<cobia::CapePMCServiceType>) {
	///     assert_eq!(format!("{}", a), "[Inproc64, COM64, Local]");
	/// }
	/// 
	/// let arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64,cobia::CapePMCServiceType::Local]);
	/// test_format(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
	/// ```

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (count, v) in self.iter().enumerate() {
            if count != 0 { write!(f, ", ")?; }
            write!(f, "{}", v)?;
        }
        write!(f, "]")
    }
}


impl<'a,Element:Copy+Clone+'a> IntoIterator for &'a CapeArrayEnumerationIn<'a,Element> {
	type Item = Element;
	type IntoIter = CapeArrayRefIterator<'a, Element>;

	/// Return an iterator over the real array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: &CapeArrayEnumerationIn<cobia::CapePMCServiceType>) {
	///		let mut hasCOM64=false;
	///		for val in a {
	///		 if val==cobia::CapePMCServiceType::COM64 {
	///			    hasCOM64=true;
	///		 }
	///		}
	///		assert!(hasCOM64);
	/// }
	/// 
	/// let arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_iter(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
	/// ```

	fn into_iter(self) -> CapeArrayRefIterator<'a, Element> {
		CapeArrayRefIterator {
			data: self.as_slice(),
			index: 0,
		}
	}
}

/// An iterator that consumes a CapeArrayEnumerationIn
///
/// This struct is created by the IntoIterator trait on CapeArrayEnumerationIn.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_iter(a: &CapeArrayEnumerationIn<cobia::CapePMCServiceType>) {
///		let mut hasCOM64=false;
///		for val in a {
///		 if val==cobia::CapePMCServiceType::COM64 {
///			    hasCOM64=true;
///		 }
///		}
///		assert!(hasCOM64);
/// }
/// 
/// let arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
/// test_iter(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
/// ```

pub struct CapeArrayEnumerationIteratorIn<'a,Element:Copy+Clone> {
	arr: CapeArrayEnumerationIn<'a,Element>,
	index: usize
}

impl<'a,Element:Copy+Clone> Iterator for CapeArrayEnumerationIteratorIn<'a,Element> {
	type Item = Element;
	fn next(&mut self) -> Option<Self::Item> {
		if self.index < (self.arr.size as usize) {
			let res = unsafe{ *self.arr.data.add(self.index)};
			self.index += 1;
			Some(res)
		} else {
			None
		}
	}
}


impl<'a,Element:Copy+Clone> IntoIterator for CapeArrayEnumerationIn<'a,Element> {
	type Item = Element;
	type IntoIter = CapeArrayEnumerationIteratorIn<'a,Element>;

	/// Return an iterator over the real array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: &CapeArrayEnumerationIn<cobia::CapePMCServiceType>) {
	///		let mut hasCOM64=false;
	///		for val in a {
	///		 if val==cobia::CapePMCServiceType::COM64 {
	///			    hasCOM64=true;
	///		 }
	///		}
	///		assert!(hasCOM64);
	/// }
	/// 
	/// let arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_iter(&CapeArrayEnumerationInFromProvider::from(&arr).as_cape_array_enumeration_in());
	/// ```

	fn into_iter(self) -> Self::IntoIter {
		CapeArrayEnumerationIteratorIn {
			arr: self,
			index: 0,
		}
	}
}

impl<'a,Element:Copy+Clone> CapeArrayEnumerationProviderIn for CapeArrayEnumerationIn<'a,Element> {
	fn as_cape_array_enumeration_in(&self) -> C::ICapeArrayEnumeration {
		unsafe { **self.interface }
	}
}

/// CapeArrayEnumerationOut wraps an ICapeArrayEnumeration interface pointer.
///
/// Given an ICapeArrayEnumeration interface pointer, this allows setting
///  and getting the elements.
///
/// A NULL pointer is not allowed.
/// 
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayEnumeration ouput arguments.
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
/// fn set_content(a: &mut CapeArrayEnumerationOut<CapePMCServiceType>) {
///		a.put_array(&[CapePMCServiceType::Inproc64,CapePMCServiceType::COM64]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayEnumerationVec::<CapePMCServiceType>::new();
/// set_content(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
/// assert_eq!(arr.as_vec(), &vec![CapePMCServiceType::Inproc64,CapePMCServiceType::COM64]);
/// ```

pub struct CapeArrayEnumerationOut<'a,Element:Copy+Clone> {
	interface: &'a mut *mut C::ICapeArrayEnumeration,
	data: *mut Element,
	size: C::CapeSize,
}

impl<'a,Element:Copy+Clone> CapeArrayEnumerationOut<'a,Element> {
	/// Create a new CapeArrayEnumerationOut from an ICapeArrayEnumeration interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeArrayEnumeration interface
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	///	let i_cape_array_enumeration=arr.as_cape_array_enumeration_out();
	///	let mut i_cape_array_enumeration_ptr=(&i_cape_array_enumeration as *const C::ICapeArrayEnumeration).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayEnumerationOut::<cobia::CapePMCServiceType>::new(&mut i_cape_array_enumeration_ptr); //CapeArrayEnumerationOut from *mut C::ICapeArrayEnumeration
	/// assert_eq!(a.as_vec(), vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// ```

	pub fn new(interface: &'a mut *mut C::ICapeArrayEnumeration) -> Self {
		if interface.is_null() {
			panic!("CapeArrayEnumerationOut cannot be created from a NULL pointer");
		}
		let mut data: *mut Element = std::ptr::null_mut();
		let mut size: C::CapeSize = 0;
		unsafe { (*(**interface).vTbl).get.unwrap()((**interface).me,&mut data as *mut *mut Element as *mut *mut i32,&mut size as *mut C::CapeSize) };
		Self {
			interface,
			data,
			size,
		}
	}

	/// Get the size of the array
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_size(a: &mut CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///     assert_eq!(a.size(), 2);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_size(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
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
	/// fn test_empty(a: &mut CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///     assert!(a.is_empty());
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::new();
	/// test_empty(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.size == 0
	}
	
	/// Return the content of the array as a vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_content(a: &mut CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///    assert_eq!(a.as_vec(), vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_content(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
	/// ```

	pub fn as_vec(&self) -> Vec<Element> {
		let slice = unsafe { std::slice::from_raw_parts(self.data, self.size as usize) };
		slice.to_vec()
	}

	/// Set the content of the array from a slice
	///
	/// # Arguments
	///
	/// * `arr` - A slice or array of vector
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_content(a: &mut CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///    a.put_array(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]).unwrap();
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// set_content(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
	/// assert_eq!(arr.as_vec(), &vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]); //the values have been stored on the object that implements ICapeArrayEnumeration<cobia::CapePMCServiceType>
	/// ```

	pub fn put_array(&mut self, array: &[Element]) -> Result<(), COBIAError> {
		let res=unsafe {( *((**self.interface).vTbl) ).setsize.unwrap()((**self.interface).me,array.len() as C::CapeSize,&mut self.data as *mut *mut Element as *mut *mut i32)};
		if res == COBIAERR_NOERROR {
			self.size = array.len() as C::CapeSize;
			for (i, val) in array.iter().enumerate() {
				unsafe { *self.data.add(i as usize) = *val };
			}
			Ok(())
		} else {
			Err(COBIAError::Code(res))
		}
	}

	/// Resize the array
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
	/// fn resize(a: &mut CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///    a.resize(4).unwrap();
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_slice(&[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// resize(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
	/// assert_eq!(arr.size(), 4); 
	/// ```

	pub fn resize(&mut self, size: usize) -> Result<(), COBIAError> {
		let res=unsafe {( *((**self.interface).vTbl) ).setsize.unwrap()((**self.interface).me,size as C::CapeSize,&mut self.data as *mut *mut Element as *mut *mut i32)};
		if res == COBIAERR_NOERROR {
			self.size = size as C::CapeSize;
			Ok(())
		} else {
			Err(COBIAError::Code(res))
		}
	}

	/// Return the content of the real array as a real slice.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_content(a: &mut CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///    assert_eq!(a.as_slice(), &[cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_content(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
	/// ```

	pub fn as_slice(&self) -> &[Element] {
		let slice = unsafe { std::slice::from_raw_parts(self.data, self.size as usize) };
		slice
	}

	/// Return an iterator for the array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: &mut CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///    let mut iter = a.iter();
	///		assert_eq!(iter.next().unwrap(), cobia::CapePMCServiceType::Inproc64);
	///		assert_eq!(iter.next().unwrap(), cobia::CapePMCServiceType::COM64);
	///		assert!(!iter.next().is_some());
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_iter(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
	/// ```

	pub fn iter(&self) -> CapeArrayRefIterator<'_,Element> {
		CapeArrayRefIterator {
			data: &self.as_slice(),
			index: 0
		}
	}

}

impl<'a,Element:Copy+Clone> std::ops::Index<usize> for CapeArrayEnumerationOut<'a,Element> {
	type Output = Element;

	/// Indexing
	///
	/// Returns a reference to the string at the given index.
	///
	/// # Arguments
	///
	/// * `index` - The index of the string to be returned
	///
	/// # Examples
	///
	/// ```rust
	/// use cobia::*;
	///
	/// fn test_index(a: &mut CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///    assert_eq!(a[1], cobia::CapePMCServiceType::COM64);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_index(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
	/// ```

	fn index(&self, index: usize) -> &Self::Output {
		if index>=(self.size as usize) {
			panic!("index out of bounds");
		}
		unsafe { &*self.data.add(index) as &Element }
	}

}

impl<'a,Element:Copy+Clone> std::ops::IndexMut<usize> for CapeArrayEnumerationOut<'a,Element> {
	/// Indexing
	///
	/// Returns a mutable reference to the string at the given index.
	///
	/// # Arguments
	///
	/// * `index` - The index of the string to be returned
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_content(a: &mut CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///    assert_eq!(a.as_vec(), vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM32]);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM32]);
	/// test_content(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
	/// ```

	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		if index>=(self.size as usize) {
			panic!("index out of bounds");
		}
		unsafe { &mut *self.data.add(index) as &mut Element }
	}
}


impl<'a,Element:Copy+Clone+std::fmt::Display> fmt::Display for CapeArrayEnumerationOut<'a,Element> {

	/// Display the content of the real array as a real vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_format(a: &mut CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///    assert_eq!(format!("{}", a), "[Inproc64, COM64, Local]");
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64,cobia::CapePMCServiceType::Local]);
	/// test_format(&mut CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
	/// ```

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (count, v) in self.iter().enumerate() {
            if count != 0 { write!(f, ", ")?; }
            write!(f, "{}", v)?;
        }
        write!(f, "]")
    }
}


impl<'a,Element:Copy+Clone+'a> IntoIterator for &'a CapeArrayEnumerationOut<'a,Element> {
	type Item = Element;
	type IntoIter = CapeArrayRefIterator<'a, Element>;

	/// Return an iterator over the real array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///		let mut hasCOM64=false;
	///		for val in a {
	///		    if val==cobia::CapePMCServiceType::COM64 {
	///		        hasCOM64=true;
	///		    }
	///		}
	///		assert!(hasCOM64);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_iter(CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
	/// ```

	fn into_iter(self) -> CapeArrayRefIterator<'a, Element> {
		CapeArrayRefIterator {
			data: self.as_slice(),
			index: 0,
		}
	}
}

/// An iterator that consumes a CapeArrayEnumerationOut
///
/// This struct is created by the IntoIterator trait on CapeArrayEnumerationOut.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_iter(a: CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
///		let mut hasCOM64=false;
///		for val in a {
///		    if val==cobia::CapePMCServiceType::COM64 {
///		        hasCOM64=true;
///		    }
///		}
///		assert!(hasCOM64);
/// }
/// 
/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
/// test_iter(CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
/// ```


pub struct CapeArrayEnumerationIteratorOut<'a,Element:Copy+Clone> {
	arr: CapeArrayEnumerationOut<'a,Element>,
	index: usize
}

impl<'a,Element:Copy+Clone> Iterator for CapeArrayEnumerationIteratorOut<'a,Element> {
	type Item = Element;
	fn next(&mut self) -> Option<Self::Item> {
		if self.index < (self.arr.size as usize) {
			let res = unsafe{ *self.arr.data.add(self.index)};
			self.index += 1;
			Some(res)
		} else {
			None
		}
	}
}


impl<'a,Element:Copy+Clone> IntoIterator for CapeArrayEnumerationOut<'a,Element> {
	type Item = Element;
	type IntoIter = CapeArrayEnumerationIteratorOut<'a,Element>;

	/// Return an iterator over the real array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: CapeArrayEnumerationOut<cobia::CapePMCServiceType>) {
	///		let mut hasCOM64=false;
	///		for val in a {
	///		    if val==cobia::CapePMCServiceType::COM64 {
	///		        hasCOM64=true;
	///		    }
	///		}
	///		assert!(hasCOM64);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayEnumerationVec::<cobia::CapePMCServiceType>::from_vec(vec![cobia::CapePMCServiceType::Inproc64,cobia::CapePMCServiceType::COM64]);
	/// test_iter(CapeArrayEnumerationOutFromProvider::from(&mut arr).as_cape_array_enumeration_out());
	/// ```

	fn into_iter(self) -> Self::IntoIter {
		CapeArrayEnumerationIteratorOut {
			arr: self,
			index: 0,
		}
	}
}

impl<'a,Element:Copy+Clone> CapeArrayEnumerationProviderOut for CapeArrayEnumerationOut<'a,Element> {
	fn as_cape_array_enumeration_out(&mut self) -> C::ICapeArrayEnumeration {
		unsafe { **self.interface }
	}
}