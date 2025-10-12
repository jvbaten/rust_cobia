#[cfg_attr(not(target_os="windows"), path = "cape_string_const_posix.rs")]
#[cfg_attr(target_os="windows", path = "cape_string_const_win32.rs")]
mod cape_string_impl_;
pub use cape_string_impl_::*;

impl<'a,T:AsRef<str>> From<T> for CapeStringHashKey<'a> {
	fn from(s: T) -> Self {
		CapeStringHashKey::from_string(s)
	}
}

impl<'a> Clone for CapeStringHashKey<'a> {
	fn clone(&self) -> Self {
		match self {
			CapeStringHashKey::Owned(data) => CapeStringHashKey::Owned(data.clone()),
			CapeStringHashKey::Borrowed(ptr,len,_) => CapeStringHashKey::Borrowed(*ptr,*len,std::default::Default::default())
		}
	}
}