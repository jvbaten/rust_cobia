use crate::{C,COBIAError,CapeArrayByteInFromProvider};
use crate::cape_array::{ArrayInterface,CapeArrayIn,CapeArrayOut};
use crate::{CapeArrayByteProviderIn,CapeArrayByteProviderOut};

impl ArrayInterface<C::CapeByte> for C::ICapeArrayByte {

	fn get(&mut self,data:&mut *mut C::CapeByte,size:&mut C::CapeSize) {
		unsafe { (*self.vTbl).get.unwrap()(self.me,data as *mut *mut C::CapeByte,size) };
	}

	fn set(&mut self,data:&mut *mut C::CapeByte,size:C::CapeSize) -> C::CapeResult {
		unsafe { (*self.vTbl).setsize.unwrap()(self.me,size,data as *mut *mut C::CapeByte) }
	}

	fn get_const(&self,data:&mut *const C::CapeByte,size:&mut C::CapeSize) {
		unsafe { (*self.vTbl).get.unwrap()(self.me,data as *mut *const C::CapeByte as *mut *mut C::CapeByte,size) };
	}

}

/// CapeArrayByteIn wraps an ICapeArrayByte interface pointer.
///
/// Given an ICapeArrayByte interface pointer, this allows setting
///  and getting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayByte input arguments.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_content(a: &CapeArrayByteIn) {
///     assert_eq!(a.as_vec(), vec![2u8,8u8,10u8]);
/// }
/// 
/// let arr = cobia::CapeArrayByteVec::from_slice(&[2u8,8u8,10u8]);
/// test_content(&CapeArrayByteInFromProvider::from(&arr).as_cape_array_byte_in())
/// ```

pub type CapeArrayByteIn<'a> = CapeArrayIn<'a,C::CapeByte,C::ICapeArrayByte>;


impl<'a> CapeArrayByteProviderIn for CapeArrayByteIn<'a> {
	fn as_cape_array_byte_in(&self) -> C::ICapeArrayByte {
		unsafe { **self.interface }
	}
}

/// CapeArrayByteOut wraps an ICapeArrayByte interface pointer.
///
/// Given an ICapeArrayByte interface pointer, this allows setting
///  and getting the elements.
///
/// This interface is typically used as arguments to rust methods
/// on traits that are generated from CAPE-OPEN interfaces that have
/// ICapeArrayByte output arguments.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_content(a: &mut CapeArrayByteOut) {
///		a.put_array(&[2u8,8u8,10u8]).unwrap();
/// }
/// 
/// let mut arr = cobia::CapeArrayByteVec::new();
/// set_content(&mut CapeArrayByteOutFromProvider::from(&mut arr).as_cape_array_byte_out());
/// assert_eq!(arr.as_vec(), &vec![2u8,8u8,10u8]);
/// ```

pub type CapeArrayByteOut<'a> = CapeArrayOut<'a,C::CapeByte,C::ICapeArrayByte>;

impl<'a> CapeArrayByteOut<'a> {

	/// Set the content of the byte array from any object that implements CapeArrayByteProviderIn.
	///
	/// # Arguments
	/// * `array` - An object that implements CapeArrayByteProviderIn
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// let mut arr = cobia::CapeArrayByteVec::new();
	/// let mut arr1 = cobia::CapeArrayByteVec::from_slice(&vec![2u8,8u8,10u8]);
	/// CapeArrayByteOutFromProvider::from(&mut arr).as_cape_array_byte_out().set(&arr1);
	/// assert_eq!(arr.as_vec(), &[2u8,8u8,10u8]);
	/// ```

	pub fn set<T:CapeArrayByteProviderIn>(&mut self,array:&T) -> Result<(), COBIAError> {
		let mut byte_array_in_from_provider = CapeArrayByteInFromProvider::from(array);
		let byte_array=byte_array_in_from_provider.as_cape_array_byte_in();
		self.resize(byte_array.size())?;
		for i in 0..byte_array.size() {
			self[i]=byte_array[i];
		}
		Ok(())
	}

}

impl<'a> CapeArrayByteProviderOut for CapeArrayByteOut<'a> {
	fn as_cape_array_byte_out(&mut self) -> C::ICapeArrayByte {
		unsafe { **self.interface }
	}
}



