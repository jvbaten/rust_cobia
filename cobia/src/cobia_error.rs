use crate::*;

/// A COBIA error, with description
///
/// Many functions in this module return a `Result` with the error
/// type set to `COBIAError`. A `COBIAError` can be constructed from
///
/// - a string message, for an internal error message
/// - a `CapeResult` error code, for an internal message that corresponds to a predefined CAPE-OPEN error code
/// - a `CapeError`, for an error that was returned by an external CAPE-OPEN component
/// - a string message and a `CapeError`, for an internal error message that was caused by an external CAPE-OPEN component
///
/// The `COBIAError` type implements the `Error` trait, so it can be used in functions that return `Result`.
/// The `COBIAError` type also implements the `Display` trait, so it can be formatted as a string.
#[derive(Clone)]
pub enum COBIAError {
	Message(String),
	MessageWithCause(String, CapeError),
	Code(CapeResult),
	CAPEOPEN(CapeError),
}

impl COBIAError {

	/// Returns the CAPE-OPEN error code corresponding to this error.
	///
	/// If this error is a `COBIAError::Code`, the stored code is returned.
	/// For all other variants, `COBIAERR_UNKNOWNERROR` is returned, since
	/// no single error code can represent them.
	pub fn as_code(&self) -> CapeResult {
		//in case we can only return an error code, so COBIAERR_CAPEOPENERROR is not valid here
		match self {
			COBIAError::Code(code) => *code,
			_ => COBIAERR_UNKNOWNERROR,
		}
	}
	/// Constructs a `COBIAError` from a CAPE-OPEN result code and a smart-pointer object.
	///
	/// If `code` is `COBIAERR_CAPEOPENERROR`, the last error is retrieved from `object`
	/// and a `COBIAError::CAPEOPEN` is returned. Otherwise a `COBIAError::Code` is returned.
	pub fn from_object<T: cape_smart_pointer::CapeSmartPointer>(
		code: CapeResult,
		object: &T,
	) -> Self {
		match code {
			COBIAERR_CAPEOPENERROR => {
				//object contains the last error
				let e = object.last_error();
				match e {
					Some(err) => COBIAError::CAPEOPEN(err),
					None => COBIAError::Code(code),
				}
			}
			_ => COBIAError::Code(code),
		}
	}
	/// Constructs a `COBIAError` from a CAPE-OPEN result code and a raw `ICapeInterface` pointer.
	///
	/// If `code` is `COBIAERR_CAPEOPENERROR`, `getLastError` is called on `interface` to
	/// retrieve the underlying `CapeError` and a `COBIAError::CAPEOPEN` is returned.
	/// Otherwise a `COBIAError::Code` is returned.
	///
	/// # Safety
	/// `interface` must be a valid, non-null pointer to a live `ICapeInterface` object for
	/// the duration of this call.
	pub unsafe fn from_cape_interface_pointer(
		code: CapeResult,
		interface: *mut crate::C::ICapeInterface,
	) -> Self {
		match code {
			COBIAERR_CAPEOPENERROR => {
				//object contains the last error
				let mut err_interface: *mut C::ICapeError = std::ptr::null_mut();
				let res = unsafe {
					((*(*(interface as *mut C::ICapeInterface)).vTbl)
						.getLastError
						.unwrap())(
						(*interface).me,
						&mut err_interface as *mut *mut C::ICapeError,
					)
				};
				if res == COBIAERR_NOERROR {
					if err_interface.is_null() {
						COBIAError::Code(COBIAERR_NULLPOINTER)
					} else {
						COBIAError::CAPEOPEN(unsafe { CapeError::attach(err_interface) })
					}
				} else {
					COBIAError::Code(COBIAERR_UNKNOWNERROR)
				}
			}
			_ => COBIAError::Code(code),
		}
	}
}

/// Formats a `COBIAError` as a human-readable string.
///
/// - `COBIAError::Message` — the message is written directly.
/// - `COBIAError::MessageWithCause` — written as `"<message>, caused by <cause>"`.
/// - `COBIAError::Code` — the CAPE-OPEN description for the code is looked up via
///   `capeGetErrorDescription`; if that fails the numeric code is written instead.
/// - `COBIAError::CAPEOPEN` — delegates to the `CapeError` `Display` implementation.
impl fmt::Display for COBIAError {
	/// Formats the COBIA error using the given formatter.
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			COBIAError::Message(message) => write!(f, "{}", message),
			COBIAError::MessageWithCause(message, cause) => {
				write!(f, "{}, caused by {}", message, cause)
			}
			COBIAError::Code(code) => {
				let mut s = CapeStringImpl::new();
				let res = unsafe {
					C::capeGetErrorDescription(
						*code,
						&mut s.as_cape_string_out() as *mut C::ICapeString,
					)
				};
				if res == COBIAERR_NOERROR {
					write!(f, "{}", s)
				} else {
					write!(f, "COBIA error code: {}", code)
				}
			}
			COBIAError::CAPEOPEN(cape_error) => write!(f, "{}", cape_error),
		}
	}
}

/// Formats a `COBIAError` for debugging purposes by delegating to `fmt::Display`.
impl std::fmt::Debug for COBIAError {
	/// Formats the COBIA error using the given formatter.
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		std::fmt::Display::fmt(self, f)
	}
}

/// Implements the `std::error::Error` trait for `COBIAError`.
impl error::Error for COBIAError {
	//fn description(&self) -> &str is deprecated in favor of Display
	//fn cause(&self) -> Option<&dyn error::Error> is deprecated in favor of source
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		None
	}
}

/// Converts a `COBIAError` into a `String` using its `fmt::Display` representation.
impl From<COBIAError> for String {
	fn from(val: COBIAError) -> Self {
		format!("{}", val)
	}
}
