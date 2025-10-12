#[cfg_attr(not(target_os="windows"), path = "cape_string_impl_posix.rs")]
#[cfg_attr(target_os="windows", path = "cape_string_impl_win32.rs")]
mod cape_string_impl_;

pub use cape_string_impl_::*;

impl<T:AsRef<str>> From<T> for CapeStringImpl {
	fn from(s: T) -> Self {
		CapeStringImpl::from_string(s)
	}
}

impl std::default::Default for CapeStringImpl {
	fn default() -> Self {
		CapeStringImpl::new()
	}
}