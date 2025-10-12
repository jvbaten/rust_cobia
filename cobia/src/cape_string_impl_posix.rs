use crate::cape_data_traits::*;
use crate::C;
use crate::cape_result_value::*;
use std::fmt;

/// Default ICapeString implementation
///
/// For COBIA, strings go over the pipeline as null terminated.
/// For most systems, COBIA requires UTF-8 encoding 
///
/// Rust uses non-null-terminated strings that are UTF-8 encoded.
///
/// Because this translation is always required, there is no 
/// read-only string implementation that refers to a string slice.
///
/// This implementation uses a `String` to store the string data, 
/// and the null character is explicitly added to the string.
///

#[derive(Debug,Clone)]
pub struct CapeStringImpl {
	data: String, //always ends in a null character
}

impl CapeStringImpl {

	///Default constructor
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::new();
	/// assert_eq!(s.as_string(),"");
	/// ```
	pub fn new() -> Self {
		CapeStringImpl {
			data: "\0".to_string(),
		}
	}
	///Construct from string
	///
	/// # Arguments
	///
	/// * `s` - A string slice to be converted to a CapeStringImpl
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// ```
	pub fn from_string<T:AsRef<str>>(s: T) -> Self {
		let s=s.as_ref();
		let mut data=String::with_capacity(s.len()+1);
		data.push_str(s);
		data.push('\0');
		CapeStringImpl {
			data
		}
	}
	///Construct from raw data
	///
	/// # Arguments
	///
	/// * `data` - A pointer to the string data, with null terminated character
	/// * `size` - The size of the string data, exlucing the null terminated character
	///
	pub unsafe fn from_raw_data(data: *const C::CapeCharacter, size: C::CapeSize) -> Self {
		CapeStringImpl {
			data:String::from_utf8_lossy(unsafe{std::slice::from_raw_parts(data as *const u8, (size as usize)+1)}).into()
		}
	}

	///Return as string
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(s.as_string(),"idealGasEnthalpy");
	/// ```
	pub fn as_string(&self) -> String {
		self.data[..self.data.len() - 1].into()
	}

	///Set string
	///
	/// # Arguments
	///
	/// * `s` - Any CAPE-OPEN string type
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let mut s=cobia::CapeStringImpl::new();
	/// s.set(&cobia::CapeStringImpl::from_string("idealGasEnthalpy"));
	/// assert_eq!(s.as_string(),"idealGasEnthalpy");
	/// ```

	pub fn set<T: CapeStringConstProvider>(&mut self, s: &T) {
		let(ptr,size)=s.as_capechar_const_with_length();
		self.data.clear();
		self.data.reserve(size as usize + 1);
		self.data.extend(String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(ptr as *const u8, size as usize) }).chars());
		self.data.push('\0');
	}

	/// Check empty
	///
	/// Returns true if the string is empty (no data, or only the null terminator).
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::new();
	/// assert_eq!(s.is_empty(),true);
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(s.is_empty(),false);
	/// ```
	pub fn is_empty(&self) -> bool {
		self.data.as_bytes()[0]==0u8
	}

	///Set string
	///
	/// # Arguments
	///
	/// * `s` - A string slice to be set
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let mut s=cobia::CapeStringImpl::new();
	/// s.set_string("idealGasEnthalpy");
	/// assert_eq!(s.as_string(),"idealGasEnthalpy");
	/// ```

	pub fn set_string<T: AsRef<str>>(&mut self, s: T) {
		let s=s.as_ref();
		self.data.clear();
		self.data.reserve(s.len()+1);
		self.data.extend(s.chars());
		self.data.push('\0');
	}

	extern "C" fn string_get(
		me: *mut ::std::os::raw::c_void,
		data: *mut *const C::CapeCharacter,
		size: *mut C::CapeSize,
	) {
		let p = me as *mut CapeStringImpl;
		let s: &mut CapeStringImpl = unsafe { &mut *p };
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
		let p = me as *mut CapeStringImpl;
		let s: &mut CapeStringImpl = unsafe { &mut *p };
		s.data.clear();
		let new_size=(size as usize)+1;
		if new_size>s.data.capacity() {
			s.data.reserve_exact(new_size-s.data.capacity());
		}
		if size > 0 {
			s.data.extend(String::from_utf8_lossy(&unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) }).chars());
		}
		s.data.push('\0'); //null terminator
		COBIAERR_NOERROR
	}

	const CAPE_STRING_VTABLE: C::ICapeString_VTable = C::ICapeString_VTable {
		get: Some(CapeStringImpl::string_get),
		set: Some(CapeStringImpl::string_set),
	};

	/// Convert to lower case.
	///
	/// Most CAPE-OPEN string comparisons are case insensitive. By allowing converson to lower
	/// case, we can use this in conjuntion with Eq, PartialEq, Hash, etc to make lookup tables
	/// and comparisons without the need to convert to Rust's utf-8 encoded string (on Windows,
	/// CapeString is UTF-16 encoded and null terminated).
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(s.to_lowercase(),cobia::CapeStringImpl::from_string("idealgasenthalpy"));
	/// ```

	pub fn to_lowercase(&self) -> Self {
		CapeStringImpl {
			data: self.data.to_lowercase(),
		}
	}

	/// Convert to upper case.
	///
	/// Most CAPE-OPEN string comparisons are case insensitive. By allowing converson to upper
	/// case, we can use this in conjuntion with Eq, PartialEq, Hash, etc to make lookup tables
	/// and comparisons without the need to convert to Rust's utf-8 encoded string (on Windows,
	/// CapeString is UTF-16 encoded and null terminated).
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(s.to_uppercase(),cobia::CapeStringImpl::from_string("IDEALGASENTHALPY"));
	/// ```

	pub fn to_uppercase(&self) -> Self {
		CapeStringImpl {
			data: self.data.to_uppercase(),
		}
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
	/// use cobia;
	/// let s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// let s2=cobia::CapeStringImpl::from_string("IDEALGASENTHALPY");
	/// assert_eq!(s1.eq_ignore_case(&s2),true);
	/// ```
	pub fn eq_ignore_case<T:CapeStringConstProvider>(&self, other: &T) -> bool {
		let mut it1=self.data.chars();
		let (other_ptr,other_len)=other.as_capechar_const_with_length();
		let other_slice=unsafe{std::slice::from_raw_parts(other_ptr as *const u8,other_len as usize)};
		let other_str=String::from_utf8_lossy(other_slice);
		let mut it2=other_str.chars();
		loop {
			match (it1.next(),it2.next()) {
				(Some(c1),Some(c2)) => {
					if c1.to_lowercase().next().unwrap() != c2.to_lowercase().next().unwrap() {
						return false;
					}
				}
				(Some(c1),None) => return (c1=='\0')&&(it1.next()==None),
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
	/// use cobia;
	/// let s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// let s2=cobia::CapeStringImpl::from_string("IDEALGASENTHALPY");
	/// assert_eq!(s1.eq(&s2),false);
	/// let s3=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(s1.eq(&s3),true);
	/// ```
	pub fn eq<T:CapeStringConstProvider>(&self, other: &T) -> bool {
		let mut it1=self.data.chars();
		let (other_ptr,other_len)=other.as_capechar_const_with_length();
		let other_slice=unsafe{std::slice::from_raw_parts(other_ptr as *const u8,other_len as usize)};
		let other_str=String::from_utf8_lossy(other_slice);
		let mut it2=other_str.chars();
		loop {
			match (it1.next(),it2.next()) {
				(Some(c1),Some(c2)) => {
					if c1 != c2 {
						return false;
					}
				}
				(Some(c1),None) => return (c1=='\0')&&(it1.next()==None),
				(None,Some(_)) => return false,
				(None,None) => return true,
			}
		}
	}
}

impl CapeStringConstProvider for CapeStringImpl {
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
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy"); //must remain in scope....
	/// let (ptr,len)=s.as_capechar_const_with_length(); ///... while ptr is used
	/// assert_eq!(len,16);
	/// ```
	fn as_capechar_const_with_length(&self) -> (*const C::CapeCharacter, C::CapeSize) {
		(self.data.as_ptr() as *const C::CapeCharacter, (self.data.len() - 1) as C::CapeSize) //length without the terminating null
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
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy"); //must remain in scope....
	/// let ptr=s.as_capechar_const(); ///... while ptr is used
	/// assert_eq!(unsafe{*ptr},'i' as i8);
	/// ```
	fn as_capechar_const(&self) -> *const C::CapeCharacter {
		self.data.as_ptr() as *const C::CapeCharacter
	}
}

impl fmt::Display for CapeStringImpl {
	/// Formats the CapeStringImpl error using the given formatter.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(format!("{}",s),"idealGasEnthalpy");
	/// ```
	///
	/// ```
	/// use cobia;
	/// let s=cobia::CapeStringImpl::new();
	/// assert_eq!(format!("{}",s),"");
	/// ```
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", &self.data[..(self.data.len() - 1)])
	}
}

impl CapeStringProviderIn for CapeStringImpl {
	fn as_cape_string_in(&self) -> C::ICapeString {
		C::ICapeString {
			vTbl:(&CapeStringImpl::CAPE_STRING_VTABLE as *const C::ICapeString_VTable).cast_mut(),
			me:(self as *const CapeStringImpl).cast_mut() as *mut ::std::os::raw::c_void
		}
	}
}

impl CapeStringProviderOut for CapeStringImpl {
	fn as_cape_string_out(&mut self) -> C::ICapeString {
		C::ICapeString {
			vTbl:(&CapeStringImpl::CAPE_STRING_VTABLE as *const C::ICapeString_VTable).cast_mut(),
			me:(self as *const CapeStringImpl).cast_mut() as *mut ::std::os::raw::c_void
		}
	}
}

impl<T:CapeStringConstProvider> PartialEq<T> for CapeStringImpl {

	/// Case sensitive comparison
	///
	/// # Arguments
	/// * other - The other string to compare to
	///
	/// # Return
	/// * true if the strings are equal, false otherwise.
	///
	/// # Examples
	/// 
	/// ```
	/// use cobia;
	/// let s1=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// let s2=cobia::CapeStringImpl::from_string("idealGasEnthalpy");
	/// assert_eq!(s1==s2,true);
	/// let s3=cobia::CapeStringImpl::from_string("IdealGasEnthalpy");
	/// assert_eq!(s1==s3,false);
	/// ```
	fn eq(&self, other: &T) -> bool {
		let (ptr,len)=other.as_capechar_const_with_length();
		let mut self_len:usize=self.data.len();
		if self_len>0 {self_len-=1;}
		let len=len as usize;
		if self_len == len {
			let mut ptr1=ptr;
			let bytes=self.data.as_bytes();
			for i in 0..(len as usize) {
				if unsafe { *ptr1 } as u8 != bytes[i] {
					return false;
				}
				ptr1=unsafe { ptr1.add(1) };
			}
			return true;			
		}
		false
	}
}

impl Eq for CapeStringImpl {
}

impl std::hash::Hash for CapeStringImpl {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.data.hash(state);
	}
}