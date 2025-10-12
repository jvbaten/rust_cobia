use crate::C;
use crate::*;

/// Error description from CAPE-OPEN object
///
/// Each CAPE-OPEN interface has an getLastError member function that returns
/// an error interface for any call that results COBIAERR_CAPEOPENERROR.
///
/// This is a smart pointer to manage the error object and call its members.

pub struct CapeError {
	pub(crate) interface: *mut C::ICapeError,
}

impl CapeError {

	/// Create a new CapeError from an ICapeError interface pointer.
	///
	/// The CapeError object is not typically used directly. Errors
	/// are managed through the COBIAError object, which also manages
	/// the CapeError based errors raised by CAPE-OPEN objects.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeError interface
	///
	/// # Safety
	///
	/// The interface pointer must be non-null and must point to an object
	/// that implements the ICapeError interface.
	///
	/// # Panics
	///
	/// Panics if the interface pointer is null.

	pub fn from_interface_pointer(interface: *mut C::ICapeError) ->  Self {
		if interface.is_null() {
			panic!("Null pointer in creation of CapeError");
		}
		unsafe {((*(*interface).vTbl).base.addReference.unwrap())((*interface).me)};
		Self {
			interface
		}
	}

	/// Create a new CapeError from an ICapeError interface pointer without adding a reference.
	///
	/// The CapeError object is not typically used directly. Errors
	/// are managed through the COBIAError object, which also manages
	/// the CapeError based errors raised by CAPE-OPEN objects.
	///
	/// # Arguments
	///
	/// * `interface` - A pointer to an ICapeError interface
	///
	/// # Safety
	///
	/// The interface pointer must be non-null and must point to an object
	/// that implements the ICapeError interface.
	///
	/// # Panics
	///
	/// Panics if the interface pointer is null.

	pub fn attach(interface: *mut C::ICapeError) ->  Self {
		if interface.is_null() {
			panic!("Null pointer in creation of CapeError");
		}
		Self {
			interface
		}
	}

	/// Get the error text
	///
	/// Gets the error text from the error interface. The error text
	/// is descriptive information about the nature of the error.

	pub fn get_error_text(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getErrorText.unwrap())((*self.interface).me,(&s.as_cape_string_out() as *const C::ICapeString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get the error that caused this error.
	///
	/// Sometimes an error is casued by another error. If so, the
	/// error that causes this error is available through this function.

	pub fn get_cause(&self) -> Option<CapeError> {
		let mut interface: *mut C::ICapeError = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).getCause.unwrap())(
				(*self.interface).me,
				&mut interface as *mut *mut C::ICapeError
			)
		};
		if result != COBIAERR_NOERROR || interface.is_null() {
			None			
		} else {
			Some(CapeError { interface })
		}
	}

	/// Get the error source
	///
	/// The error source is a descriptive name of the object that
	/// raised the error. 

	pub fn get_source(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getSource.unwrap())((*self.interface).me,(&s.as_cape_string_out() as *const C::ICapeString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get the error scope
	///
	/// The error scope is the function that was being executed when
	/// the error was raised.

	pub fn get_scope(&self) -> Result<String, COBIAError> {
		let mut s = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getScope.unwrap())((*self.interface).me,(&s.as_cape_string_out() as *const C::ICapeString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(s.as_string())
		} else {
			Err(COBIAError::Code(result))
		}
	}
	
}

/// Display 

impl std::fmt::Display for CapeError {

	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mut err:Option<CapeError> = Some(self.clone());
		while let Some(e)=err {
			match e.get_scope() {
				Ok(scope) => {
					match e.get_source() {
						Ok(source) => {
							write!(f, "in {} of {}: ", scope, source)?;
						},
						_ => {
							write!(f, "in {}", scope)?;
						}
					}
				},
				_ => {
					match e.get_source() {
						Ok(source) => {
							write!(f, "{}: ", source)?;
						},
						_ => {}
					}
				}
			}
			match e.get_error_text() {
				Ok(text) => {
					write!(f, "{}", text)?;
				},
				_ => {
					write!(f, "Unknown error")?;
				}
			}
			err=e.get_cause();
			if err.is_some() {
				write!(f, ", caused by: ")?;
			}
		}
		Ok(())
	}
}

/// Release pointer
///
/// ICapeError derives from ICobiaBase, which contains
/// addReference() and release(). The Drop trait calls release.

impl Drop for CapeError {
	fn drop(&mut self) {
		unsafe {
			((*(*self.interface).vTbl).base.release.unwrap())((*self.interface).me);
		}
	}
}

/// Add pointer reference
///
/// ICapeError derives from ICobiaBase, which contains
/// addReference() and release(). The Clone trait calls addReference.

impl Clone for CapeError {
	fn clone(&self) -> Self {
		unsafe {
			((*(*self.interface).vTbl).base.addReference.unwrap())((*self.interface).me);
		}
		CapeError {
			interface: self.interface,
		}
	}
}

