use crate::{COBIAError, CapeArrayRealInFromProvider, C};
use crate::cape_array::{CapeArrayIn,CapeArrayOut,ArrayInterface};
use crate::{CapeArrayRealProviderIn,CapeArrayRealProviderOut};

impl ArrayInterface<C::CapeReal> for C::ICapeArrayReal {

	fn get(&mut self,data:&mut *mut C::CapeReal,size:&mut C::CapeSize) {
		unsafe { (*self.vTbl).get.unwrap()(self.me,data as *mut *mut C::CapeReal,size) };
	}

	fn set(&mut self,data:&mut *mut C::CapeReal,size:C::CapeSize) -> C::CapeResult {
		unsafe { (*self.vTbl).setsize.unwrap()(self.me,size,data as *mut *mut C::CapeReal) }
	}

	fn get_const(&self,data:&mut *const C::CapeReal,size:&mut C::CapeSize) {
		unsafe { (*self.vTbl).get.unwrap()(self.me,data as *mut *const C::CapeReal as *mut *mut C::CapeReal,size) };
	}
}

/// CapeArrayRealIn wraps an ICapeArrayReal interface pointer.
///
/// Given an ICapeArrayReal interface pointer, this allows setting
///  and getting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayReal input arguments.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayRealIn) {
///     assert_eq!(a.as_vec(), vec![4.5,6.5]);
/// }
/// 
/// let arr = cobia::CapeArrayRealSlice::new(&[4.5,6.5]);
/// test_content(&CapeArrayRealInFromProvider::from(&arr).as_cape_array_real_in());
/// ```

pub type CapeArrayRealIn<'a> = CapeArrayIn<'a,C::CapeReal,C::ICapeArrayReal>;

impl<'a> CapeArrayRealProviderIn for CapeArrayRealIn<'a> {
	fn as_cape_array_real_in(&self) -> C::ICapeArrayReal {
		unsafe{**self.interface}
	}
}

/// CapeArrayRealOut wraps an ICapeArrayReal interface pointer.
///
/// Given an ICapeArrayReal interface pointer, this allows setting
///  and getting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayReal output arguments.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_content(a: &mut CapeArrayRealOut) {
///     a.put_array(&[4.5,6.5]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayRealVec::new();
/// set_content(&mut CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out());
/// assert_eq!(arr.as_vec(), &vec![4.5,6.5]);
/// ```

pub type CapeArrayRealOut<'a> = CapeArrayOut<'a,C::CapeReal,C::ICapeArrayReal>;

impl<'a> CapeArrayRealOut<'a> {

	/// Set the content of the real array from any object that implements CapeArrayRealProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayRealProviderIn
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut arr = cobia::CapeArrayRealVec::new();
	/// let mut arr1 = cobia::CapeArrayRealVec::from_slice(&vec![125.1,-19.4,3.14]);
	/// CapeArrayRealOutFromProvider::from(&mut arr).as_cape_array_real_out().set(&arr1);
	/// assert_eq!(arr.as_vec(), &[125.1,-19.4,3.14]);
	/// ```

	pub fn set<T:CapeArrayRealProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut real_array_in_from_provider = CapeArrayRealInFromProvider::from(array);
		let real_array=real_array_in_from_provider.as_cape_array_real_in();
		self.resize(real_array.size())?;
		for i in 0..real_array.size() {
			self[i]=real_array[i];
		}
		Ok(())
	}

}

impl<'a> CapeArrayRealProviderOut for CapeArrayRealOut<'a> {
	fn as_cape_array_real_out(&mut self) -> C::ICapeArrayReal {
		unsafe { **self.interface }
	}
}
