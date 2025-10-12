use crate::{C,COBIAError,CapeArrayBooleanInFromProvider};
use crate::cape_array::{ArrayInterface,CapeArrayIn,CapeArrayOut};
use crate::{CapeArrayBooleanProviderIn,CapeArrayBooleanProviderOut};

impl ArrayInterface<C::CapeBoolean> for C::ICapeArrayBoolean {

	fn get(&mut self,data:&mut *mut C::CapeBoolean,size:&mut C::CapeSize) {
		unsafe { (*self.vTbl).get.unwrap()(self.me,data as *mut *mut C::CapeBoolean,size) };
	}

	fn set(&mut self,data:&mut *mut C::CapeBoolean,size:C::CapeSize) -> C::CapeResult {
		unsafe { (*self.vTbl).setsize.unwrap()(self.me,size,data as *mut *mut C::CapeBoolean) }
	}

	fn get_const(&self,data:&mut *const C::CapeBoolean,size:&mut C::CapeSize) {
		unsafe { (*self.vTbl).get.unwrap()(self.me,data as *mut *const C::CapeBoolean as *mut *mut C::CapeBoolean,size) };
	}
}

/// CapeArrayBooleanIn wraps an ICapeArrayBoolean interface pointer.
///
/// Given an ICapeArrayBoolean interface pointer, this allows setting
///  and getting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayBoolean input arguments.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayBooleanIn) {
///     assert_eq!(a.as_bool_vec(), vec![false,true,true,false]);
/// }
/// 
/// let arr = cobia::CapeArrayBooleanVec::from_bool_slice(&[false,true,true,false]);
/// test_content(&CapeArrayBooleanInFromProvider::from(&arr).as_cape_array_boolean_in())
/// ```

pub type CapeArrayBooleanIn<'a> = CapeArrayIn<'a,C::CapeBoolean,C::ICapeArrayBoolean>;

impl<'a> CapeArrayBooleanIn<'a> {

	/// Returns the elements of the array as a `Vec<bool>`.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_content(a: &CapeArrayBooleanIn) {
	///     assert_eq!(a.as_bool_vec(), vec![false,true,true,false]);
	/// }
	/// 
	/// let arr = cobia::CapeArrayBooleanVec::from_bool_slice(&[false,true,true,false]);
	/// test_content(&CapeArrayBooleanInFromProvider::from(&arr).as_cape_array_boolean_in())
	/// ```

	pub fn as_bool_vec(&self) -> Vec<bool> {
		let vec=self.as_vec();
		let mut v = Vec::with_capacity(vec.len());
		for (i,_) in vec.iter().enumerate() {
			v.push(vec[i]!=0);
		}
		v
	}

}

impl<'a> CapeArrayBooleanProviderIn for CapeArrayBooleanIn<'a> {
	fn as_cape_array_boolean_in(&self) -> C::ICapeArrayBoolean {
		unsafe{**self.interface}
	}
}

/// CapeArrayBooleanOut wraps an ICapeArrayBoolean interface pointer.
///
/// Given an ICapeArrayBoolean interface pointer, this allows setting
///  and getting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayBoolean output arguments.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_content(a: &mut CapeArrayBooleanOut) {
///		a.put_array(&[false as CapeBoolean,true as CapeBoolean,true as CapeBoolean,false as CapeBoolean]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayBooleanVec::new();
/// set_content(&mut CapeArrayBooleanOutFromProvider::from(&mut arr).as_cape_array_boolean_out());
/// assert_eq!(arr.as_vec(), &vec![false as CapeBoolean,true as CapeBoolean,true as CapeBoolean,false as CapeBoolean]);
/// ```

pub type CapeArrayBooleanOut<'a> = CapeArrayOut<'a,C::CapeBoolean,C::ICapeArrayBoolean>;

impl<'a> CapeArrayBooleanOut<'a> {

	/// Returns the elements of the array as a `Vec<bool>`.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_content(a: CapeArrayBooleanOut) {
	///     assert_eq!(a.as_bool_vec(), vec![false,true,true,false]);
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayBooleanVec::from_bool_slice(&[false,true,true,false]);
	/// test_content(CapeArrayBooleanOutFromProvider::from(&mut arr).as_cape_array_boolean_out())
	/// ```

	pub fn as_bool_vec(&self) -> Vec<bool> {
		let vec=self.as_vec();
		let mut v = Vec::with_capacity(vec.len());
		for (i,_) in vec.iter().enumerate() {
			v.push(vec[i]!=0);
		}
		v
	}

	/// Set the content of the boolean array from any object that implements CapeArrayBooleanProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayBooleanProviderIn
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// fn set_content(a: &mut CapeArrayBooleanOut, b: &CapeArrayBooleanIn) {
	///		a.put_array(&[false as CapeBoolean,true as CapeBoolean,true as CapeBoolean,false as CapeBoolean]).unwrap();
	/// }
	/// 
	/// let mut arr = cobia::CapeArrayBooleanVec::new();
	/// let arr1 = cobia::CapeArrayBooleanVec::from_bool_slice(&[false,true,true,false]);
	/// set_content(&mut CapeArrayBooleanOutFromProvider::from(&mut arr).as_cape_array_boolean_out(),&CapeArrayBooleanInFromProvider::from(&arr1).as_cape_array_boolean_in());
	/// assert_eq!(arr.as_vec(), &[false as CapeBoolean,true as CapeBoolean,true as CapeBoolean,false as CapeBoolean]);
	/// ```

	pub fn set<T:CapeArrayBooleanProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut boolean_array_in_from_provider = CapeArrayBooleanInFromProvider::from(array);
		let boolean_array=boolean_array_in_from_provider.as_cape_array_boolean_in();
		self.resize(boolean_array.size())?;
		for i in 0..boolean_array.size() {
			self[i]=boolean_array[i];
		}
		Ok(())
	}
}

impl<'a> CapeArrayBooleanProviderOut for CapeArrayBooleanOut<'a> {
	fn as_cape_array_boolean_out(&mut self) -> C::ICapeArrayBoolean {
		unsafe { **self.interface }
	}
}