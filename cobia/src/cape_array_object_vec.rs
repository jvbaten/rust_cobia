use crate::C;
use crate::*;

/// Base implementation for CapeArrayStringVec and CapeArrayValueVec

#[allow(private_bounds)]
#[derive (Clone)]
pub struct CapeArrayObjectVec<ElementImpl,ElementInterface> {
	vec: Vec<ElementImpl>,
	interface_vec: Vec<ElementInterface>,
	interface_ptr_vec: Vec<*mut ElementInterface>,
}

#[allow(private_bounds)]
impl<ElementImpl,ElementInterface> CapeArrayObjectVec<ElementImpl,ElementInterface> {

	/// Create a new CapeArrayObjectVec
	///
	/// Creates a new CapeArrayObjectVec with an empty array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayStringVec::new();
	/// assert_eq!(arr.as_string_vec().len(), 0);
	/// ```
	pub fn new() -> Self {
		Self {
			vec: Vec::new(),
			interface_vec: Vec::new(),
			interface_ptr_vec: Vec::new(),
		}
	}

	/// Return a vector
	///
	/// Returns a reference to the vector of ElementImpl.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayStringVec::from_slice(&["idealGasEnthalpy", "idealGasEntropy"]);
	/// assert_eq!(arr.as_vec()[0].as_string(), "idealGasEnthalpy".to_string());
	/// ```

	pub fn as_vec(&self) -> &Vec<ElementImpl> {
		&self.vec
	}

	///Return size 
	///
	///Returns the size of the vector.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayStringVec::from_slice(&["idealGasEnthalpy", "idealGasEntropy"]);
	/// assert_eq!(arr.size(), 2);
	/// ```
	pub fn size(&self) -> usize {
		self.vec.len()
	}

	/// Check if empty
	///
	/// Returns true if the vector is empty.
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayStringVec::new();
	/// assert!(arr.is_empty());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.vec.is_empty()
	}	

	/// Return an iterator for the array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: &CapeArrayStringIn) {
	///		let mut iter = a.iter();
	///		assert_eq!(cobia::CapeStringConstNoCase::from_string("IDEALGASENTHALPY"),iter.next().unwrap());
	///		assert_eq!(cobia::CapeStringConstNoCase::from_string("idealgasentropy"),iter.next().unwrap());
	///		assert!(!iter.next().is_some());
	/// }
	/// 
	/// let arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy", "idealGasEntropy"]);
	/// test_iter(&CapeArrayStringInFromProvider::from(&arr).as_cape_array_string_in());
	/// ```

	pub fn iter(&self) -> CapeArrayObjectRefIterator<'_,ElementImpl> {
		CapeArrayObjectRefIterator {
			data: &self.vec.as_slice(),
			index: 0
		}
	}

}

/// An iterator that takes a reference to the data in CapeArrayObjectVec
///
/// This struct is created by the iter method on CapeArrayObjectVec as well as by the IntoInterator trait on &CapeArrayObjectVec
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_iter(a: &CapeArrayStringIn) {
///		let mut iter = a.iter();
///		assert_eq!(cobia::CapeStringConstNoCase::from_string("IDEALGASENTHALPY"),iter.next().unwrap());
///		assert_eq!(cobia::CapeStringConstNoCase::from_string("idealgasentropy"),iter.next().unwrap());
///		assert!(!iter.next().is_some());
/// }
/// 
/// let arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy", "idealGasEntropy"]);
/// test_iter(&CapeArrayStringInFromProvider::from(&arr).as_cape_array_string_in());
/// ```

pub struct CapeArrayObjectRefIterator<'a, ElementImpl> {
	pub(crate) data: &'a[ElementImpl],
	pub(crate) index: usize
}

impl<'a, ElementImpl> Iterator for CapeArrayObjectRefIterator<'a, ElementImpl> {
	type Item = &'a ElementImpl;
	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.data.len() {
			let res = &self.data[self.index];
			self.index += 1;
			Some(res)
		} else {
			None
		}
	}
}

impl<ElementImpl,ElementInterface> std::ops::Index<usize> for CapeArrayObjectVec<ElementImpl,ElementInterface> {
	type Output = ElementImpl;

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
	/// let arr = cobia::CapeArrayStringVec::from_slice(&["idealGasEnthalpy", "idealGasEntropy"]);
	/// assert_eq!(arr[0].as_string(), "idealGasEnthalpy");
	/// ```

	fn index(&self, index: usize) -> &Self::Output {
		&self.vec[index]
	}
}

impl<ElementImpl,ElementInterface> std::ops::IndexMut<usize> for CapeArrayObjectVec<ElementImpl,ElementInterface> {
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
	/// let mut arr = cobia::CapeArrayStringVec::from_slice(&["idealGasEnthalpy", "idealGasEntropy"]);
	/// arr[0].set_string("idealGasHeatCapacity");
	/// assert_eq!(arr.as_string_vec(), vec!["idealGasHeatCapacity".to_string(), "idealGasEntropy".to_string()]);
	/// ```

	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.vec[index]
	}
}

/// Vector based CapeArrayStringOut implementation
///
/// ICapeArrayString is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `Vec<CapeStringImpl>`.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_content(a: &mut CapeArrayStringOut) {
///     a.put_array(&["idealGasEnthalpy", "idealGasEntropy"]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayStringVec::new();
/// set_content(&mut CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out());
/// assert_eq!(arr.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
/// ```
pub type CapeArrayStringVec = CapeArrayObjectVec<CapeStringImpl,C::ICapeString>;


impl CapeArrayStringVec {

	/// Initialize from string vector
	///
	/// Creates a new CapeArrayStringVec from a vector of array of strings.
	///
	/// # Arguments
	///
	/// * `array` - A vector or array or slice of strings or string slices to be converted to a CapeArrayStringVec
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let arr = cobia::CapeArrayStringVec::from_slice(&["idealGasEnthalpy", "idealGasEntropy"]);
	/// assert_eq!(arr.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// ```
	pub fn from_slice<T: AsRef<str>>(array: &[T]) -> CapeArrayStringVec {
		let mut vec = Vec::new();
		let mut interface_vec = Vec::new();
		let mut interface_ptr_vec = Vec::new();
		vec .reserve(array.len());
		interface_vec.reserve(array.len());
		interface_ptr_vec.reserve(array.len());
		for s in array.iter() {
			vec.push(CapeStringImpl::from_string(s.as_ref()));
			interface_vec.push(vec.last_mut().unwrap().as_cape_string_out());
			interface_ptr_vec.push((interface_vec.last().unwrap() as *const C::ICapeString).cast_mut());
		}
		Self {
			vec: vec,
			interface_vec,
			interface_ptr_vec,
		}
	}

	/// Return a string vector
	///
	/// Returns a vector of strings from the CapeArrayStringVec.
	///
	/// Note that this operation comes with an overhead of string conversion.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let arr = cobia::CapeArrayStringVec::from_slice(&["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// let strvec = arr.as_string_vec();
	/// assert_eq!(strvec, vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// ```
	pub fn as_string_vec(&self) -> Vec<String> {
		let mut vec = Vec::new();
		vec.reserve(self.vec.len());
		for s in self.vec.iter() {
			vec.push(s.as_string());
		}
		vec
	}

	/// Resize
	///
	/// Change the size of the vector.
	///
	/// # Arguments
	///
	/// * `size` - The new size of the vector
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayStringVec::from_slice(&["idealGasEnthalpy", "idealGasEntropy"]);
	/// arr.resize(3);
	/// assert_eq!(arr.size(), 3);
	/// ```
	pub fn resize(&mut self, size: usize) {
		let old_size = self.vec.len();
		let size = size as usize;
		if size < old_size {
			self.vec.truncate(size);
			self.interface_vec.truncate(size);
			self.interface_ptr_vec.truncate(size);
		} else {
			self.vec.reserve((size - old_size) as usize);
			for _ in old_size..size {
				self.vec.push(CapeStringImpl::new());
				self.interface_vec.push(self.vec.last_mut().unwrap().as_cape_string_out());
			}
			//vector may have been re-allocated, redo interfaces
			self.interface_ptr_vec.resize(size, std::ptr::null_mut());
			for i in old_size..size {
				self.interface_vec[i]=self.vec[i].as_cape_string_out();
				self.interface_ptr_vec[i]=(&self.interface_vec[i] as *const C::ICapeString).cast_mut();
			}
		}
	}

	/// Set the content of the string array from any object that implements CapeArrayStringProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayStringProviderIn
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	///
	/// let mut arr = cobia::CapeArrayStringVec::new();
	/// let arr1 = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// arr.set(&arr1);
	/// assert_eq!(arr.as_string_vec(), ["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]); //the values have been stored on the object that implements ICapeArrayString
	/// ```
	pub fn set<T:CapeArrayStringProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut string_array_in_from_provider = CapeArrayStringInFromProvider::from(array);
		let string_array=string_array_in_from_provider.as_cape_array_string_in();
		self.resize(string_array.size());
		for i in 0..string_array.size() {
			self.vec[i].set(&string_array.at(i)?);
		}
		Ok(())
	}

	///interface member

	extern "C" fn get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut *mut C::ICapeString,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let str_arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = str_arr.interface_ptr_vec.as_ptr() as *mut *mut C::ICapeString;
			*size = str_arr.interface_vec.len() as C::CapeSize;
		}
	}

	///interface member

	extern "C" fn setsize(
		me: *mut ::std::os::raw::c_void,
		size: C::CapeSize,
		data: *mut *mut *mut C::ICapeString,
	) -> C::CapeResult {
		let p = me as *mut Self;
		let str_arr: &mut Self = unsafe { &mut *p };
		str_arr.resize(size as usize);
		unsafe {
			*data = str_arr.interface_ptr_vec.as_ptr() as *mut *mut C::ICapeString;
		}
		COBIAERR_NOERROR
	}

	///interface v-table

	const CAPE_ARRAY_STRING_VTABLE: C::ICapeArrayString_VTable = C::ICapeArrayString_VTable {
		get: Some(Self::get),
		setsize: Some(Self::setsize),
	};

}

impl<T: AsRef<str>,const N: usize> From<&[T;N]> for CapeArrayStringVec {
	/// Creates a new CapeArrayStringVec from reference to a slice of strings.
	///
	/// # Arguments
	///
	/// * `array` - A slice of strings or string slices to be converted to a CapeArrayStringVec
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let arr = cobia::CapeArrayStringVec::from(&["idealGasEnthalpy", "idealGasEntropy"]); 
	/// assert_eq!(arr.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// ```

	fn from(array: &[T; N]) -> CapeArrayStringVec {
		CapeArrayStringVec::from_slice(array)
	}
}

impl<T: AsRef<str>> From<&[T]> for CapeArrayStringVec {
	/// Creates a new CapeArrayStringVec from reference to a slice of strings.
	///
	/// # Arguments
	///
	/// * `array` - A slice of strings or string slices to be converted to a CapeArrayStringVec
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let arr = cobia::CapeArrayStringVec::from(&["idealGasEnthalpy", "idealGasEntropy"]); 
	/// assert_eq!(arr.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// ```

	fn from(array: &[T]) -> CapeArrayStringVec {
		CapeArrayStringVec::from_slice(array)
	}
}

impl<T: CapeArrayStringProviderIn> PartialEq<T> for CapeArrayStringVec {
	/// Partial equality
	///
	/// Checks if the content of the CapeArrayStringVec is equal to the content of another object that implements CapeArrayStringProviderIn.
	///
	/// # Arguments
	///
	/// * `other` - An object that implements CapeArrayStringProviderIn
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let arr1 = cobia::CapeArrayStringVec::from_slice(&["idealGasEnthalpy", "idealGasEntropy"]);
	/// let arr2 = cobia::CapeArrayStringVec::from_slice(&["idealGasEnthalpy", "idealGasEntropy"]);
	/// let arr3 = cobia::CapeArrayStringVec::from_slice(&["IdealGasEnthalpy", "IdealGasEntropy"]);
	/// assert!(arr1 == arr2);
	/// assert!(arr1 != arr3);
	/// ```
	fn eq(&self, other: &T) -> bool {
		let mut provider=CapeArrayStringInFromProvider::from(other);
		let other=provider.as_cape_array_string_in(); 
		if self.size() != other.size() {
			return false;
		}
		//compare the string vectors
		for (i, s) in self.vec.iter().enumerate() {
			match other.at(i) {
				Ok(s_other) => {
					if s != &s_other {
						return false;
					}
				},
				Err(_) => return false, //if we cannot get the string, they are not equal
			}
		}
		true
	}
}

impl CapeArrayStringProviderIn for CapeArrayStringVec {
	/// Convert to ICapeArrayString
	///
	/// Returns a reference to the ICapeArrayString interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayStringVec::from_slice(&vec!["idealGasEnthalpy", "idealGasEntropy"]);
	///	let i_cape_array_string=arr.as_cape_array_string_in();
	///	let mut i_cape_array_string_ptr=(&i_cape_array_string as *const C::ICapeArrayString).cast_mut(); //normally a pointer to the interface is received
	///	let sa = cobia::CapeArrayStringIn::new(&mut i_cape_array_string_ptr); //CapeArrayStringIn from *mut C::ICapeArrayString
	/// assert_eq!(sa.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// ```

	fn as_cape_array_string_in(&self) -> C::ICapeArrayString {
		C::ICapeArrayString {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&Self::CAPE_ARRAY_STRING_VTABLE as *const C::ICapeArrayString_VTable).cast_mut(),
		}
	}
}

impl CapeArrayStringProviderOut for CapeArrayStringVec {
	/// Convert to ICapeArrayString
	///
	/// Returns a mutable reference to the ICapeArrayString interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayStringVec::from(&["idealGasEnthalpy", "idealGasEntropy"]);
	///	let i_cape_array_string=arr.as_cape_array_string_out();
	///	let mut i_cape_array_string_ptr=(&i_cape_array_string as *const C::ICapeArrayString).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayStringOut::new(&mut i_cape_array_string_ptr); //CapeArrayStringOut from *mut C::ICapeArrayString
	/// assert_eq!(a.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
	/// ```

	fn as_cape_array_string_out(&mut self) -> C::ICapeArrayString {
		C::ICapeArrayString {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&Self::CAPE_ARRAY_STRING_VTABLE as *const C::ICapeArrayString_VTable).cast_mut(),
		}
	}
}

/// Vector based CapeArrayValueOut implementation
///
/// ICapeArrayValue is passed as data container between CAPE-OPEN functions. 
/// It is up to the caller to provide the interface, and its implementation. 
/// This class provides a default impementation using a `Vec<CapeValueImpl>`.
///
/// To manipulate the values, one could use the CapeArrayValueOut wrapper;
/// only a limited number of functions are implemented here.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_content(a: &mut CapeArrayStringOut) {
///     a.put_array(&["idealGasEnthalpy", "idealGasEntropy"]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayStringVec::new();
/// set_content(&mut CapeArrayStringOutFromProvider::from(&mut arr).as_cape_array_string_out());
/// assert_eq!(arr.as_string_vec(), vec!["idealGasEnthalpy".to_string(), "idealGasEntropy".to_string()]);
/// ```
pub type CapeArrayValueVec = CapeArrayObjectVec<CapeValueImpl,C::ICapeValue>;

impl CapeArrayValueVec {


	/// Initialize from CapeValueContent slice
	///
	/// Creates a new CapeArrayValueVec from or slice of CapeValueContent;
	/// note that the values will be cloned.
	///
	/// # Arguments
	///
	/// * `array` - A slice of values to converted to a CapeArrayValueVec
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Real(2.4),cobia::CapeValueContent::Boolean(true)]);
	/// assert_eq!(arr.as_value_vec(), vec![cobia::CapeValueContent::Real(2.4),cobia::CapeValueContent::Boolean(true)]);
	/// ```
	pub fn from_slice(array: &[CapeValueContent]) -> CapeArrayValueVec {
		let mut vec = Vec::new();
		let mut interface_vec = Vec::new();
		let mut interface_ptr_vec = Vec::new();
		vec.reserve(array.len());
		interface_vec.reserve(array.len());
		interface_ptr_vec.reserve(array.len());
		for val in array.iter() {
			vec.push(CapeValueImpl::from_content(val.clone()));
			interface_vec.push(vec.last_mut().unwrap().as_cape_value_out());
			interface_ptr_vec.push((interface_vec.last().unwrap() as *const C::ICapeValue).cast_mut());
		}
		Self {
			vec,
			interface_vec,
			interface_ptr_vec
		}
	}

	/// Return a value vector
	///
	/// Returns a vector of values from the CapeArrayValueVec.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Empty,cobia::CapeValueContent::Integer(4)]);
	/// let valvec = arr.as_value_vec();
	/// assert_eq!(valvec, vec![cobia::CapeValueContent::Empty,cobia::CapeValueContent::Integer(4)]);
	/// ```
	pub fn as_value_vec(&self) -> Vec<CapeValueContent> {
		let mut vec:Vec<CapeValueContent> = Vec::new();
		vec.reserve(self.vec.len());
		for val in self.vec.iter() {
			vec.push(val.value());
		}
		vec
	}

	/// Resize
	///
	/// Change the size of the vector.
	///
	/// # Arguments
	///
	/// * `size` - The new size of the vector
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Empty,cobia::CapeValueContent::Integer(4)]);
	/// arr.resize(3);
	/// assert_eq!(arr.size(), 3);
	/// ```
	pub fn resize(&mut self, size: usize) {
		let old_size = self.vec.len();
		let size = size as usize;
		if size < old_size {
			self.vec.truncate(size);
			self.interface_vec.truncate(size);
			self.interface_ptr_vec.truncate(size);
		} else {
			self.vec.reserve((size - old_size) as usize);
			self.interface_vec.reserve((size - old_size) as usize);
			for _ in old_size..size {
				self.vec.push(CapeValueImpl::new());
				self.interface_vec.push(self.vec.last_mut().unwrap().as_cape_value_out());
			}
			//vector may have been re-allocated, redo interfaces
			self.interface_ptr_vec.resize(size, std::ptr::null_mut());
			for i in old_size..size {
				self.interface_vec[i]=self.vec[i].as_cape_value_out();
				self.interface_ptr_vec[i]=(&self.interface_vec[i] as *const C::ICapeValue).cast_mut();
			}
		}
	}

	/// Set the content of the value array from any object that implements CapeArrayValueProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayValueProviderIn
	///
	/// # Example
	///
	/// ```
	/// use cobia;
	/// let mut arr = cobia::CapeArrayValueVec::new();
	/// let mut arr1 = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Empty,cobia::CapeValueContent::Integer(4)]);
	/// arr.set(&arr1);
	/// assert_eq!(arr.as_value_vec(), vec![cobia::CapeValueContent::Empty,cobia::CapeValueContent::Integer(4)]);
	/// ```
	pub fn set<T:CapeArrayValueProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut value_array_in_from_provider = CapeArrayValueInFromProvider::from(array);
		let value_array=value_array_in_from_provider.as_cape_array_value_in();
		self.resize(value_array.size());
		for i in 0..value_array.size() {
			self.vec[i].set(&value_array.at(i)?)?;
		}
		Ok(())
	}

	///interface member

	extern "C" fn get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *mut *mut C::ICapeValue,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut Self;
		let str_arr: &mut Self = unsafe { &mut *p };
		unsafe {
			*data = str_arr.interface_ptr_vec.as_ptr() as *mut *mut C::ICapeValue;
			*size = str_arr.interface_vec.len() as C::CapeSize;
		}
	}

	///interface member

	extern "C" fn setsize(
		me: *mut ::std::os::raw::c_void,
		size: C::CapeSize,
		data: *mut *mut *mut C::ICapeValue,
	) -> C::CapeResult {
		let p = me as *mut Self;
		let str_arr: &mut Self = unsafe { &mut *p };
		str_arr.resize(size as usize);
		unsafe {
			*data = str_arr.interface_ptr_vec.as_ptr() as *mut *mut C::ICapeValue;
		}
		COBIAERR_NOERROR
	}

	///interface v-table

	const CAPE_ARRAY_VALUE_VTABLE: C::ICapeArrayValue_VTable = C::ICapeArrayValue_VTable {
		get: Some(Self::get),
		setsize: Some(Self::setsize),
	};


}

impl CapeArrayValueProviderIn for CapeArrayValueVec {
	/// Convert to ICapeArrayValue
	///
	/// Returns a reference to the ICapeArrayValue interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayValueVec::from_slice(&vec![cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
	///	let i_cape_value=arr.as_cape_array_value_in();
	///	let mut i_cape_value_ptr=(&i_cape_value as *const C::ICapeArrayValue).cast_mut(); //normally a pointer to the interface is received
	///	let sa = cobia::CapeArrayValueIn::new(&mut i_cape_value_ptr); //CapeArrayValueIn from *mut C::ICapeArrayValue
	/// assert_eq!(sa.as_value_vec().unwrap(), vec![cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
	/// ```

	fn as_cape_array_value_in(&self) -> C::ICapeArrayValue {
		C::ICapeArrayValue {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&Self::CAPE_ARRAY_VALUE_VTABLE as *const C::ICapeArrayValue_VTable).cast_mut(),
		}
	}
}

impl CapeArrayValueProviderOut for CapeArrayValueVec {
	/// Convert to ICapeArrayValue
	///
	/// Returns a mutable reference to the ICapeArrayValue interface.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
	///	let i_cape_array_value=arr.as_cape_array_value_out();
	///	let mut i_cape_array_value_ptr=(&i_cape_array_value as *const C::ICapeArrayValue).cast_mut(); //normally a pointer to the interface is received
	///	let va = cobia::CapeArrayValueOut::new(&mut i_cape_array_value_ptr); //CapeArrayValueOut from *mut C::ICapeArrayValue
	/// assert_eq!(va.as_value_vec().unwrap(), vec![cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
	/// ```

	fn as_cape_array_value_out(&mut self) -> C::ICapeArrayValue {
		C::ICapeArrayValue {
			me: (self as *const Self).cast_mut() as *mut ::std::os::raw::c_void,
			vTbl: (&Self::CAPE_ARRAY_VALUE_VTABLE as *const C::ICapeArrayValue_VTable).cast_mut(),
		}
	}
}

impl<T: CapeArrayValueProviderIn> PartialEq<T> for CapeArrayValueVec {
	/// Partial equality
	///
	/// Checks if the content of the CapeArrayValueVec is equal to the content of another object that implements CapeArrayValueProviderIn.
	///
	/// # Arguments
	///
	/// * `other` - An object that implements CapeArrayValueProviderIn
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let arr1 = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
	/// let arr2 = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
	/// let arr3 = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Boolean(false),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
	/// assert!(arr1 == arr2);
	/// assert!(arr1 != arr3);
	/// ```
	fn eq(&self, other: &T) -> bool {
		let mut provider=CapeArrayValueInFromProvider::from(other);
		let other=provider.as_cape_array_value_in(); 
		if self.size() != other.size() {
			return false;
		}
		//compare the value vectors
		for (i, v) in self.vec.iter().enumerate() {
			match other.at(i) {
				Ok(v_other) => {
					if v != &v_other {
						return false;
					}
				},
				Err(_) => return false, //if we cannot get the value, they are not equal
			}
		}
		true
	}
}