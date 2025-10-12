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
	MessageWithCause(String,CapeError),
	Code(CapeResult),
	CAPEOPEN(CapeError),
}

impl COBIAError {
	pub fn as_code(&self) -> CapeResult {
		//in case we can only return an error code, so COBIAERR_CAPEOPENERROR is not valid here
		match self {
			COBIAError::Code(code) => *code,
			_ => COBIAERR_UNKNOWNERROR,
		}
	}
	pub fn from_object<T:cape_smart_pointer::CapeSmartPointer>(code:CapeResult,object:&T) -> Self {
		match code {
			COBIAERR_CAPEOPENERROR => {
				//object contains the last error
				let e=object.last_error();
				match e {
					Some(err) => COBIAError::CAPEOPEN(err),
					None => COBIAError::Code(code)
				}
			},
			_ => COBIAError::Code(code)
		}
	}
	pub fn from_cape_interface_pointer(code:CapeResult,interface:*mut crate::C::ICapeInterface) -> Self {
		match code {
			COBIAERR_CAPEOPENERROR => {
				//object contains the last error
				let mut err_interface : * mut C::ICapeError=std::ptr::null_mut();
				let res=unsafe {
					((*(*(interface as *mut C::ICapeInterface)).vTbl).getLastError.unwrap())((*interface).me,&mut err_interface as *mut *mut C::ICapeError)
				};
				if res==COBIAERR_NOERROR {
					if err_interface.is_null() {
						COBIAError::Code(COBIAERR_NULLPOINTER)
					} else {
						COBIAError::CAPEOPEN(CapeError::attach(err_interface))
					}
				} else {
					COBIAError::Code(COBIAERR_UNKNOWNERROR)
				}
			},
			_ => COBIAError::Code(code)
		}
	}
}

impl fmt::Display for COBIAError {
	/// Formats the COBIA error using the given formatter.
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			COBIAError::Message(message) => write!(f, "{}", message),
			COBIAError::MessageWithCause(message, cause) => write!(f, "{}, caused by {}", message,cause),
			COBIAError::Code(code) => {
				let mut s = CapeStringImpl::new();
				let res = unsafe { C::capeGetErrorDescription(*code, &mut s.as_cape_string_out() as *mut C::ICapeString) };
				if res == COBIAERR_NOERROR {
					write!(f, "{}", s)
				} else {
					write!(f, "COBIA error code: {}", code)
				}
			},
			COBIAError::CAPEOPEN(cape_error) => write!(f, "{}", cape_error),
		}
	}
}

impl std::fmt::Debug for COBIAError {
	/// Formats the COBIA error using the given formatter.
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		std::fmt::Display::fmt(self, f)
	}
}

/// Implements the Error trait for COBIAError
impl error::Error for COBIAError {
	//fn description(&self) -> &str is deprecated in favor of Display
	//fn cause(&self) -> Option<&dyn error::Error> is deprecated in favor of source
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		None
	}
}

impl Into<String> for COBIAError {
	fn into(self) -> String {
		format!("{}",self)
	}
}
