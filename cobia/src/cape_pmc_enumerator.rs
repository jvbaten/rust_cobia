use crate::C;
use crate::*;

/// Enumerator for all registered PMCs.
///
/// This is used to get the details of each PMC by name or UUID,
/// or to get all registered PMCs.

pub struct CapePMCEnumerator {
	pub(crate) interface: *mut C::ICapePMCEnumerator,
}

impl CapePMCEnumerator {

	/// Create a new PMC enumerator.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// //...
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn new() -> Result<CapePMCEnumerator, COBIAError> {
		let mut interface: *mut C::ICapePMCEnumerator = std::ptr::null_mut();
		let result =
			unsafe { C::capeGetPMCEnumerator(&mut interface as *mut *mut C::ICapePMCEnumerator) };
		if result == COBIAERR_NOERROR {
			Ok(CapePMCEnumerator { interface })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get a PMC by its UUID.
	///
	/// Typically a PME will enumerate PMCs to allow the user to pick one. 
	/// Then the PME stores the UUID of the PMC that the user picked, so 
	/// that in future sessions, it can be re-created.
	///
	/// Upon saving a flowsheet, the UUID of the PMC is stored in the 
	/// flowsheet, along with the UUID of the PME, and perhaps additional
	/// information that is needed to re-create the PMC (such as the package
	/// name in case of a Property Package created from a CAPE-OPEN 1.2 
	/// Property Package Manager).
	///
	/// Then, when restoring the flowsheet, the PMC is created from its
	/// UUID, and subsequently depersisted.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// let mut first_pmc : Option<cobia::CapePMCRegistrationDetails> = None;
	/// for pmc in pmc_enumerator.all_pmcs().unwrap() {
	///   first_pmc = Some(pmc);
	///   break;
	/// }
	/// if let Some(pmc) = first_pmc {
	///     let uuid = pmc.get_uuid().unwrap();
	///     let pmc = pmc_enumerator.get_pmc_by_uuid(&uuid).unwrap();
	///     assert_eq!(pmc.get_uuid().unwrap(),uuid);
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get_pmc_by_uuid(&self, uuid: &CapeUUID) -> Result<CapePMCRegistrationDetails, COBIAError> {
		let mut interface: *mut C::ICapePMCRegistrationDetails = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).getPMCbyUUID.unwrap())(
				(*self.interface).me,
				(uuid as *const C::CapeUUID).cast_mut(),
				&mut interface as *mut *mut C::ICapePMCRegistrationDetails,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapePMCRegistrationDetails { interface })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	pub fn get_pmc_by_prog_id(
		&self,
		prog_id: &str,
	) -> Result<CapePMCRegistrationDetails, COBIAError> {
		let mut interface: *mut C::ICapePMCRegistrationDetails = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).getPMCbyProgId.unwrap())(
				(*self.interface).me,
				CapeStringImpl::from_string(prog_id).as_capechar_const(),
				&mut interface as *mut *mut C::ICapePMCRegistrationDetails,
			)
		};
		if result == COBIAERR_NOERROR {
			Ok(CapePMCRegistrationDetails { interface })
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get all registered PMCs of specific type(s).
	///
	/// Get a collection of all registered PMCs, of a given type or 
	/// multiple given types.
	///
	/// In a typical scenario the PME is interested in instantiating, and therefore
	/// selecting,a specific type of PMC, such as a Property Package, Unit Operation, ...
	/// This function provides a collection with all PMCs of the given type(s).
	///
	/// On Windows, COM based PMCs are included in the collection.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// use cobia::cape_open;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// //let's create some thermo! Enumerate all Property Package Managers and stand-alone Property Packages
	/// let pmc_types=[cape_open::CATEGORYID_PROPERTYPACKAGEMANAGER,cape_open::CATEGORYID_STANDALONEPROPERTYPACKAGE];
	/// let pmcs = pmc_enumerator.pmcs(&pmc_types).unwrap();
	/// for pmc in pmcs {
	///     println!("Found Thermo-PMC: {} ({})",pmc.get_name().unwrap(),pmc.get_description().unwrap());
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn pmcs(&self,cat_ids: &[CapeUUID]) -> Result<CobiaCollection<CapePMCRegistrationDetails>,COBIAError> {
		let mut p: *mut C::ICobiaCollection = std::ptr::null_mut();
		let result = unsafe {
			//let cat_ids_repr: std::raw::Slice<CapeUUID> = cat_ids.repr(); TODO std::raw::Slice is unstable, use when available
			let mut v: std::vec::Vec<CapeUUID> = Vec::new(); //work-around: store in a local vector
			v.extend_from_slice(cat_ids);
			((*(*self.interface).vTbl).getPMCsByCategory.unwrap())(
				(*self.interface).me,
				v.as_ptr(),
				v.len() as C::CapeSize,
				&mut p as *mut *mut C::ICobiaCollection
			)
		};
		if result == COBIAERR_NOERROR {
			if p.is_null() {
				Err(COBIAError::Code(COBIAERR_NULLPOINTER))
			} else {
				Ok(CobiaCollection::attach(p))
			}
		} else {
			Err(COBIAError::Code(result))
		}
	}

	/// Get all registered PMCs of any type.
	///
	/// Get a collection of all registered PMCs, of any PMC type.
	///
	/// This function is not typically used, as in a typical scenario
	/// the PME is interested in instantiating, and therefore selecting,
	/// a specific type of PMC, such as a Property Package, Unit Operation, ...
	///
	/// On Windows, COM based PMCs are included in the collection.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
	/// let pmcs = pmc_enumerator.all_pmcs().unwrap();
	/// for pmc in pmcs {
	///     println!("Found PMC: {} ({})",pmc.get_name().unwrap(),pmc.get_description().unwrap());
	/// }
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn all_pmcs(&self) -> Result<CobiaCollection<CapePMCRegistrationDetails>,COBIAError> {
		let mut p: *mut C::ICobiaCollection = std::ptr::null_mut();
		let result = unsafe {
			((*(*self.interface).vTbl).getAllPMCs.unwrap())(
				(*self.interface).me,
				&mut p as *mut *mut C::ICobiaCollection
			)
		};
		if result == COBIAERR_NOERROR {
			if p.is_null() {
				Err(COBIAError::Code(COBIAERR_NULLPOINTER))
			} else {
				Ok(CobiaCollection::attach(p))
			}
		} else {
			Err(COBIAError::Code(result))
		}
	}

}

/// Release pointer
///
/// ICapePMCEnumerator derives from ICobiaBase, which contains
/// addReference() and release(). The Drop trait calls release.

impl Drop for CapePMCEnumerator {
	fn drop(&mut self) {
		unsafe {
			((*(*self.interface).vTbl).base.release.unwrap())((*self.interface).me);
		}
	}
}

/// Add pointer reference
///
/// ICapePMCEnumerator derives from ICobiaBase, which contains
/// addReference() and release(). The Clone trait calls addReference.

impl Clone for CapePMCEnumerator {
	fn clone(&self) -> Self {
		unsafe {
			((*(*self.interface).vTbl).base.addReference.unwrap())((*self.interface).me);
		}
		CapePMCEnumerator {
			interface: self.interface,
		}
	}
}
