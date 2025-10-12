pub use crate::*;
use std::fmt;
use std::marker::PhantomData;
use crate::C;
use crate::{CapeArrayValueProviderIn,CapeArrayValueProviderOut};

/// CapeArrayValueIn wraps an ICapeArrayValue interface pointer in a read-only manner
///
/// Given a reference to an ICapeArrayValue interface pointer, this allows getting,
/// but not setting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayValue input arguments.
///
/// A NULL interface pointer is treated as an empty array.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayValueIn) {
///		assert_eq!(a.as_value_vec().unwrap(), vec![cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
/// }
/// 
/// let arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
/// test_content(&CapeArrayValueInFromProvider::from(&arr).as_cape_array_value_in());
/// ```

pub struct CapeArrayValueIn<'a> {
	data: *mut *mut C::ICapeValue,
	size: C::CapeSize,
	interface: &'a *mut C::ICapeArrayValue,
	_lifetime: PhantomData<&'a ()> //even though we do not refer to the interace after contruction, life time is bound to the interface, as each of the elements are
}

impl<'a> CapeArrayValueIn<'a> {
	/// Create a new CapeValueIn from an ICapeArrayValue interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeArrayValue interface
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
	///	let i_cape_array_value=arr.as_cape_array_value_in();
	///	let mut i_cape_array_value_ptr=(&i_cape_array_value as *const C::ICapeArrayValue).cast_mut(); //normally a pointer to the interface is received
	///	let va = cobia::CapeArrayValueIn::new(&mut i_cape_array_value_ptr); //CapeArrayValueIn from *mut C::ICapeArrayValue
	/// assert_eq!(va.as_value_vec().unwrap(), vec![cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
	/// ```

	pub fn new(interface: &'a *mut C::ICapeArrayValue) -> CapeArrayValueIn<'a> {
		if interface.is_null() {
			CapeArrayValueIn {
				data : std::ptr::null_mut(),
				size : 0,
				interface,
				_lifetime : std::default::Default::default()
			}
		} else {
			let mut data: *mut *mut C::ICapeValue = std::ptr::null_mut();
			let mut size: C::CapeSize = 0;
			unsafe { (*(**interface).vTbl).get.unwrap()((**interface).me, &mut data, &mut size) };
			CapeArrayValueIn {
				data,
				size,
				interface,
				_lifetime : std::default::Default::default()
			}
		}
	}

	/// Return the size of the array
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_size(a: &CapeArrayValueIn) {
	///		assert_eq!(a.size(), 3);
	/// }
	/// 
	/// let arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
	/// test_size(&CapeArrayValueInFromProvider::from(&arr).as_cape_array_value_in());
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
	/// fn test_empty(a: &CapeArrayValueIn) {
	///		assert!(a.is_empty());
	/// }
	/// 
	/// let arr = cobia::CapeArrayValueVec::new();
	/// test_empty(&CapeArrayValueInFromProvider::from(&arr).as_cape_array_value_in());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.size == 0
	}

	/// Get an element
	///
	/// # Arguments
	///
	/// * `index` - The index of the element to get
	/// 
	/// Note that neither Index and IndexMut is 
	/// provided for CapeArrayValueIn, because these interfaces
	/// yield a reference to the element, whereas the elements
	/// of CapeArrayValueIn are represented by an interface,
	/// which is conveniently wrapped into CapeValueIn. 
	/// 
	/// Note that the life time of the CapeValueIn is tied to the
	/// life time of the CapeArrayValueIn.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_size(a: &CapeArrayValueIn) {
	///		assert_eq!(a.at(1).unwrap().get_boolean().unwrap(), true);
	/// }
	/// 
	/// let arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Real(2.4),cobia::CapeValueContent::Boolean(true)]);
	/// test_size(&CapeArrayValueInFromProvider::from(&arr).as_cape_array_value_in());
	/// ```

	pub fn at(&self, index: usize) -> Result<CapeValueIn<'a>, COBIAError> {
		if index >= self.size as usize {
			return Err(COBIAError::Code(COBIAERR_NOSUCHITEM));
		}
		let p=unsafe { self.data.add(index) };
		if unsafe{*p}.is_null() {
			//this provided by the implementor of ICapeArrayValue and should not be null
			Err(COBIAError::Code(COBIAERR_NULLPOINTER))
		} else {
			Ok(CapeValueIn::new(unsafe { &mut *p }))
		}
	}

	/// Return the content of the value array as a vector of CapeValueContent
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_content(a: &CapeArrayValueIn) {
	///		assert_eq!(a.as_value_vec().unwrap(), vec![cobia::CapeValueContent::Real(2.4),cobia::CapeValueContent::Boolean(true)]);
	/// }
	/// 
	/// let arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Real(2.4),cobia::CapeValueContent::Boolean(true)]);
	/// test_content(&CapeArrayValueInFromProvider::from(&arr).as_cape_array_value_in());
	/// ```

	pub fn as_value_vec(&self) -> Result<Vec<CapeValueContent>, COBIAError> {
		let mut vec= Vec::new();
		vec.reserve(self.size as usize);
		for i in 0..self.size {
			let p=unsafe { *self.data.add(i as usize) };
			if p.is_null() {
				//this provided by the implementor of ICapeArrayValue and should not be null
				vec.push(CapeValueContent::Empty);
			} else {
				let val=CapeValueIn::new(&p);
				match val.get_type() {
					Ok(tp)=>
						match tp {
							CapeValueType::String => vec.push(CapeValueContent::String(
								match val.get_string() {
									Ok(s)=>s,
									Err(e)=> {return Err(e);}
								})),
							CapeValueType::Integer => vec.push(CapeValueContent::Integer(
								match val.get_integer() {
									Ok(i)=>i,
									Err(e)=> {return Err(e);}
								})),
							CapeValueType::Boolean => vec.push(CapeValueContent::Boolean(
								match val.get_boolean() {
									Ok(b)=>b,
									Err(e)=> {return Err(e);}
								})),
							CapeValueType::Real => vec.push(CapeValueContent::Real(
								match val.get_real() {
									Ok(r)=>r,
									Err(e)=> {return Err(e);}
								})),
							CapeValueType::Empty => vec.push(CapeValueContent::Empty),
						},
					Err(e)=> {return Err(e);}
				}
			}
		}
		Ok(vec)
	}

}

pub struct CapeArrayValueInIterator<'a> {
	arr: &'a CapeArrayValueIn<'a>,
	index: usize,
}

impl<'a> Iterator for CapeArrayValueInIterator<'a> {
	type Item = CapeValueIn<'a>;

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

impl<'a> CapeArrayValueIn<'a> {
	/// Return an iterator over the value array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: &CapeArrayValueIn) {
	///		let mut iter = a.iter();
	///		assert_eq!(iter.next().unwrap().get_integer().unwrap(), 4);
	///		assert_eq!(iter.next().unwrap().get_boolean().unwrap(), true);
	///		assert!(!iter.next().is_some());
	/// }
	/// 
	/// let arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Integer(4),cobia::CapeValueContent::Boolean(true)]);
	/// test_iter(&CapeArrayValueInFromProvider::from(&arr).as_cape_array_value_in());
	/// ```

	pub fn iter(&self) -> CapeArrayValueInIterator<'_> {
		CapeArrayValueInIterator {
			arr: &self,
			index: 0,
		}
	}
}

impl<'a> fmt::Display for CapeArrayValueIn<'a> {
	/// Display the content of the value array as a vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_format(a: &CapeArrayValueIn) {
	///		assert_eq!(format!("{}", a), "[4, true, <empty>, \"H2O\"]");
	/// }
	/// 
	/// let arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Integer(4),cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Empty,cobia::CapeValueContent::String("H2O".to_string())]);
	/// test_format(&CapeArrayValueInFromProvider::from(&arr).as_cape_array_value_in());
	/// ```

	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "[")?;
		for (count, v) in self.iter().enumerate() {
			if count != 0 {
				write!(f, ", ")?;
			}
			write!(f, "{}", v)?;
		}
		write!(f, "]")
	}
}

impl<'a> CapeArrayValueProviderIn for CapeArrayValueIn<'a> {
	fn as_cape_array_value_in(&self) -> C::ICapeArrayValue {
		unsafe { **self.interface }
	}
}

/// CapeArrayValueOut wraps an ICapeArrayValue interface pointer.
///
/// Given a reference to an ICapeArrayValue interface pointer, this allows setting
///  and getting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayValue output arguments.
///
/// NULL interface pointers are not allowed.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_content(a: &mut CapeArrayValueOut) {
///		a.put_array(&[cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayValueVec::new();
/// set_content(&mut CapeArrayValueOutFromProvider::from(&mut arr).as_cape_array_value_out());
/// assert_eq!(arr.as_value_vec(), vec![cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
/// ```

pub struct CapeArrayValueOut<'a> {
	interface: &'a mut *mut C::ICapeArrayValue,
	data: *mut *mut C::ICapeValue,
	size: C::CapeSize,
}

impl<'a> CapeArrayValueOut<'a> {
	/// Create a new CapeValueOut from an ICapeArrayValue interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeArrayValue interface
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

	pub fn new(interface: &'a mut *mut C::ICapeArrayValue) -> CapeArrayValueOut<'a> {
		let mut data: *mut *mut C::ICapeValue = std::ptr::null_mut();
		let mut size: C::CapeSize = 0;
		unsafe { (*(**interface).vTbl).get.unwrap()((**interface).me, &mut data, &mut size) };
		CapeArrayValueOut {
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
	/// fn check_size(a: &mut CapeArrayValueOut) {
	///		assert_eq!(a.size(), 3);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Boolean(true),cobia::CapeValueContent::Real(1.2),cobia::CapeValueContent::Empty]);
	/// check_size(&mut CapeArrayValueOutFromProvider::from(&mut arr).as_cape_array_value_out());
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
	/// fn check_empty(a: &mut CapeArrayValueOut) {
	///		assert!(a.is_empty());
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayValueVec::new();
	/// check_empty(&mut CapeArrayValueOutFromProvider::from(&mut arr).as_cape_array_value_out());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.size == 0
	}

	/// Set the content from a slice of CapeValueContent
	///
	/// # Arguments
	///
	/// * `arr` - A slice of CapeValueContent
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_content(a: &mut CapeArrayValueOut) {
	///		a.put_array(&[cobia::CapeValueContent::String("methane".to_string()),cobia::CapeValueContent::Empty,cobia::CapeValueContent::Boolean(true)]).unwrap();
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayValueVec::new();
	/// set_content(&mut CapeArrayValueOutFromProvider::from(&mut arr).as_cape_array_value_out());
	/// assert_eq!(arr.as_value_vec(), [cobia::CapeValueContent::String("methane".to_string()),cobia::CapeValueContent::Empty,cobia::CapeValueContent::Boolean(true)]); //the values have been stored on the object that implements ICapeArrayValue
	/// ```

	pub fn put_array(&mut self, array: &[CapeValueContent]) -> Result<(), COBIAError> {
		let mut data: *mut *mut C::ICapeValue=std::ptr::null_mut();
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
					//this provided by the implementor of ICapeArrayValue and should not be null
					return Err(COBIAError::Code(COBIAERR_NULLPOINTER));
				}
				let el = CapeValueOut::new(unsafe { &mut *p });
				match s {
					CapeValueContent::Empty => el.set_empty()?,
					CapeValueContent::String(s) => el.set_string(s)?,
					CapeValueContent::Integer(i) => el.set_integer(*i)?,
					CapeValueContent::Boolean(b) => el.set_boolean(*b)?,
					CapeValueContent::Real(r) => el.set_real(*r)?,
				}
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
	/// provided for CapeArrayValueOut, because these interfaces
	/// yield a reference to the element, whereas the elements
	/// of CapeArrayValueOut are represented by an interface,
	/// which is conveniently wrapped into CapeValueOut. 
	/// 
	/// Note that the life time of the CapeValueOut is tied to the
	/// life time of the CapeArrayValueOut.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_element(a: &mut CapeArrayValueOut) {
	///		assert_eq!(a.at(2).unwrap().get_boolean().unwrap(), true);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayValueVec::from_slice(&vec![cobia::CapeValueContent::String("methane".to_string()),cobia::CapeValueContent::Empty,cobia::CapeValueContent::Boolean(true)]);
	/// test_element(&mut CapeArrayValueOutFromProvider::from(&mut arr).as_cape_array_value_out());
	/// assert_eq!(arr.as_value_vec(), [cobia::CapeValueContent::String("methane".to_string()),cobia::CapeValueContent::Empty,cobia::CapeValueContent::Boolean(true)]); //the values have been stored on the object that implements ICapeArrayValue
	/// ```

	pub fn at(&self, index: usize) -> Result<CapeValueOut<'a>, COBIAError> {
		if index >= self.size as usize {
			return Err(COBIAError::Code(COBIAERR_NOSUCHITEM));
		}
		let p=unsafe { self.data.add(index) };
		if p.is_null() {
			//this provided by the implementor of ICapeArrayValue and should not be null
			Err(COBIAError::Code(COBIAERR_NULLPOINTER))
		} else {
			Ok(CapeValueOut::new(unsafe { &mut *p} ))
		}
	}

	/// Return the content of the value array as a vector of CapeValueContent
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_content(a: &mut CapeArrayValueOut) {
	///		assert_eq!(a.as_value_vec().unwrap(), vec![cobia::CapeValueContent::Real(2.4),cobia::CapeValueContent::Boolean(true)]);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Real(2.4),cobia::CapeValueContent::Boolean(true)]);
	/// test_content(&mut CapeArrayValueOutFromProvider::from(&mut arr).as_cape_array_value_out());
	/// ```

	pub fn as_value_vec(&self) -> Result<Vec<CapeValueContent>, COBIAError> {
		let mut vec= Vec::new();
		vec.reserve(self.size as usize);
		for i in 0..self.size {
			let p=unsafe { *self.data.add(i as usize) };
			if p.is_null() {
				//this provided by the implementor of ICapeArrayValue and should not be null
				vec.push(CapeValueContent::Empty);
			} else {
				let val=CapeValueIn::new(&p);
				match val.get_type() {
					Ok(tp)=>
						match tp {
							CapeValueType::String => vec.push(CapeValueContent::String(
								match val.get_string() {
									Ok(s)=>s,
									Err(e)=> {return Err(e);}
								})),
							CapeValueType::Integer => vec.push(CapeValueContent::Integer(
								match val.get_integer() {
									Ok(i)=>i,
									Err(e)=> {return Err(e);}
								})),
							CapeValueType::Boolean => vec.push(CapeValueContent::Boolean(
								match val.get_boolean() {
									Ok(b)=>b,
									Err(e)=> {return Err(e);}
								})),
							CapeValueType::Real => vec.push(CapeValueContent::Real(
								match val.get_real() {
									Ok(r)=>r,
									Err(e)=> {return Err(e);}
								})),
							CapeValueType::Empty => vec.push(CapeValueContent::Empty),
						},
					Err(e)=> {return Err(e);}
				}
			}
		}
		Ok(vec)
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
	/// fn set_content(a: &mut CapeArrayValueOut) {
	///		a.resize(3).unwrap();
	///		a.at(0).unwrap().set_string("idealGasEnthalpy").unwrap();
	///		a.at(1).unwrap().set_real(2.4).unwrap();
	///		a.at(2).unwrap().set_integer(0).unwrap();
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayValueVec::new();
	/// set_content(&mut CapeArrayValueOutFromProvider::from(&mut arr).as_cape_array_value_out());
	/// assert_eq!(arr.as_value_vec(), vec![cobia::CapeValueContent::String("idealGasEnthalpy".to_string()),cobia::CapeValueContent::Real(2.4),cobia::CapeValueContent::Integer(0)]); //the values have been stored on the object that implements ICapeArrayString
	/// ```
	pub fn resize(&mut self, size: usize) -> Result<(), COBIAError> {
		let mut data:*mut *mut C::ICapeValue=std::ptr::null_mut();
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
	/// fn set_element(a: &mut CapeArrayValueOut) {
	///		a.put_value(1, cobia::CapeValueContent::String("density".to_string())).unwrap();
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::String("methane".to_string()),cobia::CapeValueContent::Empty,cobia::CapeValueContent::Boolean(true)]);
	/// set_element(&mut CapeArrayValueOutFromProvider::from(&mut arr).as_cape_array_value_out());
	/// assert_eq!(arr.as_value_vec(), [cobia::CapeValueContent::String("methane".to_string()),cobia::CapeValueContent::String("density".to_string()),cobia::CapeValueContent::Boolean(true)]);
	/// ```
	pub fn put_value(&mut self, index: usize, value: CapeValueContent) -> Result<(), COBIAError> {
		if index >= self.size as usize {
			return Err(COBIAError::Code(COBIAERR_NOSUCHITEM));
		}
		let p=unsafe { self.data.add(index) };
		if p.is_null() {
			//this provided by the implementor of ICapeArrayValue and should not be null
			return Err(COBIAError::Code(COBIAERR_NULLPOINTER));
		}
		let el = CapeValueOut::new(unsafe { &mut *p });
		match value {
			CapeValueContent::Empty => el.set_empty(),
			CapeValueContent::String(value) => el.set_string(&value),
			CapeValueContent::Integer(value) => el.set_integer(value),
			CapeValueContent::Boolean(value) => el.set_boolean(value),
			CapeValueContent::Real(value) => el.set_real(value),
		}
	}

	/// Set the content of the value array from any object that implements CapeArrayValueProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayValueProviderIn
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut arr = cobia::CapeArrayValueVec::new();
	/// let mut arr1 = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::Real(2.4),cobia::CapeValueContent::Boolean(true)]);
	/// CapeArrayValueOutFromProvider::from(&mut arr).as_cape_array_value_out().set(&arr1);
	/// assert_eq!(arr.as_value_vec(), vec![cobia::CapeValueContent::Real(2.4),cobia::CapeValueContent::Boolean(true)]);
	/// ```

	pub fn set<T:CapeArrayValueProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut value_array_in_from_provider = CapeArrayValueInFromProvider::from(array);
		let value_array=value_array_in_from_provider.as_cape_array_value_in();
		self.resize(value_array.size())?;
		for i in 0..value_array.size() {
			self.at(i)?.set(&value_array.at(i)?)?;
		}
		Ok(())
	}

}

pub struct CapeArrayValueIterator<'a> {
	arr: &'a CapeArrayValueOut<'a>,
	index: usize,
}

impl<'a> Iterator for CapeArrayValueIterator<'a> {
	type Item = CapeValueOut<'a>;

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

impl<'a> CapeArrayValueOut<'a> {
	/// Return an iterator over the value array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_iter(a: &mut CapeArrayValueOut) {
	///		let mut iter = a.iter();
	///		assert_eq!(iter.next().unwrap().get_string().unwrap(), "methane".to_string());
	///		assert_eq!(iter.next().unwrap().get_type().unwrap(), cobia::CapeValueType::Empty);
	///		assert_eq!(iter.next().unwrap().get_boolean().unwrap(), true);
	///		assert!(!iter.next().is_some());
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::String("methane".to_string()),cobia::CapeValueContent::Empty,cobia::CapeValueContent::Boolean(true)]);
	/// check_iter(&mut CapeArrayValueOutFromProvider::from(&mut arr).as_cape_array_value_out());
	/// ```

	pub fn iter(&self) -> CapeArrayValueIterator<'_> {
		CapeArrayValueIterator {
			arr: &self,
			index: 0,
		}
	}
}

impl<'a> fmt::Display for CapeArrayValueOut<'a> {
	/// Display the content of the value array as a value vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_format(a: &mut CapeArrayValueOut) {
	///		assert_eq!(format!("{}", a), "[\"methane\", <empty>, true]");
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayValueVec::from_slice(&[cobia::CapeValueContent::String("methane".to_string()),cobia::CapeValueContent::Empty,cobia::CapeValueContent::Boolean(true)]);
	/// check_format(&mut CapeArrayValueOutFromProvider::from(&mut arr).as_cape_array_value_out());
	/// ```

	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "[")?;
		for (count, v) in self.iter().enumerate() {
			if count != 0 {
				write!(f, ", ")?;
			}
			write!(f, "{}", v)?;
		}
		write!(f, "]")
	}
}

impl<'a> CapeArrayValueProviderOut for CapeArrayValueOut<'a> {
	fn as_cape_array_value_out(&mut self) -> C::ICapeArrayValue {
		unsafe { **self.interface }
	}
}