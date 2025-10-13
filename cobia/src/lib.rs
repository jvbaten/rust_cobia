//! # Crate cobia - RUST Language binding for CO-LaN's COBIA middleware.
//!
//! CAPE-OPEN consists of a series of specifications to expand the range of application of 
//! process simulation technologies. The CAPE-OPEN specifications specify a set of software interfaces 
//! that allow plug and play inter-operability between a given process modelling environment (PME) 
//! and a third-party process modelling component (PMC).
//! 
//! The CAPE-OPEN specifications are supported by the non-profit organization CO-LaN: 
//! [http://www.colan.org](http://www.colan.org)
//! 
//! The COBIA middle-ware is a platform independent object model and request broker implementation
//! to facilitate the inter-operation of CAPE-OPEN compliant PMEs and PMCs.
//! 
//! This create provides the following:
//! - Safe Rust wrappers around the raw C FFI bindings to COBIA's API routines
//! - Rust based implementation of the CAPE-OPEN data types
//! - Rust based wrappers around externally implemented CAPE-OPEN data types
//! - A Rust code generator utility that utilizes COBIA's public type API to generate Rust bindings
//! - Rust traits representing COBIA interfaces
//! - Rust native implementations of COBIA based interfaces using objects that provide the Rust traits
//! - Smart pointers to COBIA objects
//! - Rust definitions for externall implemented COBIA objects
//! - Macros to make implementing CAPE-OPEN components in Rust
//!
//! # Prerequisites
//!
//! To compile against this create, the following prerequisites must be installed:
//! - The COBIA SDK (available for download from [https://colan.repositoryhosting.com/trac/colan_cobia/downloads](https://colan.repositoryhosting.com/trac/colan_cobia/downloads)
//! - The required ingredients for bindgen, including a valid CLANG installation, see [https://rust-lang.github.io/rust-bindgen/requirements.html](https://rust-lang.github.io/rust-bindgen/requirements.html)
//!
//! This package uses bindgen to generate Rust access to the C-API of COBIA. As with most Rust implementations that
//! refer to C/C++ code, all external access is unsafe, and marked as such in the cobia crate itself. Typically one
//! does not need to use the unsafe keyword in implementations that use cobia, with a few exceptions.
//! 
//! # Interface implementation considerations
//!
//! COBIA is a middle-ware that allows objects implemented in different languages to inter-operate. They 
//! do so through the use of reference counted interfaces, much like other middle-wares such as COM or CORBA.
//! 
//! To allow the Rust compiler to check the life time and validity of pointers, interface pointers are passed by
//! reference. A Rust object that implements one or more COBIA interface is allocated on the heap by boxing it
//! as part of its creation. Once the object is boxed, its vTable pointers are initialized to point to the correct
//! memory location. The boxed object is then converted to a raw pointer; this raw pointer is the interface pointer's
//! 'me' member, and is used by the raw interface methods to access the Rust object. The raw interface methods 
//! require Rust traits to be implemented on the Rust object, to provide the actual functionality.
//! 
//! Data that requires heap allocations (strings, arrays, ...) are handled a little differently. They too have a 
//! vTable, but at the design of this crate it was decided not to box the data objects, as the heap allocation
//! for the object itself may well not be needed and may hamper performance.
//! 
//! As a result, it is quite possible, and even likely, for data object implementations to be moved around after
//! its creation. When passed to a foreign function, the vTable for the data object is created on the fly, pointing
//! to where the data object is currently located. The reference to the vTable requires a reference to the data object
//! implementation, so Rust's life span checking is used to ensure that the data object implementation remains valid
//! during the life span of the vTable, and the vTable remains valid for the duration of the external call.
//! 
//! To further leverage Rust's safety features, data objects have In and Out versions. The In version cannot be modified
//! by the receiver, whereas the intent of the Out version is exactly to be modified. Functions that require an In data
//! type, can do so by a reference to it. Function that require an Out data type on the other hand, require a mutable reference;
//! this ensures that an Out data object is not modified unexpectedly.
//!
//! # Rust code generation
//! 
//! To generate Rust bindings for interfaces described by CAPE-OPEN IDL (either in a .cidl file or a library in the 
//! COBIA registry), the cidl.exe tool can be used. This tool is part of the current crate. This tool is used during
//! the build process to generate the bindings for the default cape_open and cape_open_1_2 modules, containing 
//! all known CAPE-OPEN interfaces in the installed COBIA SDK.
//! 
//! The cidl tool can also be used to generate bindings for custom CAPE-OPEN interfaces; this functionality is currently
//! untested, arising a business case.
//!
//! # CAPE-OPEN object implementations
//! 
//! To implement a CAPE-OPEN object in rust, one can use the `cape_object_implementation` macro. This macro takes as 
//! arguments the interfaces that are implemented by the object, as well as some optional constructor details.
//! 
//! To implement the entry points for a COBIA dynamic link library, one can use the `pmc_entry_points` macro.
//!
//! The reader is directed to the Salt Water Property Package example and the Distillation Shortcut Unit Operation
//! examples in the repository for further details.
//! 
//! Extenally implemented COBIA objects are accessed through smart pointers; an interface specific smart pointer is 
//! generated for each known COBIA interface, along with the Rust methods that represent the interface methods.
//! 
//! # Acknowledgements
//!
//! This package uses process_path, to get the path of the currently executing process or dynamic library, (C) Copyright Wesley Wiser and process_path contributors
//!
//! This package uses bindgen, to automatically generates Rust FFI bindings to C and C++ libraries, (C) Jyun-Yan You

//todo: build bindings for custom interfaces using cidl.exe and update above documentation
//todo: IDL interfaces 
//todo: Marshal interfaces (for a C++ client these are not directly exposed, as they are used via implementation classes that provide this functionality)
//todo: CapeResult cobiaDepersistFromTransitionFormat(ICapeInterface *reader,ICapeInterface **transitionFormat,CapeInteger majorVersion,CapeInteger minorVersion);
//todo: CapeResult cobiaDepersistPMCFromTransitionFormat(ICapeInterface *PMC, ICapeInterface *reader,CapeInteger majorVersion,CapeInteger minorVersion);

//To build documentation: 
//  cargo doc --no-deps --package cobia --open

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]
pub mod C;

mod cobia_error;
pub use cobia_error::COBIAError;
mod cape_data_traits;
pub mod prelude;
pub use cape_data_traits::*;
mod cape_data_from_provider;
pub use cape_data_from_provider::*;
mod cape_array_object_vec;
#[cfg(doc)] pub use cape_array_object_vec::CapeArrayObjectVec;
pub use cape_array_object_vec::{CapeArrayStringVec,CapeArrayValueVec};
mod cobia_collection;
pub use cobia_collection::CobiaCollection;
mod cobia_identification;
pub use cobia_identification::CobiaIdentification;
pub use C::CapeBoolean;
pub use C::CapeReal;
pub use C::CapeInteger;
pub use C::CapeResult;
pub use C::CapeCharacter;
pub use C::CapeByte;
pub use C::CapeEnumeration;
#[cfg_attr(not(target_os="windows"), path = "cape_string_posix.rs")]
#[cfg_attr(target_os="windows", path = "cape_string_win32.rs")]
mod cape_string;
pub use cape_string::{CapeStringIn,CapeStringOut};
mod cape_string_impl;
pub use cape_string_impl::CapeStringImpl;
mod cape_string_const;
pub use cape_string_const::CapeStringConstNoCase;
pub use cape_string_const::CapeStringHashKey;
mod cape_string_map;
pub use cape_string_map::CapeOpenMap;
mod cape_array_vec;
#[cfg(doc)] pub use cape_array_vec::CapeArrayVec;
pub use cape_array_vec::{CapeArrayRealVec,CapeArrayIntegerVec,CapeArrayByteVec,CapeArrayBooleanVec,CapeArrayEnumerationVec};
mod cape_array_slice;
#[cfg(doc)] pub use cape_array_slice::CapeArraySlice;
pub use cape_array_slice::{CapeArrayRealSlice,CapeArrayIntegerSlice,CapeArrayByteSlice,CapeArrayBooleanSlice,CapeArrayEnumerationSlice};
mod cape_array_real_scalar;
pub use cape_array_real_scalar::CapeArrayRealScalar;
pub mod cape_value_impl;
pub use cape_value_impl::{CapeValueImpl,CapeValueContent};
mod cape_value;
pub use cape_value::{CapeValueIn,CapeValueOut};
mod cape_array;
#[cfg(doc)] pub use cape_array::{CapeArrayIn,CapeArrayOut};
mod cape_array_real;
pub use cape_array_real::{CapeArrayRealIn,CapeArrayRealOut};
mod cape_array_integer;
pub use cape_array_integer::{CapeArrayIntegerIn,CapeArrayIntegerOut};
mod cape_array_byte;
pub use cape_array_byte::{CapeArrayByteIn,CapeArrayByteOut};
mod cape_array_boolean;
pub use cape_array_boolean::{CapeArrayBooleanIn,CapeArrayBooleanOut};
mod cape_array_enumeration;
pub use cape_array_enumeration::{CapeArrayEnumerationIn,CapeArrayEnumerationOut};
mod cape_array_string;
pub use cape_array_string::{CapeArrayStringIn,CapeArrayStringOut};
mod cape_array_value;
pub use cape_array_value::{CapeArrayValueIn,CapeArrayValueOut};
mod cape_object;
pub use cape_object::CapeObject;
mod cape_error;
pub use cape_error::CapeError;
mod cape_error_impl;
pub use cape_error_impl::CapeErrorImpl;
mod cape_result_value;
mod cobia_enums;
pub use cobia_enums::*;
mod cape_registry_key;
pub use cape_registry_key::*;
mod cape_registry_key_writer;
pub use cape_registry_key_writer::CapeRegistryKeyWriter;
mod cape_registrar;
pub use cape_registrar::CapeRegistrar;
mod cape_registry_writer;
pub use cape_registry_writer::CapeRegistryWriter;
mod cape_pmc_registration_details;
pub use cape_pmc_registration_details::CapePMCRegistrationDetails;
mod cape_pmc_enumerator;
pub use cape_pmc_enumerator::CapePMCEnumerator;
mod cape_type_library_details;
pub use cape_type_library_details::CapeLibraryDetails;
mod cape_type_library_enumerator;
pub use cape_type_library_enumerator::CapeTypeLibraries;
mod cobia_pmc_helpers;
pub use cobia_pmc_helpers::*;
mod cape_object_impl;
pub use cape_object_impl::*;
mod cape_smart_pointer;
#[cfg_attr(not(target_os="windows"), path = "cape_window_id_posix.rs")]
#[cfg_attr(target_os="windows", path = "cape_window_id_win32.rs")]
mod cape_window_id;
pub use cape_window_id::{CapeWindowId,CapeWindowIdToRaw,CapeWindowIdFromRaw};

pub mod cape_open; //types common to all CAPE-OPEN versions
pub mod cape_open_1_2; //types specific to CAPE-OPEN 1.2

use core::hash::Hash;
use std::error;
use std::fmt;
use std::hash::Hasher;
use std::path::PathBuf;

//macros
pub use cobia_macro::*;

pub use cape_result_value::*;

// //! # Rust COBIA binding
// //!
// //! 'cobia' is a Rust binding for the COBIA library.
// //!

/// #COBIA initialization routine
///
/// Must be called prior to calling any COBIA routine.
///
/// # Examples
///
/// ```
/// use cobia;
/// cobia::cape_open_initialize().unwrap();
/// ```

#[must_use]
pub fn cape_open_initialize() -> Result<(), COBIAError> {
	let mut s = CapeStringImpl::new();
	unsafe {
		if !C::capeInitialize((&s.as_cape_string_out() as *const C::ICapeString).cast_mut()) {
			Err(COBIAError::Message(s.as_string()))
		} else {
			Ok(())
		}
	}
}

/// #COBIA clean-up routine
///
/// Deallocates COBIA resources. Should be called when COBIA is no longer needed.
/// No COBIA routines should be called after this routine is called.
///
/// # Examples
///
/// ```
/// use cobia;
/// cobia::cape_open_initialize().unwrap();
/// //use COBIA and do some stuff
/// cobia::cape_open_cleanup()
/// ```

pub fn cape_open_cleanup() {
	unsafe { C::capeCleanup() }
}

/// #get COBIA version
///
/// Returns the COBIA version
///
/// # Examples
///
/// ```
/// use cobia;
/// cobia::cape_open_initialize().unwrap();
/// println!("Cobia version: {}",cobia::get_cobia_version());
/// ```

pub fn get_cobia_version() -> String {
	let mut s = CapeStringImpl::new();
	unsafe {
		C::capeGetCobiaVersion((&s.as_cape_string_out() as *const C::ICapeString).cast_mut());
	}
	s.as_string()
}

/// #get COBIA language
///
/// Returns the COBIA language
///
/// # Examples
///
/// ```
/// use cobia;
/// cobia::cape_open_initialize().unwrap();
/// println!("Cobia language: {}",cobia::get_cobia_language());
/// ```

pub fn get_cobia_language() -> String {
	let mut s = CapeStringImpl::new();
	unsafe {
		C::capeGetCobiaLanguage((&s.as_cape_string_out() as *const C::ICapeString).cast_mut());
	}
	s.as_string()
}

/// Wrapper class around native CapeUUID
pub use C::CapeUUID;

impl CapeUUID {
	/// #Create a new CapeUUID
	///
	/// Creates a new CapeUUID, and generates unique content
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// cobia::cape_open_initialize().unwrap();
	/// let uuid=cobia::CapeUUID::new();
	/// ```
	pub fn new() -> Self {
		unsafe {
			C::capeGenerateUUID()
		}
	}

	/// #Create a null CapeUUID
	///
	/// Creates a null CapeUUID
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// cobia::cape_open_initialize().unwrap();
	/// let uuid=cobia::CapeUUID::null();
	/// let uuid_1=cobia::CapeUUID::from_string("{00000000-0000-0000-0000-000000000000}").unwrap();
	/// assert_eq!(uuid_1,uuid);
	/// ```
	pub const fn null() -> Self {
		CapeUUID { data: [0; 16] }
	}

	/// #Create a CapeUUID from character slice
	///
	/// Creates a new CapeUUID from a character slice
	///
	/// # Arguments
	///
	/// * `slice` - A character slice to be converted to a CapeUUID
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// cobia::cape_open_initialize().unwrap();
	/// let uuid=cobia::CapeUUID::from_slice(&[0x12u8,0x34,0x56,0x78,0x9a,0xbc,0xde,0xf0,0x12,0x34,0x56,0x78,0x90,0xab,0xcd,0xef]);
	/// let uuid_1=cobia::CapeUUID::from_string("{12345678-9abc-def0-1234-567890abcdef}").unwrap();
	/// assert_eq!(uuid_1,uuid);
	/// ```
	pub const fn from_slice(slice: &[u8; 16]) -> Self {
		Self {data: *slice}
	}

	/// #Create a new CapeUUID from a string
	///
	/// Creates a new CapeUUID from a string
	///
	/// # Arguments
	///
	/// * `s` - A string slice to be converted to a CapeUUID
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// cobia::cape_open_initialize().unwrap();
	/// let uuid_1=cobia::CapeUUID::from_string("{12345678-9abc-def0-1234-567890abcdef}").unwrap();
	/// let uuid_2=cobia::CapeUUID::from_string("{12345678-9ABC-DEF0-1234-567890ABCDEF}").unwrap();
	/// assert_eq!(uuid_1,uuid_2);
	/// ```
	pub fn from_string(s: &str) -> Result<Self, COBIAError> {
		let mut uuid = CapeUUID::null();
		let str_uuid = CapeStringImpl::from_string(s);
		let res = unsafe { C::capeUUIDFromString(str_uuid.as_capechar_const(), &mut uuid) };
		if res == COBIAERR_NOERROR {
			Ok(uuid)
		} else {
			Err(COBIAError::Code(res))
		}
	}

	/// #Create a string from a CapeUUID
	///
	/// Creates a string from a CapeUUID
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// cobia::cape_open_initialize().unwrap();
	/// let uuid=cobia::CapeUUID::from_string("{12345678-9ABC-DEF0-1234-567890ABCDEF}").unwrap();
	/// let s=uuid.as_string();
	/// assert_eq!(&s,"{12345678-9abc-def0-1234-567890abcdef}");
	/// ```
	pub fn as_string(&self) -> String {
		let mut s = CapeStringImpl::new();
		unsafe {
			C::capeStringFromUUID(self as *const CapeUUID, (&s.as_cape_string_out() as *const C::ICapeString).cast_mut());
		}
		s.as_string()
	}

	/// #Compare two CapeUUIDs
	///
	/// Compares two CapeUUIDs
	///
	/// # Arguments
	///
	/// * `other` - The other CapeUUID to compare to
	///
	/// # Returns
	///
	/// * -1 if self is less than other
	/// * 0 if self is equal to other
	/// * 1 if self is greater than other
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// cobia::cape_open_initialize().unwrap();
	/// let uuid_1=cobia::CapeUUID::from_string("{12345678-9abc-def0-1234-567890abcdee}").unwrap();
	/// let uuid_2=cobia::CapeUUID::from_string("{12345678-9abc-def0-1234-567890abcdef}").unwrap();
	/// assert_eq!(uuid_1.compare(&uuid_2),-1);
	/// assert_eq!(uuid_2.compare(&uuid_1),1);
	/// assert_eq!(uuid_2.compare(&uuid_2),0);
	/// ```
	pub fn compare(&self, other: &Self) -> i32 {
		unsafe {
			C::capeUUID_Compare(
				self as *const CapeUUID,
				other as *const CapeUUID,
			)
		}
	}
}

impl From<&[u8; 16]> for CapeUUID {
	fn from(slice: &[u8; 16]) -> Self {
		CapeUUID::from_slice(slice)
	}
}

impl From<&str> for CapeUUID {
	fn from(s: &str) -> Self {
		CapeUUID::from_string(s).unwrap()
	}
}

impl PartialEq for CapeUUID {
	/// #Compare two CapeUUIDs
	///
	/// Compares two CapeUUIDs
	///
	/// # Arguments
	///
	/// * `other` - The other CapeUUID to compare to
	///
	/// # Returns
	///
	/// * true if self is equal to other
	/// * false if self is not equal to other
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// cobia::cape_open_initialize().unwrap();
	/// let uuid_1=cobia::CapeUUID::from_string("{12345678-9abc-def0-1234-567890abcdef}").unwrap();
	/// let uuid_2=cobia::CapeUUID::from_string("{12345678-9abc-def0-1234-567890abcdef}").unwrap();
	/// assert_eq!(uuid_1,uuid_2);
	/// ```
	///
	/// ```
	/// use cobia;
	/// cobia::cape_open_initialize().unwrap();
	/// let uuid_1=cobia::CapeUUID::from_string("{12345678-9abc-def0-1234-567890abcdef}").unwrap();
	/// let uuid_2=cobia::CapeUUID::from_string("{87654321-9abc-def0-1234-567890abcdef}").unwrap();
	/// assert!(uuid_1!=uuid_2);
	/// ```

	fn eq(&self, other: &Self) -> bool {
		unsafe {
			C::capeUUID_Equal(
				self as *const CapeUUID,
				other as *const CapeUUID,
			)
		}
	}
}

impl Eq for CapeUUID {}

impl Hash for CapeUUID {
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write(&self.data);
	}
}

impl std::fmt::Display for CapeUUID {
	/// Formats the CapeUUID using the given formatter.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// cobia::cape_open_initialize().unwrap();
	/// let uuid=cobia::CapeUUID::from_string("{12345678-9abc-def0-1234-567890abcdef}").unwrap();
	/// assert_eq!(format!("{}",uuid),"{12345678-9abc-def0-1234-567890abcdef}");
	/// ```

	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_string())
	}
}

/// #get COBIA installation folder
///
/// Returns the COBIA installation folder of the currently loaded COBIA
///
/// # Examples
///
/// ```
/// use cobia;
/// cobia::cape_open_initialize().unwrap();
/// println!("Cobia installation folder: {}",cobia::get_cobia_folder().to_str().unwrap());
/// ```
pub fn get_cobia_folder() -> PathBuf {
	let mut s = CapeStringImpl::new();
	unsafe {
		C::capeGetCOBIAFolder((&s.as_cape_string_out() as *const C::ICapeString).cast_mut());
	}
	PathBuf::from(s.as_string())
}

/// #get COBIA user data folder
///
/// Returns the folder where COBIA per-user data is located (e.g. registry)
///
/// # Examples
///
/// ```
/// use cobia;
/// cobia::cape_open_initialize().unwrap();
/// println!("Cobia user data folder: {}",cobia::get_cobia_user_data_folder().to_str().unwrap());
/// ```
pub fn get_cobia_user_data_folder() -> PathBuf {
	let mut s = CapeStringImpl::new();
	unsafe {
		C::capeGetCOBIAUserDataFolder((&s.as_cape_string_out() as *const C::ICapeString).cast_mut());
	}
	PathBuf::from(s.as_string())
}

/// #get COBIA system data folder
///
/// Returns the folder where COBIA all-users data is located (e.g. registry)
///
/// For a per-user installation, this is the same folder as the user data folder.
///
/// # Examples
///
/// ```
/// use cobia;
/// cobia::cape_open_initialize().unwrap();
/// println!("Cobia system data folder: {}",cobia::get_cobia_system_data_folder());
/// ```
pub fn get_cobia_system_data_folder() -> String {
	let mut s = CapeStringImpl::new();
	unsafe {
		C::capeGetCOBIAUserDataFolder((&s.as_cape_string_out() as *const C::ICapeString).cast_mut());
	}
	s.as_string()
}

/// Service function to get in-process service type for current bitness
#[cfg(target_pointer_width = "64")]
pub fn inproc_service_type() -> CapePMCServiceType {
	CapePMCServiceType::Inproc64
}

/// Service function to get in-process service type for current bitness
#[cfg(target_pointer_width = "32")]
pub fn inproc_service_type() -> CapePMCServiceType {
	CapePMCServiceType::Inproc32
}


#[cfg(test)]
mod tests {
	use crate::*;
	use regex::Regex;

	#[test]
	fn get_version() {
		cape_open_initialize().unwrap();
		let version = get_cobia_version();
		let re = Regex::new(r"\d+\.\d+\.\d+\.\d+").unwrap();
		assert!(re.is_match(&version));
	}

	#[test]
	fn cobia_folder_exists() {
		cape_open_initialize().unwrap();
		let folder = get_cobia_folder();
		assert!(folder.exists());
	}
}
