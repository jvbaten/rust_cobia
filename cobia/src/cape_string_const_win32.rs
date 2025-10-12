use crate::{C,CapeStringImpl};
#[cfg(doc)] use crate::CapeOpenMap;
use crate::cape_data_traits::*;
use std::fmt;
use crate::cape_result_value::*;

/// Class to store a platform dependent string encoding for use in case
/// insensitive hash maps or for use of case insensitive comparisons.
///
/// For COBIA, strings go over the pipeline
///  as null terminated. For Windows, COBIA requires UTF-16 encoding.
///
/// Because this translation is always required, there is no 
/// read-only string implementation that refers to a string slice.
/// However, this string implementation is immutable and can therefore
/// be used as static.
///
/// This implementation uses a `Vec<u16>` to store the string data.
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
	data: Vec<C::CapeCharacter>,
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
		let mut data=Vec::new();
		data.reserve(s.len() + 1);
		for c in s.encode_utf16() {
			data.push(CapeStringImpl::to_lower_case(c));
		}
		data.push(0);
		CapeStringConstNoCase {
			data,
		}
	}

	///Construct from CapeCharacter pointer
	///
	/// # Arguments
	///
	/// * `ptr` - A const CapeCharacter pointer
	/// * `size` - Length of the string pointed to
	pub fn from_cape_char_const(ptr:*const C::CapeCharacter, size:C::CapeSize) -> Self {
		let mut data=Vec::new();
		let size=size as usize;
		data.reserve(size+1);
		for i in 0..size {
			let c=unsafe { *ptr.add(i as usize) };
			data.push(CapeStringImpl::to_lower_case(c));
		}
		data.push(0);
		CapeStringConstNoCase {
			data,
		}
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
					data: vec![0u16]
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
		let len = self.data.len() - 1;
		String::from_utf16_lossy(&self.data[..len])
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
		if self.data.is_empty() {
			write!(f, "")
		} else {
			let len = self.data.len() - 1;
			write!(f, "{}", String::from_utf16_lossy(&self.data[..len]))
		}
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
	/// assert_eq!(unsafe{*ptr},'i' as u16);
	/// ```
	fn as_capechar_const(&self) -> *const C::CapeCharacter {
		self.data.as_ptr()
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
		let (ptr,len)=other.as_capechar_const_with_length();
		let len=len as usize;
		if self.data.len()-1 == len {
			let mut ptr=ptr;
			for i in 0..len {
				if CapeStringImpl::to_lower_case(unsafe { *ptr }) != self.data[i] {
					return false;
				}
				ptr=unsafe { ptr.add(1) };
			}
			return true;			
		}
		false
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
		let slice=unsafe{ std::slice::from_raw_parts(data, size as usize)};
		s.data.reserve(slice.len() + 1);
		s.data.extend_from_slice(&slice);
		s.data.push(0); //null terminator
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
/// For COBIA, strings go over the pipeline
///  as null terminated. For Windows, COBIA requires UTF-16 encoding.
///
/// The CapeStringHashKey implementation uses a `CapeStringConstNoCase` 
/// to store, owned string data, or allows reference to data provided 
/// by any class that implements CapeStringConstProvider, so that 
/// a copy of the data is not needed for hash lookups.
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

	///Construct from string that owns the data
	///
	/// # Arguments
	///
	/// * `ptr` - A const CapeCharacter pointer
	/// * `size` - Length of the string pointed to
	pub fn from_cape_char_const(ptr:*const C::CapeCharacter, size:C::CapeSize) -> Self {
		CapeStringHashKey::Owned(CapeStringConstNoCase::from_cape_char_const(ptr,size))
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
			let data=unsafe { std::slice::from_raw_parts(ptr,len as usize) };
			String::from_utf16_lossy(&data)
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
			let slice = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
			write!(f, "{}", String::from_utf16_lossy(slice))
		}
	}
}

impl<'a> PartialEq for CapeStringHashKey<'a> {
	fn eq(&self, other: &Self) -> bool {
		//optimize for owned vs borrowed, owned strings are already in lower case
		match self {
			CapeStringHashKey::Owned(data) => {
				let (my_ptr,my_len)=data.as_capechar_const_with_length();
				match other {
					CapeStringHashKey::Owned(other_data) => {
						let (other_ptr,other_len)=other_data.as_capechar_const_with_length();
						if other_len==my_len {
							let mut my_ptr=my_ptr;
							let mut other_ptr=other_ptr;
							let my_ptr_end=unsafe { my_ptr.add(my_len as usize) };
							while my_ptr!=my_ptr_end {
								if unsafe { *other_ptr } != unsafe { *my_ptr } {
									return false;
								}
								my_ptr=unsafe { my_ptr.add(1) };
								other_ptr=unsafe { other_ptr.add(1) };
							}
							return true;
						} 
						false
					}
					CapeStringHashKey::Borrowed(other_ptr,other_len,_) => {
						if *other_len==my_len {
							let mut my_ptr=my_ptr;
							let mut other_ptr=*other_ptr;
							let my_ptr_end=unsafe { my_ptr.add(my_len as usize) };
							while my_ptr!=my_ptr_end {
								if CapeStringImpl::to_lower_case(unsafe { *other_ptr }) != unsafe { *my_ptr } {
									return false;
								}
								my_ptr=unsafe { my_ptr.add(1) };
								other_ptr=unsafe { other_ptr.add(1) };
							}
							return true;
						}
						false
					}
				}
			},
			CapeStringHashKey::Borrowed(my_ptr,my_len,_) => {
				let my_len=*my_len;
				match other {
					CapeStringHashKey::Owned(other_data) => {
						let (other_ptr,other_len)=other_data.as_capechar_const_with_length();
						if other_len==my_len {
							let mut my_ptr=*my_ptr;
							let mut other_ptr=other_ptr;
							let my_ptr_end=unsafe { my_ptr.add(my_len as usize) };
							while my_ptr!=my_ptr_end {
								if unsafe { *other_ptr } != CapeStringImpl::to_lower_case(unsafe { *my_ptr }) {
									return false;
								}
								my_ptr=unsafe { my_ptr.add(1) };
								other_ptr=unsafe { other_ptr.add(1) };
							}
							return true;
						} 
						false
					}
					CapeStringHashKey::Borrowed(other_ptr,other_len,_) => {
						if *other_len==my_len {
							let mut my_ptr=*my_ptr;
							let mut other_ptr=*other_ptr;
							let my_ptr_end=unsafe { my_ptr.add(my_len as usize) };
							while my_ptr!=my_ptr_end {
								if CapeStringImpl::to_lower_case(unsafe { *other_ptr }) != CapeStringImpl::to_lower_case(unsafe { *my_ptr }) {
									return false;
								}
								my_ptr=unsafe { my_ptr.add(1) };
								other_ptr=unsafe { other_ptr.add(1) };
							}
							return true;
						}
						false
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
				let mut my_ptr=my_ptr;
				let my_ptr_end=unsafe { my_ptr.add(my_len as usize) };
				while my_ptr!=my_ptr_end {
					unsafe { (*my_ptr).hash(state) }; //already lower case
					my_ptr=unsafe { my_ptr.add(1) };
				}
			}
			CapeStringHashKey::Borrowed(my_ptr,my_len,_) => {
				let mut my_ptr=*my_ptr;
				let my_ptr_end=unsafe { my_ptr.add(*my_len as usize) };
				while my_ptr!=my_ptr_end {
					CapeStringImpl::to_lower_case(unsafe { *my_ptr} ).hash(state);
					my_ptr=unsafe { my_ptr.add(1) };
				}
			}
		}
	}
}

