use crate::C;
use crate::*;
use cape_smart_pointer::CapeSmartPointer;

const ICOBIAIDENTIFICATION_UUID:CapeUUID=CapeUUID::from_slice(&[0xa9u8,0xb3u8,0x9eu8,0x4au8,0x7du8,0x48u8,0x40u8,0x81u8,0xa1u8,0x7fu8,0x65u8,0x22u8,0x82u8,0x40u8,0x85u8,0xb5u8]);

/// CobiaIdentification interface smart pointer
///
/// Smart pointer for ICobiaIdentification interface.
///
/// ICobiaIdentification is typically implemented 
/// in objects in ICobiaCollections.

#[cape_smart_pointer(ICOBIAIDENTIFICATION_UUID)]
pub struct CobiaIdentification {
	pub(crate) interface: *mut C::ICobiaIdentification,
}

impl CobiaIdentification {

	/// Create a new CobiaIdentification from an interface pointer
	/// 
	/// Not typically called directly. Used by CapeSmartPointer.
	///
	/// # Safety
	///
	/// The interface pointer must be valid and must point to an object
	/// that implements the ICobiaIdentification interface.
	///
	/// # Panics
	///
	/// Panics if the interface pointer is null.
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let libraries = library_enumerator.libraries().unwrap();
	/// if libraries.size() > 0 {
	/// 	let library =  libraries.at(0).unwrap();
	/// 	println!("Library name: {}", library.get_name().unwrap());
	///     //create CobiaIdentification from library, undirectly calls from_interface_pointer()
	/// 	let iden = cobia::CobiaIdentification::from_object(&library).unwrap(); 
	/// 	println!("Library name: {}", iden.get_component_name().unwrap());
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub(crate) fn from_interface_pointer(interface: *mut C::ICobiaIdentification) ->  Self {
		if interface.is_null() {
			panic!("Null pointer in creation of CobiaIdentification");
		}
		unsafe {((*(*interface).vTbl).base.addReference.unwrap())((*interface).me)};
		Self {
			interface
		}
	}

	/// Create a new CobiaIdentification from an interface pointer without adding a reference
	/// 
	/// Not typically called directly. Used by CapeSmartPointer.
	///
	/// # Safety
	///
	/// The interface pointer must be valid and must point to an object
	/// that implements the ICobiaIdentification interface.
	///
	/// # Panics
	///
	/// Panics if the interface pointer is null.

	pub(crate) fn attach(interface: *mut C::ICobiaIdentification) ->  Self {
		if interface.is_null() {
			panic!("Null pointer in creation of CobiaIdentification");
		}
		Self {
			interface
		}
	}

	/// Get the component name
	/// 
	/// Gets the name of the component
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let libraries = library_enumerator.libraries().unwrap();
	/// if libraries.size() > 0 {
	/// 	let library =  libraries.at(0).unwrap();
	/// 	println!("Library name: {}", library.get_name().unwrap());
	/// 	let iden = cobia::CobiaIdentification::from_object(&library).unwrap();
	/// 	println!("Library name: {}", iden.get_component_name().unwrap());
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_component_name(&self) -> Result<String, COBIAError> {
		let mut name = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getComponentName.unwrap())((*self.interface).me, (&name.as_cape_string_out() as *const C::ICapeString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(name.as_string())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Set the component name
	///
	/// Sets the name of the component; typically disallowed.
	/// 
	/// Only primary PMC objects typically allow for a name change.
	///
	/// This interface however is implemented on object from ICobiaCollection.
	/// The put_component_name method is typically not implemented on objects.

	pub fn put_component_name(&self,name: &str) -> Result<(), COBIAError> {
		let name=CapeStringImpl::from(name);
		let result = unsafe {
			((*(*self.interface).vTbl).getComponentName.unwrap())((*self.interface).me, (&name.as_cape_string_in() as *const C::ICapeString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Get the component description
	/// 
	/// Gets the description of the component
	///
	/// # Example
	///
	/// ```
	/// use cobia::*;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let libraries = library_enumerator.libraries().unwrap();
	/// if libraries.size() > 0 {
	/// 	let library =  libraries.at(0).unwrap();
	/// 	println!("Library name: {}", library.get_name().unwrap());
	/// 	let iden = cobia::CobiaIdentification::from_object(&library).unwrap();
	/// 	println!("Library description: {}", iden.get_component_description().unwrap());
	/// }
	/// cobia::cape_open_cleanup();
	/// ```


	pub fn get_component_description(&self) -> Result<String, COBIAError> {
		let mut description = CapeStringImpl::new();
		let result = unsafe {
			((*(*self.interface).vTbl).getComponentDescription.unwrap())((*self.interface).me, (&description.as_cape_string_out() as *const C::ICapeString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(description.as_string())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

	/// Set the component description
	///
	/// Sets the description of the component; typically disallowed.
	/// 
	/// Only primary PMC objects typically allow for a description change.
	///
	/// This interface however is implemented on object from ICobiaCollection.
	/// The put_component_description method is typically not implemented on objects.


	pub fn put_component_description(&self,description: &str) -> Result<(), COBIAError> {
		let description=CapeStringImpl::from(description);
		let result = unsafe {
			((*(*self.interface).vTbl).getComponentDescription.unwrap())((*self.interface).me,(&description.as_cape_string_in() as *const C::ICapeString).cast_mut())
		};
		if result == COBIAERR_NOERROR {
			Ok(())
		} else {
			Err(COBIAError::from_object(result,self))
		}
	}

}

