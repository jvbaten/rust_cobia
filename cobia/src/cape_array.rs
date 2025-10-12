use std::fmt;
use crate::COBIAError;
use crate::C;
use std::marker::PhantomData;
use crate::cape_result_value::*;

/// Trait for common array functionality of any CapeArrayOut wrapper

pub(crate) trait ArrayInterface<Element> : Copy {
	fn get_const(&self,data:&mut *const Element,size:&mut C::CapeSize);
	fn get(&mut self,data:&mut *mut Element,size:&mut C::CapeSize);
	fn set(&mut self,data:&mut *mut Element,size:C::CapeSize) -> C::CapeResult;
}

/// CapeArrayIn wraps an ICapeArray interface pointer.
///
/// Given a reference to an ICapeArray interface pointer, this allows gettings,
/// but not setting, the elements.
///
/// This interface is typically used as input arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArray arguments.
///
/// This class takes a reference to the interface pointer. A NULL pointer
/// is treated as an empty array.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(arr: &CapeArrayRealIn) {
///     assert_eq!(arr.as_vec(), vec![4.6,8.6,1e-3]);
/// }
/// 
/// let arr = cobia::CapeArrayRealSlice::new(&[4.6,8.6,1e-3]);
/// test_content(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in())
/// ```

#[allow(private_bounds)]
pub struct CapeArrayIn<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> {
	data: *const Element,
	size: C::CapeSize,
	pub(crate) interface: &'a *mut Interface,
	_lifetime: PhantomData<&'a ()>, //even though we do not refer to the interace after contruction, life time is bound to the interface because of the data that is pointed to
	_interface_type : PhantomData<Interface>
}

#[allow(private_bounds)]
impl<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> CapeArrayIn<'a,Element,Interface> {
	/// Create a new CapeArrayIn from an ICapeArray interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeArray interface
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let arr = cobia::CapeArrayRealSlice::new(&[4.5,6.5]);
	///	let mut i_cape_array_real=arr.as_cape_array_real_in();
	///	let i_cape_array_real_ptr=(&i_cape_array_real as *const C::ICapeArrayReal).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayRealIn::new(&i_cape_array_real_ptr); //CapeArrayRealIn from *mut C::ICapeArrayReal
	/// assert_eq!(a.as_vec(), vec![4.5,6.5]);
	/// ```

	pub fn new(interface: &'a *mut Interface) -> Self {
		if interface.is_null() {
			Self {		
				data: std::ptr::null_mut(),
				size: 0,
				interface,
				_lifetime : std::default::Default::default(),
				_interface_type : std::default::Default::default()
			}
		} else {
			let mut data: *const Element = std::ptr::null_mut();
			let mut size: C::CapeSize = 0;
			unsafe { **interface }.get_const(&mut data,&mut size);
			Self {			
				data,
				size,
				interface,
				_lifetime : std::default::Default::default(),
				_interface_type : std::default::Default::default()
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
	/// fn test_size(arr: &CapeArrayRealIn) {
	///     assert_eq!(arr.size(), 3);
	/// }
	/// 
	/// let arr = cobia::CapeArrayRealVec::from_slice(&[3.5,5.5,8.2]);
	/// test_size(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in())
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
	/// fn test_not_empty(arr: &CapeArrayRealIn) {
	///    assert!(!arr.is_empty());
	/// }
	/// 
	/// let arr = cobia::CapeArrayRealVec::from_slice(&[3.5,5.5,8.2]);
	/// test_not_empty(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in())
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
	/// fn test_content(arr: &CapeArrayRealIn) {
	///     assert_eq!(arr.as_vec(), vec![1.2,1.4,1.6]);
	/// }
	/// 
	/// let arr = cobia::CapeArrayRealSlice::new(&[1.2,1.4,1.6]);
	/// test_content(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in())
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
	/// fn test_content(arr: &CapeArrayRealIn) {
	///     assert_eq!(arr.as_slice(), &[1.2,1.4,1.6]);
	/// }
	/// 
	/// let arr = cobia::CapeArrayRealVec::from_vec(vec![1.2,1.4,1.6]);
	/// test_content(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in())
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
	/// fn test_iter(a: &CapeArrayRealIn) {
	///		let mut iter = a.iter();
	///		assert_eq!(iter.next().unwrap(), 0.1);
	///		assert_eq!(iter.next().unwrap(), 0.2);
	///		assert!(!iter.next().is_some());
	/// }
	/// 
	/// let arr = cobia::CapeArrayRealVec::from_vec(vec![0.1,0.2]);
	/// test_iter(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in())
	/// ```

	pub fn iter(&self) -> CapeArrayRefIterator<'_,Element> {
		CapeArrayRefIterator {
			data: &self.as_slice(),
			index: 0
		}
	}

}


/// An iterator that consumes a CapeArrayIn
///
/// This struct is created by the IntoIterator trait on CapeArrayIn.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_sum(a: &CapeArrayRealIn) {
///		let mut sum=0.0;
///		for val in a {
///		    sum+=val;
///		}
///		assert!((sum-0.9).abs()<1e-12);
/// }
/// 
/// let arr = cobia::CapeArrayRealVec::from_vec(vec![0.3,0.6]);
/// test_sum(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in())
/// ```

#[allow(private_bounds)]
pub struct CapeArrayInIterator<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> {
	arr: CapeArrayIn<'a,Element,Interface>,
	index: usize
}

impl<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> Iterator for CapeArrayInIterator<'a,Element, Interface> {
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

impl<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> std::ops::Index<usize> for CapeArrayIn<'a,Element,Interface> {

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
	/// ```
	/// use cobia::*;
	///
	/// fn test_element(arr: &CapeArrayRealIn) {
	///     assert_eq!(arr[1], 10.2);
	/// }
	/// 
	/// let arr = cobia::CapeArrayRealSlice::new(&[10.1,10.2,10.3]);
	/// test_element(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in())
	/// ```

	fn index(&self, index: usize) -> &Self::Output {
		if index>=(self.size as usize) {
			panic!("index out of bounds");
		}
		unsafe { &*self.data.add(index) as &Element }
	}

}

impl<'a,Element:Copy+Clone+std::fmt::Display,Interface:ArrayInterface<Element>> fmt::Display for CapeArrayIn<'a,Element,Interface> {

	/// Display the content of the real array as a real vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_format(a : &CapeArrayRealIn) {
	///     assert_eq!(format!("{}", a), "[1, 1.1, 1.2]");
	/// }
	/// 
	/// let arr = cobia::CapeArrayRealVec::from_vec(vec![1.0,1.1,1.2]);
	/// test_format(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in())
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


impl<'a,Element:Copy+Clone+'a,Interface:ArrayInterface<Element>> IntoIterator for &'a CapeArrayIn<'a,Element,Interface> {
	type Item = Element;
	type IntoIter = CapeArrayRefIterator<'a, Element>;

	/// Return an iterator over the real array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: &CapeArrayRealIn) {
	///		 let mut sum=0.0;
	///		 for val in a {
	///		     sum+=val;
	///		 }
	///		 assert!((sum-0.9).abs()<1e-12);
	/// }
	/// 
	/// let arr = cobia::CapeArrayRealVec::from_vec(vec![0.3,0.6]);
	/// test_iter(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in())
	/// ```

	fn into_iter(self) -> CapeArrayRefIterator<'a, Element> {
		CapeArrayRefIterator {
			data: self.as_slice(),
			index: 0,
		}
	}
}

impl<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> IntoIterator for CapeArrayIn<'a,Element,Interface> {
	type Item = Element;
	type IntoIter = CapeArrayInIterator<'a,Element,Interface>;

	/// Return an iterator over the real array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_iter(a: &CapeArrayRealIn) {
	///		let mut sum=0.0;
	///		for val in a {
	///		    sum+=val;
	///		}
	///		assert!((sum-0.9).abs()<1e-12);
	/// }
	/// 
	/// let arr = cobia::CapeArrayRealVec::from_vec(vec![0.3,0.6]);
	/// test_iter(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in())
	/// ```

	fn into_iter(self) -> Self::IntoIter {
		CapeArrayInIterator {
			arr: self,
			index: 0,
		}
	}
}

/// CapeArrayOut wraps an ICapeArray interface pointer.
///
/// Given a reference to an ICapeArray interface reference, this allows setting
///  and getting the elements.
///
/// This interface is typically used as output arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArray arguments.
///
/// This class takes a mutable reference to the interface, as
/// it should be the only class that is in use at a time to change the 
/// data behind the interface (as the data pointer is cached)
///
/// NULL pointers are not allowed here
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
pub struct CapeArrayOut<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> {
	pub(crate) interface: &'a mut *mut Interface,
	data: *mut Element,
	size: C::CapeSize,
}

#[allow(private_bounds)]
impl<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> CapeArrayOut<'a,Element,Interface> {
	/// Create a new CapeArrayOut from an ICapeArray interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeArray interface
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[4.6,8.6,1e-3]);
	///	let i_cape_array_real=arr.as_cape_array_real_out();
	///	let mut i_cape_array_real_ptr=(&i_cape_array_real as *const C::ICapeArrayReal).cast_mut(); //normally a pointer to the interface is received
	///	let a = cobia::CapeArrayRealOut::new(&mut i_cape_array_real_ptr); //CapeArrayRealOut from *mut C::ICapeArrayReal
	/// assert_eq!(a.as_vec(), vec![4.6,8.6,1e-3]);
	/// ```

	pub fn new(interface: &'a mut *mut Interface) -> Self {
		if (*interface).is_null() {
			panic!("NULL pointer not allowed");
		}
		let mut data: *mut Element = std::ptr::null_mut();
		let mut size: C::CapeSize = 0;
		unsafe {**interface}.get(&mut data,&mut size);
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
	/// fn test_content(arr: &mut CapeArrayRealOut) {
	///		assert_eq!(arr.size(), 3);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[3.5,5.5,8.2]);
	/// test_content(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
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
	/// fn check_not_empty(arr: &mut CapeArrayRealOut) {
	///		assert!(!arr.is_empty());
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[3.5,5.5,8.2]);
	/// check_not_empty(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
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
	/// fn check_content(a: &mut CapeArrayRealOut) {
	///		assert_eq!(a.as_vec(), vec![1.2,1.4,1.6]);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[1.2,1.4,1.6]);
	/// check_content(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
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
	/// fn set_content(arr: &mut CapeArrayRealOut) {
	///		arr.put_array(&[3.1,3.2,3.3,3.4]).unwrap();
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[4.5,6.5]);
	/// set_content(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
	/// assert_eq!(arr.as_vec(), &vec![3.1,3.2,3.3,3.4]); //the values have been stored on the object that implements ICapeArrayReal
	/// ```

	pub fn put_array(&mut self, array: &[Element]) -> Result<(), COBIAError> {
		let res=unsafe {**self.interface}.set(&mut self.data,array.len() as C::CapeSize);
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
	/// fn resize_array(a: &mut CapeArrayRealOut) {
	///		a.resize(4).unwrap();
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::new();
	/// resize_array(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
	/// assert_eq!(arr.size(), 4); 

	pub fn resize(&mut self, size: usize) -> Result<(), COBIAError> {
		let res=unsafe{**self.interface}.set(&mut self.data,size as C::CapeSize);
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
	/// fn check_content(a: &mut CapeArrayRealOut) {
	///		assert_eq!(a.as_slice(), &[1.2,1.4,1.6])
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[1.2,1.4,1.6]);
	/// check_content(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
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
	/// fn check_iter(a: &mut CapeArrayRealOut) {
	///		 let mut iter = a.iter();
	///		 assert_eq!(iter.next().unwrap(), 0.1);
	///		 assert_eq!(iter.next().unwrap(), 0.2);
	///		 assert!(!iter.next().is_some());
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::from_vec(vec![0.1,0.2]);
	/// check_iter(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
	/// ```

	pub fn iter(&self) -> CapeArrayRefIterator<'_,Element> {
		CapeArrayRefIterator {
			data: &self.as_slice(),
			index: 0
		}
	}

}


/// An iterator that takes a reference to the data in CapeArrayIn or CapeArrayOut
///
/// This struct is created by the iter method on CapeArrayIn and CapeArrayOut as well as by the IntoInterator trait on &CapeArrayOut.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn check_iter(a: CapeArrayRealOut) {
///		let mut sum=0.0;
///		for val in a {
///		    sum+=val;
///		}
///		assert!((sum-0.9).abs()<1e-12);
/// }
/// 
/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[0.3,0.6]);
/// check_iter(CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
/// ```

pub struct CapeArrayRefIterator<'a, Element:Copy+Clone> {
	pub(crate) data: &'a[Element],
	pub(crate) index: usize
}

impl<'a, Element:Copy+Clone> Iterator for CapeArrayRefIterator<'a, Element> {
	type Item = Element;
	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.data.len() {
			let res = self.data[self.index];
			self.index += 1;
			Some(res)
		} else {
			None
		}
	}
}

/// An iterator that consumes a CapeArrayOut
///
/// This struct is created by the IntoIterator trait on CapeArrayOut.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn check_iter(a: CapeArrayRealOut) {
///		let mut sum=0.0;
///		for val in a {
///		    sum+=val;
///		}
///		assert!((sum-0.9).abs()<1e-12);
/// }
/// 
/// let mut arr = cobia::CapeArrayRealVec::from_vec(vec![0.3,0.6]);
/// check_iter(CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
/// ```

#[allow(private_bounds)]
pub struct CapeArrayOutIterator<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> {
	arr: CapeArrayOut<'a,Element,Interface>,
	index: usize
}

impl<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> Iterator for CapeArrayOutIterator<'a,Element, Interface> {
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

impl<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> std::ops::Index<usize> for CapeArrayOut<'a,Element,Interface> {

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
	/// ```
	/// use cobia::*;
	///
	/// fn check_item(a: &mut CapeArrayRealOut) {
	///		 assert_eq!(a[1], 10.2);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[10.1,10.2,10.3]);
	/// check_item(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
	/// ```

	fn index(&self, index: usize) -> &Self::Output {
		if index>=(self.size as usize) {
			panic!("index out of bounds");
		}
		unsafe { &*self.data.add(index) as &Element }
	}

}

impl<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> std::ops::IndexMut<usize> for CapeArrayOut<'a,Element,Interface> {
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
	/// fn set_item(a: &mut CapeArrayRealOut) {
	///		a[1]=5.3;
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::from_slice(&[10.1,10.2,10.3]);
	/// set_item(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
	/// assert_eq!(arr.as_vec(), &vec![10.1,5.3,10.3]);
	/// ```

	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		if index>=(self.size as usize) {
			panic!("index out of bounds");
		}
		unsafe { &mut *self.data.add(index) as &mut Element }
	}
}


impl<'a,Element:Copy+Clone+std::fmt::Display,Interface:ArrayInterface<Element>> fmt::Display for CapeArrayOut<'a,Element,Interface> {

	/// Display the content of the real array as a real vector.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_format(a: &mut CapeArrayRealOut) {
	///		 assert_eq!(format!("{}", a), "[1, 1.1, 1.2]");
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::from_vec(vec![1.0,1.1,1.2]);
	/// check_format(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
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


impl<'a,Element:Copy+Clone+'a,Interface:ArrayInterface<Element>> IntoIterator for &'a CapeArrayOut<'a,Element,Interface> {
	type Item = Element;
	type IntoIter = CapeArrayRefIterator<'a, Element>;

	/// Return an iterator over the real array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_iter(a: CapeArrayRealOut) {
	///		let mut sum=0.0;
	///		for val in &a {
	///		    sum+=val;
	///		}
	///		assert!((sum-0.9).abs()<1e-12);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::from_vec(vec![0.3,0.6]);
	/// check_iter(CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
	/// ```

	fn into_iter(self) -> CapeArrayRefIterator<'a, Element> {
		CapeArrayRefIterator {
			data: self.as_slice(),
			index: 0,
		}
	}
}

impl<'a,Element:Copy+Clone,Interface:ArrayInterface<Element>> IntoIterator for CapeArrayOut<'a,Element,Interface> {
	type Item = Element;
	type IntoIter = CapeArrayOutIterator<'a,Element,Interface>;

	/// Return an iterator over the real array.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_iter(a: CapeArrayRealOut) {
	///		let mut sum=0.0;
	///		for val in a {
	///		    sum+=val;
	///		}
	///		assert!((sum-0.9).abs()<1e-12);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayRealVec::from_vec(vec![0.3,0.6]);
	/// check_iter(CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
	/// ```

	fn into_iter(self) -> Self::IntoIter {
		CapeArrayOutIterator {
			arr: self,
			index: 0,
		}
	}
}
