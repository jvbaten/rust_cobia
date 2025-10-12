use crate::*;
use crate::C::*;

/// COBIA Smart Pointer
/// 
/// External objects implement interfaces that all derive
/// from ICapeInterface.
///
/// Objects that provide wrappers for such interfaces also
/// typically maintain the reference count for the underlying
/// object.
///
/// An object can implement multiple interfaces, and one
/// interface wrapper can be converted to that for another
/// wrapper provided that the object implements the 
/// corresponding interface

pub trait CapeSmartPointer {
	type Interface;
	/// Get the interface pointer
	///
	/// This function provides the interface wrapped by the object.
	fn as_interface_pointer(&self) -> *mut Self::Interface;
	/// Get the ICapeInterface
	///
	/// All CAPE-OPEN interfaces are laid out such that the 
	/// first fields correspond to the field of ICapeInterface
	/// and therefore all interfaces can safely be cast to 
	/// ICapeInterface.
	///
	/// This function provides the ICapeInterface representation
	/// of the interface wrapped by the object.
	fn as_cape_interface_pointer(&self) -> *mut C::ICapeInterface;
	/// Get the interface ID
	///
	/// All interfaces are identified by a unique ID, which is 
	/// passed to the queryInterface member function of 
	/// ICapeInterface to obtain a pointer to that interface.
	///
	/// This function exposes the interface ID for the interface
	/// that is wrapped.
	fn get_interface_id() -> &'static CapeUUID;
	/// Get an interface wrapper instance from another object
	///
	/// This function provides an interface wrapper from any 
	/// object that implements ICapeInterface, provided that the 
	/// underlying object implements the interface.
	///
	/// This function performs a queryInterface on the 
	/// object that is passed as argument, with the interface
	/// ID returned by as_cape_interface_pointer()
	fn from_object<T:CapeSmartPointer>(smart_pointer : &T) -> Result<Self,COBIAError>  where Self: Sized;
	/// Get an interface wrapper instance from an interface pointer of the wrapped type
	///
	/// This function provides an interface wrapper directly from 
	/// the interface pointer. 
	/// 
	/// #Safety
	///
	/// The interface pointer must be valid and must point to an object
	/// that implements the interface.
	///
	/// #Panics
	///
	/// Panics if the interface pointer is null.
	fn from_interface_pointer(interface : *mut Self::Interface) -> Self;
	/// Get an interface wrapper instance from an interface pointer of the wrapped type, without adding a reference
	///
	/// This function provides an interface wrapper directly from 
	/// the interface pointer. Typical use it to attach a return 
	/// value from an external function, which must be released by the 
	/// receiver, to the smart pointer.
	/// 
	/// #Safety
	///
	/// The interface pointer must be valid and must point to an object
	/// that implements the interface.
	///
	/// #Panics
	///
	/// Panics if the interface pointer is null.
	fn attach(interface : *mut Self::Interface) -> Self;
	/// Return an interface pointer and release ownership, without decreasing a reference
	///
	/// This function releases ownership of the object by returning
	/// the contained pointer. The caller is responsible for releasing the
	/// object. Typical use is to return the pointer from a function that 
	/// is exposed externally.
	fn detach(self) -> *mut Self::Interface;
	/// Get an interface wrapper instance from any interface pointer
	///
	/// This function provides an interface wrapper directly from 
	/// any CAPE-OPEN interface pointer by performing a queryInterface.
	///
	/// If the object does not implement the interface, the function
	/// returns an error. If the interface pointer is null, the function
	/// returns an error.
	///
	/// Any CAPE-OPEN interface can be cast to ICapeInterface. This is a
	/// reinterpret-cast and therefore an unsafe operation.
	/// 
	/// #Safety
	///
	/// If non-null, the interface pointer must be valid and must point 
	/// to an object that implements the interface.
	///
	fn from_cape_interface_pointer(interface : *mut C::ICapeInterface) -> Result<Self,COBIAError>  where Self: Sized;
	/// Get the last error
	///
	/// The last error is available after a function call that
	/// returns a COBIAERR_CAPEOPENERROR. This function calls
	/// getLastError on ICapeInterface to obtain the last error.
	fn last_error(&self) -> Option<CapeError>;
}
