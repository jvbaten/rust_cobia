use crate::{COBIAError, CapeArrayIntegerInFromProvider, C};
use crate::cape_array::{ArrayInterface,CapeArrayIn,CapeArrayOut};
use crate::{CapeArrayIntegerProviderIn,CapeArrayIntegerProviderOut};

impl ArrayInterface<C::CapeInteger> for C::ICapeArrayInteger {

	fn get(&mut self,data:&mut *mut C::CapeInteger,size:&mut C::CapeSize) {
		unsafe { (*self.vTbl).get.unwrap()(self.me,data as *mut *mut C::CapeInteger,size) };
	}

	fn set(&mut self,data:&mut *mut C::CapeInteger,size:C::CapeSize) -> C::CapeResult {
		unsafe { (*self.vTbl).setsize.unwrap()(self.me,size,data as *mut *mut C::CapeInteger) }
	}

	fn get_const(&self,data:&mut *const C::CapeInteger,size:&mut C::CapeSize) {
		unsafe { (*self.vTbl).get.unwrap()(self.me,data as *mut *const C::CapeInteger as *mut *mut C::CapeInteger,size) };
	}

}

/// CapeArrayIntegerIn wraps an ICapeArrayInteger interface pointer.
///
/// Given an ICapeArrayInteger interface pointer, this allows setting
///  and getting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayInteger input arguments.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayIntegerIn) {
///     assert_eq!(a.as_vec(), vec![2,8,10]);
/// }
/// 
/// let arr = cobia::CapeArrayIntegerVec::from_slice(&[2,8,10]);
/// test_content(&CapeArrayIntegerInFromProvider::from(&arr).as_cape_array_integer_in())
/// ```

pub type CapeArrayIntegerIn<'a> = CapeArrayIn<'a,C::CapeInteger,C::ICapeArrayInteger>;

impl<'a> CapeArrayIntegerProviderIn for CapeArrayIntegerIn<'a> {
	fn as_cape_array_integer_in(&self) -> C::ICapeArrayInteger {
		unsafe { **self.interface }
	}
}

/// CapeArrayIntegerOut wraps an ICapeArrayInteger interface pointer.
///
/// Given an ICapeArrayInteger interface pointer, this allows setting
///  and getting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayInteger output arguments.
///
/// # Examples
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

pub type CapeArrayIntegerOut<'a> = CapeArrayOut<'a,C::CapeInteger,C::ICapeArrayInteger>;

impl<'a> CapeArrayIntegerOut<'a> {

	/// Set the content of the integer array from any object that implements CapeArrayIntegerProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayIntegerProviderIn
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut arr = cobia::CapeArrayIntegerVec::new();
	/// let mut arr1 = cobia::CapeArrayIntegerVec::from_slice(&vec![17,14,9,0]);
	/// CapeArrayIntegerOutFromProvider::from(&mut arr).as_cape_array_integer_out().set(&arr1);
	/// assert_eq!(arr.as_vec(), &[17,14,9,0]);
	/// ```

	pub fn set<T:CapeArrayIntegerProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut integer_array_in_from_provider = CapeArrayIntegerInFromProvider::from(array);
		let integer_array=integer_array_in_from_provider.as_cape_array_integer_in();
		self.resize(integer_array.size())?;
		for i in 0..integer_array.size() {
			self[i]=integer_array[i];
		}
		Ok(())
	}

}

impl<'a> CapeArrayIntegerProviderOut for CapeArrayIntegerOut<'a> {
	fn as_cape_array_integer_out(&mut self) -> C::ICapeArrayInteger {
		unsafe { **self.interface }
	}
}
