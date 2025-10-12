use crate::C;
use crate::*;

/// CapeStringIn wraps an ICapeString interface pointer as read-only.
///
/// Given an reference to an ICapeString interface pointer, this allows getting,
/// but not setting, the string.This is used for strings that are
/// input arguments to methods.
///
/// This interface is not typically used directly as pre-generated
/// wrappers provide input strings as str and return values as
/// Result<&str,cobia::COBIAError>. However, for 
/// CapeArrayStringIn elements, this interface is used.
///
/// A NULL interface pointer is treated as an empty string.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn test_string(s: &CapeStringIn) {
///     assert_eq!(s.as_string(),"idealGasEnthalpy");
/// }
/// 
/// let mut s1=cobia::CapeStringImpl::from("idealGasEnthalpy");
/// test_string(&CapeStringInFromProvider::from(&s1).as_cape_string_in())
/// ```

#[derive(Debug)]
pub struct CapeStringIn<'a> {
	interface: &'a *mut C::ICapeString,
	slice: &'a [i8]
}

impl<'a> CapeStringIn<'a> {
	/// Create a new CapeStringIn from an ICapeString interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeString interface
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	///	let i_cape_string=s1.as_cape_string_out();
	///	let mut i_cape_string_ptr=(&i_cape_string as *const C::ICapeString).cast_mut(); //normally a pointer to the interface is received
	///	let s = cobia::CapeStringIn::new(&i_cape_string_ptr); //CapeStringIn from *mut C::ICapeString
	/// assert_eq!(s.as_string(),"idealGasEnthalpy");
	/// ```

	pub fn new(interface: &'a *mut C::ICapeString) -> CapeStringIn<'a> {
		let mut slice: &[i8] = &[];
		if !interface.is_null() {
			let mut data: *const i8 = std::ptr::null_mut();
			let mut size: C::CapeSize = 0;
			unsafe {
				(*(**interface).vTbl).get.unwrap()(
					(**interface).me,
					&mut data as *mut *const i8,
					&mut size as *mut C::CapeSize,
				)
			}
			if (!data.is_null()) && (size != 0) {
				slice=unsafe { std::slice::from_raw_parts(data, size as usize + 1) }; //include the terminating null
			}
		}
		CapeStringIn { 
			interface,
			slice			
		}
	}

	/// Return the content of the string as a string.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_string(s: &CapeStringIn) {
	///     assert_eq!(s.as_string(),"idealGasEnthalpy");
	/// }
	/// 
	/// let mut s1=cobia::CapeStringImpl::from("idealGasEnthalpy");
	/// test_string(&CapeStringInFromProvider::from(&s1).as_cape_string_in())
	/// ```

	pub fn as_string(&self) -> String {
		if self.slice.is_empty() {
			return String::new();
		}
		let slice=unsafe { std::slice::from_raw_parts(self.slice.as_ptr() as *const u8, self.slice.len()-1) }; //exclude null
		String::from_utf8_lossy(slice).into() 
	}

	/// Return the string as a slice
	///
	/// Note that for empty strings, the null terminator may
	/// not be included, so check for a zero length slice.

	pub fn as_slice(&self) -> &[C::CapeCharacter] {
		self.slice
	}

	/// Case insentitive comparison
	/// 
	/// # Arguments
	///	
	/// * `other` - The other string to compare to
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_string(s: &CapeStringIn) {
	///		let s2=cobia::CapeStringImpl::from_string("IDEALGASENTHALPY");
	///     assert_eq!(s.eq_ignore_case(&s2),true);
	/// }
	/// 
	/// let s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// test_string(&CapeStringInFromProvider::from(&s1).as_cape_string_in())
	/// ```
	pub fn eq_ignore_case<T:CapeStringConstProvider>(&self, other: &T) -> bool {
		let my_slice=unsafe{std::slice::from_raw_parts(self.slice.as_ptr() as *const u8,self.slice.len()-1)};
		let my_str=String::from_utf8_lossy(my_slice);
		let mut my_it=my_str.chars();
		let (other_ptr,other_len)=other.as_capechar_const_with_length();
		let other_slice=unsafe{std::slice::from_raw_parts(other_ptr as *const u8,other_len as usize)};
		let other_str=String::from_utf8_lossy(other_slice);
		let mut other_it=other_str.chars();
		loop {
			match (my_it.next(),other_it.next()) {
				(Some(c1),Some(c2)) => {
					if c1.to_lowercase().next().unwrap() != c2.to_lowercase().next().unwrap() {
						return false;
					}
				}
				(Some(_),None) => return false,
				(None,Some(_)) => return false,
				(None,None) => return true,
			}
		}
	}

	/// Case sentitive comparison
	/// 
	/// # Arguments
	///	
	/// * `other` - The other string to compare to
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_string(s: &CapeStringIn) {
	///		let s2=cobia::CapeStringImpl::from_string("IDEALGASENTHALPY");
	///		assert_eq!(s.eq(&s2),false);
	///		let s3=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	///		assert_eq!(s.eq(&s3),true);
	/// }
	/// 
	/// let s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// test_string(&CapeStringInFromProvider::from(&s1).as_cape_string_in())
	/// ```
	pub fn eq<T:CapeStringConstProvider>(&self, other: &T) -> bool {
		let my_slice=unsafe{std::slice::from_raw_parts(self.slice.as_ptr() as *const u8,self.slice.len()-1)};
		let (other_ptr,other_len)=other.as_capechar_const_with_length();
		let other_slice=unsafe{std::slice::from_raw_parts(other_ptr as *const u8,other_len as usize)};
		my_slice==other_slice
	}

	/// Check empty
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_empty(s: &CapeStringIn) {
	///		assert_eq!(s.is_empty(),true);
	/// }
	/// 
	/// let s1=cobia::CapeStringImpl::new();
	/// test_empty(&CapeStringInFromProvider::from(&s1).as_cape_string_in())
	/// ```

	pub fn is_empty(&self) -> bool {
		(self.slice.is_empty())||(self.slice[0]==0i8)
	}

}

impl<'a,T:CapeStringConstProvider> PartialEq<T> for CapeStringIn<'a> {
	/// Compare the CapeStringIn with a string slice or any object that implements
	/// CapeStringConstProvider.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_string(s: &CapeStringIn) {
	///		let s2=cobia::CapeStringImpl::from_string("IDEALGASENTHALPY");
	///		assert_ne!(s,&s2);
	///		let s3=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	///		assert_eq!(s,&s3);
	/// }
	/// 
	/// let s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// test_string(&CapeStringInFromProvider::from(&s1).as_cape_string_in())
	/// ```
	fn eq(&self, other: &T) -> bool {
		self.eq(other)
	}
}

impl<'a> CapeStringConstProvider for CapeStringIn<'a> {
	///Return as CapeCharacter const pointer with length
	///
	/// The caller must ensure that the lifetime of the CapeStringImpl
	/// is longer than the pointer returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_string(s: &CapeStringIn) {
	///		let (ptr,len)=s.as_capechar_const_with_length(); ///... while ptr is used
	///		assert_eq!(len,16);
	/// }
	/// 
	/// let s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// test_string(&CapeStringInFromProvider::from(&s1).as_cape_string_in())
	/// ```
	fn as_capechar_const_with_length(&self) -> (*const C::CapeCharacter, C::CapeSize) {
		if self.slice.is_empty() {
			("\0".as_ptr() as *const C::CapeCharacter,0 as C::CapeSize)
		} else {
			(self.slice.as_ptr(),(self.slice.len()-1) as C::CapeSize)
		}
	}
	///Return as CapeCharacter const pointer
	///
	/// The caller must ensure that the lifetime of the CapeStringImpl
	/// is longer than the pointer returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_string(s: &CapeStringIn) {
	///		let (ptr,len)=s.as_capechar_const_with_length(); ///... while ptr is used
	///		assert_eq!(unsafe{*ptr},'i' as i8);
	/// }
	/// 
	/// let s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// test_string(&CapeStringInFromProvider::from(&s1).as_cape_string_in())
	/// ```
	fn as_capechar_const(&self) -> *const C::CapeCharacter {
		if self.slice.is_empty() {
			"\0".as_ptr() as *const C::CapeCharacter
		} else {
			self.slice.as_ptr()
		}
	}
}

impl<'a> std::fmt::Display for CapeStringIn<'a> {
	/// Formats the CapeStringIn error using the given formatter.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_format(s: &CapeStringIn) {
	///		assert_eq!(format!("{}",s),"idealGasEnthalpy");
	/// }
	/// 
	/// let s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// test_format(&CapeStringInFromProvider::from(&s1).as_cape_string_in())
	/// ```

	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let slice=unsafe { std::slice::from_raw_parts(self.slice.as_ptr() as *const u8, self.slice.len()-1) }; //exclude null
		write!(f, "{}", String::from_utf8_lossy(slice))
	}
}

impl<'a> CapeStringProviderIn for CapeStringIn<'a> {

	fn as_cape_string_in(&self) -> C::ICapeString {
		unsafe { **self.interface }
	}

}

/// CapeStringOut wraps an ICapeString interface pointer.
///
/// Given an ICapeString interface pointer, this allows setting
///  and getting the string.
///
/// This interface is not typically used directly as pre-generated
/// wrappers provide input strings as str and return values as
/// Result<&str,cobia::COBIAError>. However for output values and
/// CapeArrayStringOut elements, this interface is used.
///
/// NULL pointers are not allowed.
///
/// # Examples
///
/// ```
/// use cobia::*;
///
/// fn set_content(s: &mut CapeStringOut) {
///		s.set_string("idealGasEnthalpy").unwrap();
/// }
/// 
/// let mut s1=cobia::CapeStringImpl::new();
/// set_content(&mut CapeStringOutFromProvider::from(&mut s1).as_cape_string_out());
/// assert_eq!(s1.as_string(),"idealGasEnthalpy");
/// ```

#[derive(Debug)]
pub struct CapeStringOut<'a> {
	interface: &'a mut *mut C::ICapeString,
}

impl<'a> CapeStringOut<'a> {
	/// Create a new CapeStringOut from an ICapeString interface pointer.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeString interface
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// let mut s1=cobia::CapeStringImpl::from("idealGasEnthalpy");
	/// let i_cape_string=s1.as_cape_string_out();
	///	let mut i_cape_string_ptr=(&i_cape_string as *const C::ICapeString).cast_mut(); //normally a pointer to the interface is received
	///	let s = cobia::CapeStringOut::new(&mut i_cape_string_ptr); //CapeStringOut from *mut C::ICapeString
	/// assert_eq!(s.as_string(),"idealGasEnthalpy");
	/// ```

	pub fn new(interface: &'a mut *mut C::ICapeString) -> CapeStringOut<'a> {
		CapeStringOut { interface }
	}

	/// Return the content of the string as a string.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn check_content(s: &mut CapeStringOut) {
	///		assert_eq!(s.as_string(),"idealGasEnthalpy"); //return as string
	/// }
	/// 
	/// let mut s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// check_content(&mut CapeStringOutFromProvider::from(&mut s1).as_cape_string_out());
	/// ```

	pub fn as_string(&self) -> String {
		let mut data: *const i8 = std::ptr::null_mut();
		let mut size: C::CapeSize = 0;
		unsafe {
			(*(**self.interface).vTbl).get.unwrap()(
				(**self.interface).me,
				&mut data as *mut *const i8,
				&mut size as *mut C::CapeSize,
			)
		}
		if (data.is_null()) || (size == 0) {
			return String::new();
		}
		let slice = unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) };
		String::from_utf8_lossy(slice).into()
	}


	/// Set the content of the string any CAPE-OPEN string
	///
	/// # Arguments
	///
	/// * `s` - An object implementing CapeStringConstProvider
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_content(s: &mut CapeStringOut) {
	///		let s0=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	///		s.set(&s0).unwrap();
	/// }
	/// 
	/// let mut s1=cobia::CapeStringImpl::new();
	/// set_content(&mut CapeStringOutFromProvider::from(&mut s1).as_cape_string_out());
	/// assert_eq!(s1.as_string(),"idealGasEnthalpy");
	/// ```
	pub fn set<T:CapeStringConstProvider>(&self, s: &T) -> Result<(), COBIAError> {
		let (ptr, sz) = s.as_capechar_const_with_length();
		let result =
			unsafe { (*(**self.interface).vTbl).set.unwrap()((**self.interface).me, ptr, sz) };
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Set the content of the string from a string slice.
	///
	/// # Arguments
	///
	/// * `s` - A string slice
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn set_content(s: &mut CapeStringOut) {
	///		s.set_string("idealGasEnthalpy").unwrap();
	/// }
	/// 
	/// let mut s1=cobia::CapeStringImpl::new();
	/// set_content(&mut CapeStringOutFromProvider::from(&mut s1).as_cape_string_out());
	/// assert_eq!(s1.as_string(),"idealGasEnthalpy");
	/// ```
	pub fn set_string<T: AsRef<str>>(&self, s: T) -> Result<(), COBIAError> {
		self.set(&CapeStringImpl::from_string(s.as_ref()))
	}

	/// Case insentitive comparison
	/// 
	/// # Arguments
	///	
	/// * `other` - The other string to compare to
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// 
	/// fn check_string(s: &mut CapeStringOut) {
	///	    let s2=cobia::CapeStringImpl::from_string("IDEALGASENTHALPY");
	///		assert_eq!(s.eq_ignore_case(&s2),true);
	/// }
	/// 
	/// let mut s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// check_string(&mut CapeStringOutFromProvider::from(&mut s1).as_cape_string_out());
	/// ```
	pub fn eq_ignore_case<T:CapeStringConstProvider>(&self, other: &T) -> bool {
		let mut data: *const i8 = std::ptr::null_mut();
		let mut size: C::CapeSize = 0;
		unsafe {
			(*(**self.interface).vTbl).get.unwrap()(
				(**self.interface).me,
				&mut data as *mut *const i8,
				&mut size as *mut C::CapeSize,
			)
		}
		if (data.is_null()) || (size == 0) {
			//compare to empty string
			let (_other_ptr,other_len)=other.as_capechar_const_with_length();
			return other_len==0;
		}
		let my_slice = unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) };
		let my_str=String::from_utf8_lossy(my_slice);
		let mut my_it=my_str.chars();
		let (other_ptr,other_len)=other.as_capechar_const_with_length();
		let other_slice=unsafe{std::slice::from_raw_parts(other_ptr as *const u8,other_len as usize)};
		let other_str=String::from_utf8_lossy(other_slice);
		let mut other_it=other_str.chars();
		loop {
			match (my_it.next(),other_it.next()) {
				(Some(c1),Some(c2)) => {
					if c1.to_lowercase().next().unwrap() != c2.to_lowercase().next().unwrap() {
						return false;
					}
				}
				(Some(_),None) => return false,
				(None,Some(_)) => return false,
				(None,None) => return true,
			}
		}
	}

	/// Case sentitive comparison
	/// 
	/// # Arguments
	///	
	/// * `other` - The other string to compare to
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_string(s: &CapeStringOut) {
	///		let s2=cobia::CapeStringImpl::from_string("IDEALGASENTHALPY");
	///		assert_eq!(s.eq(&s2),false);
	///		let s3=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	///		assert_eq!(s.eq(&s3),true);
	/// }
	/// 
	/// let mut s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// test_string(&CapeStringOutFromProvider::from(&mut s1).as_cape_string_out())
	/// ```
	pub fn eq<T:CapeStringConstProvider>(&self, other: &T) -> bool {
		let (my_ptr,my_len)=self.as_capechar_const_with_length();
		let (other_ptr,other_len)=other.as_capechar_const_with_length();
		if my_len != other_len {
			return false; //lengths differ
		}
		let my_slice=unsafe{std::slice::from_raw_parts(my_ptr as *const u8,my_len as usize)};
		let other_slice=unsafe{std::slice::from_raw_parts(other_ptr as *const u8,other_len as usize)};
		my_slice==other_slice
	}

	/// Check empty
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// 
	/// fn check_empty(s: &mut CapeStringOut) {
	///	    assert_eq!(s.is_empty(),true);
	/// }
	/// 
	/// let mut s1=cobia::CapeStringImpl::new();
	/// check_empty(&mut CapeStringOutFromProvider::from(&mut s1).as_cape_string_out());
	/// ```
	pub fn is_empty(&self) -> bool {
		let mut data: *const i8 = std::ptr::null_mut();
		let mut size: C::CapeSize = 0;
		unsafe {
			(*(**self.interface).vTbl).get.unwrap()(
				(**self.interface).me,
				&mut data as *mut *const i8,
				&mut size as *mut C::CapeSize,
			)
		}
		(data.is_null()) || (size == 0)
	}

}

impl<'a,T:CapeStringConstProvider> PartialEq<T> for CapeStringOut<'a> {
	/// Compare the CapeStringOut with a string slice or any object that implements
	/// CapeStringConstProvider.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	///
	/// fn test_string(s: &mut CapeStringOut) {
	///		let s2=cobia::CapeStringImpl::from_string("IDEALGASENTHALPY");
	///		assert_ne!(s,&s2);
	///		let s3=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	///		assert_eq!(s,&s3);
	/// }
	/// 
	/// let mut s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// test_string(&mut CapeStringOutFromProvider::from(&mut s1).as_cape_string_out())
	/// ```
	fn eq(&self, other: &T) -> bool {
		self.eq(other)
	}
}

impl<'a> CapeStringConstProvider for CapeStringOut<'a> {
	///Return as CapeCharacter const pointer with length
	///
	/// The caller must ensure that the lifetime of the CapeStringImpl
	/// is longer than the pointer returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// 
	/// fn check_size(s: &mut CapeStringOut) {
	///     let (ptr,len)=s.as_capechar_const_with_length();
	///	    assert_eq!(len,16);
	/// }
	/// 
	/// let mut s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// check_size(&mut CapeStringOutFromProvider::from(&mut s1).as_cape_string_out());
	/// ```
	fn as_capechar_const_with_length(&self) -> (*const C::CapeCharacter, C::CapeSize) {
		if self.interface.is_null() {
			("\0".as_ptr() as *const C::CapeCharacter,0 as C::CapeSize)
		} else {
			let mut data: *const i8 = std::ptr::null_mut();
			let mut size: C::CapeSize = 0;
			unsafe {
				(*(**self.interface).vTbl).get.unwrap()(
					(**self.interface).me,
					&mut data as *mut *const i8,
					&mut size as *mut C::CapeSize,
				)
			}
			if (data.is_null()) || (size == 0) {
				("\0".as_ptr() as *const C::CapeCharacter,0 as C::CapeSize)
			} else {
				(data as *const C::CapeCharacter,size)
			}
		}
	}
	///Return as CapeCharacter const pointer with length
	///
	/// The caller must ensure that the lifetime of the CapeStringImpl
	/// is longer than the pointer returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// 
	/// fn check_ptr(s: &mut CapeStringOut) {
	///		let ptr=s.as_capechar_const(); ///... while ptr is used
	///		assert_eq!(unsafe{*ptr},'i' as i8);
	/// }
	/// 
	/// let mut s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// check_ptr(&mut CapeStringOutFromProvider::from(&mut s1).as_cape_string_out());
	/// ```
	fn as_capechar_const(&self) -> *const C::CapeCharacter {
		if self.interface.is_null() {
			"\0".as_ptr() as *const C::CapeCharacter
		} else {
			let mut data: *const i8 = std::ptr::null_mut();
			let mut size: C::CapeSize = 0;
			unsafe {
				(*(**self.interface).vTbl).get.unwrap()(
					(**self.interface).me,
					&mut data as *mut *const i8,
					&mut size as *mut C::CapeSize,
				)
			}
			if (data.is_null()) || (size == 0) {
				"\0".as_ptr() as *const C::CapeCharacter
			} else {
				data as *const C::CapeCharacter
			}
		}
	}
}


impl<'a> std::fmt::Display for CapeStringOut<'a> {
	/// Formats the CapeStringOut error using the given formatter.
	///
	/// # Examples
	///
	/// ```
	/// use cobia::*;
	/// 
	/// fn check_format(s: &mut CapeStringOut) {
	///		let ptr=s.as_capechar_const(); ///... while ptr is used
	///		assert_eq!(format!("{}",s),"idealGasEnthalpy");
	/// }
	/// 
	/// let mut s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// check_format(&mut CapeStringOutFromProvider::from(&mut s1).as_cape_string_out());
	/// ```
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let (ptr, sz) = self.as_capechar_const_with_length();
		let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, sz as usize) };
		write!(f, "{}", String::from_utf8_lossy(slice))
	}
}

impl<'a> CapeStringProviderOut for CapeStringOut<'a> {
	fn as_cape_string_out(&mut self) -> C::ICapeString {
		unsafe { **self.interface }
	}
}

