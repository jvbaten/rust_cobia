use crate::{C,CapeStringImpl};
#[cfg(doc)] use crate::CapeOpenMap;
use crate::cape_data_traits::*;
use std::fmt;
use crate::cape_result_value::*;

/// Class to store a platform dependent string encoding for use in case
/// insensitive hash maps or for use of case insensitive comparisons.
///
/// For COBIA, strings go over the pipeline as null terminated.
/// For most platforms, COBIA requires UTF-8 encoding.
///
/// Because this translation is always required, there is no 
/// read-only string implementation that refers to a string slice.
/// However, this string implementation is immutable and can therefore
/// be used as static.
///
/// This implementation uses a `String` to store the string data, and 
/// the null character is explicitly appended.
///
/// A common use case in CAPE-OPEN is to to have a string constant
/// for comparison with CapeString-like objects. This class is a  
/// specialization for this purpose: the string is stored in 
/// lower case, and PartialEq does case insensitive comparison.
/// 
/// Another common use case in CAPE-OPEN is to make hash maps of
/// strings for case-insentive lookups. A specialized class is
/// available for this purpose: [`CapeStringHashKey`].
/// 
/// PartialEq can be directly used to for a CapeStringConstNoCase on
/// the left, and any object implementing CapeStringConstProvider on
/// the right (but not vice versa)
///
/// # Examples
///
/// ```
/// use cobia::*;
/// use cobia::prelude::*;
/// let s=cobia::CapeStringConstNoCase::from_string("idealGasEnthalpy");
/// let s2=cobia::CapeStringImpl::from_string("IDEALGASENTHALPY");
/// assert_eq!(s,s2);
///
/// fn test_eq(s:CapeStringConstNoCase, s3:&CapeStringIn) {
///		assert_eq!(&s,s3);
/// }
///
///	test_eq(s,&CapeStringInFromProvider::from(&s2).as_cape_string_in()); 
/// ```

#[derive(Debug)]
pub struct CapeStringConstNoCase {
	data: String,
}

impl CapeStringConstNoCase {

	///Construct from string
	///
	/// # Arguments
	///
	/// * `s` - A string slice to be converted to a CapeStringConstNoCase
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringConstNoCase::from_string("idealGasEnthalpy");
	/// ```
	pub fn from_string<T:AsRef<str>>(s: T) -> Self {
		let s=s.as_ref();
		let mut data=String::with_capacity(s.len()+1);
		for c in s.chars() {
			data.push(c.to_lowercase().next().unwrap());
		}
		data.push('\0');
		CapeStringConstNoCase {
			data
		}
	}

	///Construct from CapeCharacter pointer
	///
	/// # Arguments
	///
	/// * `ptr` - A const CapeCharacter pointer
	/// * `size` - Length of the string pointed to
	pub fn from_cape_char_const(ptr:*const C::CapeCharacter, size:C::CapeSize) -> Self {
		let my_slice=unsafe{std::slice::from_raw_parts(ptr as *const u8,size as usize)};
		let my_str=String::from_utf8_lossy(my_slice);
		Self::from_string(my_str)
	}

	/// Create a new CapeStringConstNoCase from a string.
	///
	/// # Arguments
	///
	/// * `s` - A string
	pub fn from(s:Option<&str>) -> Self {
		match s {
			Some(s) => {
				Self::from_string(s)
			}
			None => {
				CapeStringConstNoCase {
					data: "\0".into(),
				}
			}
		}
	}

	///Return as string
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringConstNoCase::from_string("idealGasEnthalpy");
	/// assert_eq!(s.as_string(),"idealgasenthalpy"); //note that CapeStringConstNoCase stores strings in lower case
	/// ```
	pub fn as_string(&self) -> String {
		self.data[..self.data.len() - 1].into()
	}

}

impl fmt::Display for CapeStringConstNoCase {
	/// Formats the CapeStringConstNoCase error using the given formatter.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringConstNoCase::from_string("idealGasEnthalpy");
	/// assert_eq!(format!("{}",s),"idealgasenthalpy"); //note that CapeStringConstNoCase stores strings in lower case
	/// ```
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", &self.data[..(self.data.len() - 1)])
	}
}

impl CapeStringConstProvider for CapeStringConstNoCase {
	///Return as CapeCharacter const pointer
	///
	/// The caller must ensure that the lifetime of the CapeStringImpl
	/// is longer than the pointer returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let s=cobia::CapeStringConstNoCase::from_string("idealGasEnthalpy"); //must remain in scope....
	/// let ptr=s.as_capechar_const(); ///... while ptr is used
	/// assert_eq!(unsafe{*ptr},'i' as i8);
	/// ```
	fn as_capechar_const(&self) -> *const C::CapeCharacter {
		self.data.as_ptr() as *const C::CapeCharacter
	}
	///Return as CapeCharacter const pointer with length
	///
	/// The caller must ensure that the lifetime of the CapeStringImpl
	/// is longer than the pointer returned.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// let s=cobia::CapeStringConstNoCase::from_string("idealGasEnthalpy"); //must remain in scope....
	/// let (ptr,len)=s.as_capechar_const_with_length(); ///... while ptr is used
	/// assert_eq!(len,16);
	/// ```
	fn as_capechar_const_with_length(&self) -> (*const C::CapeCharacter, C::CapeSize) {
		(self.data.as_ptr() as *const C::CapeCharacter, (self.data.len() - 1) as C::CapeSize) //length without the terminating null
	}
}

impl<T:AsRef<str>> From<T> for CapeStringConstNoCase {
	fn from(s: T) -> Self {
		CapeStringConstNoCase::from_string(s)
	}
}

impl Clone for CapeStringConstNoCase {
	fn clone(&self) -> Self {
		CapeStringConstNoCase {
			data: self.data.clone(),
		}
	}
}

impl<T: CapeStringConstProvider> PartialEq<T> for CapeStringConstNoCase {
	fn eq(&self, other: &T) -> bool {
		let (my_ptr,my_len)=self.as_capechar_const_with_length();
		let my_slice=unsafe{std::slice::from_raw_parts(my_ptr as *const u8,my_len as usize)};
		let my_str=String::from_utf8_lossy(my_slice);
		let mut my_it=my_str.chars();
		let (other_ptr,other_len)=other.as_capechar_const_with_length();
		let other_slice=unsafe{std::slice::from_raw_parts(other_ptr as *const u8,other_len as usize)};
		let other_str=String::from_utf8_lossy(other_slice);
		let mut other_it=other_str.chars();
		loop {
			match (my_it.next(),other_it.next()) {
				(Some(c1),Some(c2)) => {
					if c1 != c2.to_lowercase().next().unwrap() { //c1 is already lower case
						return false;
					}
				}
				(Some(_),None) => return false,
				(None,Some(_)) => return false,
				(None,None) => return true,
			}
		}
	}
}

impl Eq for CapeStringConstNoCase {}


impl CapeStringConstNoCase {

	extern "C" fn string_get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *const C::CapeCharacter,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut CapeStringConstNoCase;
		let s: &mut CapeStringConstNoCase = unsafe { &mut *p };
		unsafe {
			*data = s.data.as_ptr() as *const C::CapeCharacter;
			*size = s.data.len() as C::CapeSize - 1; //exclude null terminator
		}
	}

	extern "C" fn string_set(
		me: *mut ::std::os::raw::c_void,
		data: *const C::CapeCharacter,
		size: C::CapeSize,
	) -> C::CapeResult {
		let p = me as *mut CapeStringConstNoCase;
		let s: &mut CapeStringConstNoCase = unsafe { &mut *p };
		s.data.clear();
		let slice=unsafe{ std::slice::from_raw_parts(data as *const u8, size as usize)};
		s.data.reserve(slice.len() + 1);
		s.data.extend(String::from_utf8_lossy(slice).as_ref().chars());
		s.data.push('\0'); //null terminator
		COBIAERR_NOERROR
	}

	const CAPE_STRING_VTABLE: C::ICapeString_VTable = C::ICapeString_VTable {
		get: Some(CapeStringConstNoCase::string_get),
		set: Some(CapeStringConstNoCase::string_set),
	};

}

impl CapeStringProviderIn for CapeStringConstNoCase {
	fn as_cape_string_in(&self) -> C::ICapeString {
		C::ICapeString {
			vTbl:(&CapeStringConstNoCase::CAPE_STRING_VTABLE as *const C::ICapeString_VTable).cast_mut(),
			me:(self as *const CapeStringConstNoCase).cast_mut() as *mut ::std::os::raw::c_void
		}
	}
}

/// Class to store a platform dependent string encoding for use in case
/// insensitive hash maps or for use of case insensitive comparisons.
///
/// For COBIA, strings go over the pipeline as null terminated.
/// For most platsforms, COBIA requires UTF-8 encoding.
///
/// The CapeStringHashKey implementation uses a `CapeStringConstNoCase` to store,
/// owned string data, or allows reference to data provided by any 
/// class that implements CapeStringConstProvider, so that a copy of 
/// the data is not needed for hash lookups.
///
/// A convenience class [`CapeOpenMap`] is defined, that uses the 
/// more performant hasher in the FxHasmap class and wraps members
/// in accordance with the above requirements.
///
/// Note that this type cannot serve as a cape string provider, as
/// this would require a mutable interface pointer.

#[derive(Debug)]
pub enum CapeStringHashKey<'a> {
	Owned(CapeStringConstNoCase),
	Borrowed(*const C::CapeCharacter, C::CapeSize,std::marker::PhantomData<&'a ()>),
}


//Borrewed keys have a local, limited time span 
//and are not shared between threads; Owned keys 
//are constant and safe to share
unsafe impl<'a> Send for CapeStringHashKey<'a> {}
unsafe impl<'a> Sync for CapeStringHashKey<'a> {} 

impl<'a> CapeStringHashKey<'a> {

	///Construct from string that owns the data
	///
	/// # Arguments
	///
	/// * `s` - A string slice to be converted to a CapeStringHashKey
	///
	pub fn from_string<T:AsRef<str>>(s: T) -> Self {
		CapeStringHashKey::Owned(CapeStringConstNoCase::from_string(s))
	}
	///Construct from CapeCharacter pointer
	///
	/// # Arguments
	///
	/// * `ptr` - A const CapeCharacter pointer
	/// * `size` - Length of the string pointed to
	pub fn from_cape_char_const(ptr:*const C::CapeCharacter, size:C::CapeSize) -> Self {
		let my_slice=unsafe{std::slice::from_raw_parts(ptr as *const u8,size as usize)};
		let my_str=String::from_utf8_lossy(my_slice);
		Self::from_string(my_str)
	}
	///Construct from string constant provider reference that does not own the data
	///
	/// # Arguments
	///
	/// * `c` - A string constant reference 
	///
	pub fn from_string_constant<'b,T:CapeStringConstProvider>(c: &'b T) -> CapeStringHashKey<'b> {
		let (ptr,len)=c.as_capechar_const_with_length();
		CapeStringHashKey::Borrowed(ptr,len,std::default::Default::default())
	}
	///Return as string
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringHashKey::from_string("idealGasEnthalpy");
	/// assert_eq!(s.as_string(),"idealgasenthalpy"); //note that CapeStringConstNoCase stores strings in lower case
	/// ```
	pub fn as_string(&self) -> String {
		let (ptr,len)=match self {
			CapeStringHashKey::Owned(str_no_case) => {
				str_no_case.as_capechar_const_with_length()
			}
			CapeStringHashKey::Borrowed(ptr,len,_) => {
				(*ptr,*len)
			}
		};
		if len==0 {
			String::new()
		} else {
			let data=unsafe { std::slice::from_raw_parts(ptr as *const u8,len as usize) };
			String::from_utf8_lossy(&data).into()
		}
	}

}

impl<'a> std::fmt::Display for CapeStringHashKey<'a> {
	/// Formats the CapeStringConstNoCase error using the given formatter.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringConstNoCase::from_string("idealGasEnthalpy");
	/// assert_eq!(format!("{}",s),"idealgasenthalpy"); //note that CapeStringConstNoCase stores strings in lower case
	/// ```
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let (ptr,len)=match self {
			CapeStringHashKey::Owned(str_no_case) => {
				str_no_case.as_capechar_const_with_length()
			}
			CapeStringHashKey::Borrowed(ptr,len,_) => {
				(*ptr,*len)
			}
		};
		if len == (0 as C::CapeSize) {
			write!(f, "")
		} else {
			let slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
			write!(f, "{}", String::from_utf8_lossy(slice).as_ref())
		}
	}
}

impl<'a> PartialEq for CapeStringHashKey<'a> {
	fn eq(&self, other: &Self) -> bool {
		//optimize for owned vs borrowed, owned strings are already in lower case
		match self {
			CapeStringHashKey::Owned(data) => {
				match other {
					CapeStringHashKey::Owned(other_data) => {
						data==other_data //both are already lower case
					}
					CapeStringHashKey::Borrowed(other_ptr,other_len,_) => {
						let (my_ptr,my_len)=data.as_capechar_const_with_length();
						let my_slice=unsafe{std::slice::from_raw_parts(my_ptr as *const u8,my_len as usize)};
						let my_str=String::from_utf8_lossy(my_slice);
						let mut my_it=my_str.chars();
						let other_slice=unsafe{std::slice::from_raw_parts(*other_ptr as *const u8,*other_len as usize)};
						let other_str=String::from_utf8_lossy(other_slice);
						let mut other_it=other_str.chars();
						loop {
							match (my_it.next(),other_it.next()) {
								(Some(c1),Some(c2)) => {
									if c1 != c2.to_lowercase().next().unwrap() { //c1 is already lower case
										return false;
									}
								}
								(Some(_),None) => return false,
								(None,Some(_)) => return false,
								(None,None) => return true,
							}
						}
					}
				}
			},
			CapeStringHashKey::Borrowed(my_ptr,my_len,_) => {
				let my_slice=unsafe{std::slice::from_raw_parts(*my_ptr as *const u8,*my_len as usize)};
				let my_str=String::from_utf8_lossy(my_slice);
				let mut my_it=my_str.chars();
				match other {
					CapeStringHashKey::Owned(other_data) => {
						let (other_ptr,other_len)=other_data.as_capechar_const_with_length();
						let other_slice=unsafe{std::slice::from_raw_parts(other_ptr as *const u8,other_len as usize)};
						let other_str=String::from_utf8_lossy(other_slice);
						let mut other_it=other_str.chars();
						loop {
							match (my_it.next(),other_it.next()) {
								(Some(c1),Some(c2)) => {
									if c1.to_lowercase().next().unwrap() != c2 { //c2 is already lower case
										return false;
									}
								}
								(Some(_),None) => return false,
								(None,Some(_)) => return false,
								(None,None) => return true,
							}
						}
					}
					CapeStringHashKey::Borrowed(other_ptr,other_len,_) => {
						let other_slice=unsafe{std::slice::from_raw_parts(*other_ptr as *const u8,*other_len as usize)};
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
				}
			}
		}
	}
}

impl<'a> Eq for CapeStringHashKey<'a> {}

impl<'a> std::hash::Hash for CapeStringHashKey<'a> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			CapeStringHashKey::Owned(data) => {
				let (my_ptr,my_len)=data.as_capechar_const_with_length();
				let slice=unsafe{std::slice::from_raw_parts(my_ptr as *const u8,my_len as usize)};
				let str=String::from_utf8_lossy(slice);
				let it=str.chars();
				for c in it {
					c.hash(state); //already lower case
				}
			}
			CapeStringHashKey::Borrowed(my_ptr,my_len,_) => {
				let slice=unsafe{std::slice::from_raw_parts(*my_ptr as *const u8,*my_len as usize)};
				let str=String::from_utf8_lossy(slice);
				let it=str.chars();
				for c in it {
					c.to_lowercase().next().unwrap().hash(state);
				}
			}
		}
	}
}
