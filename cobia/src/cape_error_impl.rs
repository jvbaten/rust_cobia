use crate::C;
use crate::*;

/// CAPE-OPEN Error object implementation
///
/// This class implements a native CAPE-OPEN error object. The error object
///  implements ICapeError. This object is constructed when ICapeInterfaceImpl
///  is asked for its current error.

pub struct CapeErrorImpl {
	interface: C::ICapeError,
	text: String,
	scope: String,
	source: String,
	cause: Option<CapeError>, 
	reference_count: i32,
}

impl CapeErrorImpl {
	pub fn new(err: &COBIAError, scope: &str, source: &str) -> *mut C::ICapeError {
		let mut cause: Option<CapeError>=None;
		let text = match err {
			COBIAError::Message(s) => String::from(s),
			COBIAError::Code(code) => {
				let mut s = CapeStringImpl::new();
				let res = unsafe { C::capeGetErrorDescription(*code, (&s.as_cape_string_out() as *const C::ICapeString).cast_mut()) };
				if res == COBIAERR_NOERROR {
					s.to_string()
				} else {
					format!("COBIA error code: {}", code)
				}
			},
			COBIAError::CAPEOPEN(err) => {
				cause=err.get_cause();
				err.get_error_text().unwrap()
			},
			COBIAError::MessageWithCause(msg, because) => {
				cause=Some(because.clone());
				msg.clone()
			},
		};
		let err_ptr = Box::into_raw(Box::new(CapeErrorImpl { //into_raw locks the object in memory - no need to pin
			interface: C::ICapeError {
				vTbl: &CapeErrorImpl::VTABLE as *const C::ICapeError_VTable
					as *mut C::ICapeError_VTable,
				me: std::ptr::null_mut(),
			},
			text,
			source: source.to_string(),
			cause,
			scope: scope.to_string(),
			reference_count: 1, //this is the return reference
		}));
		unsafe {
			(*err_ptr).interface.me = (err_ptr as *mut CapeErrorImpl) as *mut std::ffi::c_void;
			&mut (*err_ptr).interface as *mut C::ICapeError
		}
	}

	extern "C" fn add_reference(me: *mut ::std::os::raw::c_void) {
		let p: *mut CapeErrorImpl = me as *mut CapeErrorImpl;
		unsafe {
			(*p).reference_count += 1;
		}
	}

	extern "C" fn release(me: *mut ::std::os::raw::c_void) {
		let p = me as *mut CapeErrorImpl;
		unsafe {
			(*p).reference_count -= 1;
			if (*p).reference_count == 0 {
				drop(Box::from_raw(p));
			}
		}
	}

	extern "C" fn get_error_text(
		me: *mut ::std::os::raw::c_void,
		error_text: *mut C::ICapeString,
	) -> CapeResult {
		let p = me as *mut CapeErrorImpl;
		if error_text.is_null() {
			COBIAERR_NULLPOINTER
		} else {
			let mut error_text=unsafe{*((&error_text as *const *mut crate::C::ICapeString) as *mut *mut crate::C::ICapeString)};
			let error_text=CapeStringOut::new(&mut error_text);
			match unsafe { error_text.set_string(&(*p).text) } {
				Ok(_) => COBIAERR_NOERROR,
				Err(err) => match err {
					COBIAError::Code(code) => {
						return code;
					}
					_ => {
						assert!(false);
						return COBIAERR_UNKNOWNERROR;
					}
				},
			}
		}
	}

	extern "C" fn get_cause(
		me: *mut ::std::os::raw::c_void,
		cause: *mut *mut C::ICapeError,
	) -> CapeResult {
		let p = me as *mut CapeErrorImpl;
		if let Some(err) = unsafe { &(*p).cause } {
			unsafe {
				let e=err.interface;
				*cause = e;
				(*(*e).vTbl).base.addReference.unwrap()((*e).me);
				COBIAERR_NOERROR
			}
		} else {
			COBIAERR_NOSUCHITEM
		}
	}

	extern "C" fn get_source(
		me: *mut ::std::os::raw::c_void,
		component_description: *mut C::ICapeString,
	) -> CapeResult {
		let p = me as *mut CapeErrorImpl;
		if component_description.is_null() {
			COBIAERR_NULLPOINTER
		} else {
			let mut component_description=unsafe{*((&component_description as *const *mut crate::C::ICapeString) as *mut *mut crate::C::ICapeString)};
			let component_description=CapeStringOut::new(&mut component_description);
			match unsafe { component_description.set_string(&(*p).source) } {
				Ok(_) => COBIAERR_NOERROR,
				Err(err) => match err {
					COBIAError::Code(code) => {
						return code;
					}
					_ => {
						assert!(false);
						return COBIAERR_UNKNOWNERROR;
					}
				},
			}
		}
	}

	extern "C" fn get_scope(
		me: *mut ::std::os::raw::c_void,
		error_scope: *mut C::ICapeString,
	) -> CapeResult {
		let p = me as *mut CapeErrorImpl;
		if error_scope.is_null() {
			COBIAERR_NULLPOINTER
		} else {
			let mut error_scope=unsafe{*((&error_scope as *const *mut crate::C::ICapeString) as *mut *mut crate::C::ICapeString)};
			let error_scope=CapeStringOut::new(&mut error_scope);
			match unsafe { error_scope.set_string(&(*p).scope) } {
				Ok(_) => COBIAERR_NOERROR,
				Err(err) => match err {
					COBIAError::Code(code) => {
						return code;
					}
					_ => {
						assert!(false);
						return COBIAERR_UNKNOWNERROR;
					}
				},
			}
		}
	}

	const VTABLE: C::ICapeError_VTable = C::ICapeError_VTable {
		base: C::ICobiaBase_VTable {
			addReference: Some(CapeErrorImpl::add_reference),
			release: Some(CapeErrorImpl::release),
		},
		getErrorText: Some(
			CapeErrorImpl::get_error_text
				as unsafe extern "C" fn(
					me: *mut ::std::os::raw::c_void,
					errorText: *mut C::ICapeString,
				) -> CapeResult,
		),
		getCause: Some(
			CapeErrorImpl::get_cause
				as unsafe extern "C" fn(
					me: *mut ::std::os::raw::c_void,
					cause: *mut *mut C::ICapeError,
				) -> CapeResult,
		),
		getSource: Some(
			CapeErrorImpl::get_source
				as unsafe extern "C" fn(
					me: *mut ::std::os::raw::c_void,
					componentDescription: *mut C::ICapeString,
				) -> CapeResult,
		),
		getScope: Some(
			CapeErrorImpl::get_scope
				as unsafe extern "C" fn(
					me: *mut ::std::os::raw::c_void,
					errorScope: *mut C::ICapeString,
				) -> CapeResult,
		),
	};
}



