use crate::*;
use cape_smart_pointer::CapeSmartPointer;

const ICAPEINTERFACE_UUID:CapeUUID=CapeUUID::from_slice(&[0x53u8,0xa7u8,0x4eu8,0xe9u8,0xadu8,0xfau8,0x49u8,0x16u8,0xbeu8,0x95u8,0x04u8,0xe9u8,0x28u8,0x3cu8,0xc2u8,0x2eu8]);

/// Generic Cape Object smart pointer
///
/// This is a generic smart pointer to any CAPE-OPEN object, 
/// that points to the basic ICapeInterface interface.
///
/// This smart pointer is used when it is not clear what the
/// type of the object is, but it is known that it is a CAPE-OPEN.
/// 
/// #Example
///
/// ```no_run
/// use cobia;
/// use cobia::prelude::*;
/// cobia::cape_open_initialize().unwrap();
/// let pmc_enumerator = cobia::CapePMCEnumerator::new().unwrap();
/// let ppm_info_result=pmc_enumerator.get_pmc_by_prog_id("COCO_TEA.PropertyPackManager");
/// match ppm_info_result {
/// 	Ok(ppm_info) => {
/// 		let ppm_result=ppm_info.create_instance( //ppm_result is a CapeObject
///				cobia::CapePMCCreationFlags::AllowRestrictedThreading); //we only use the object in this thread
/// 		match ppm_result {
/// 			Ok(ppm) => {
/// 				//show name
/// 				let iden_result=cobia::cape_open_1_2::CapeIdentification::from_object(&ppm); //this is how to obtain a particular interface from a CapeObject
/// 				match iden_result {
/// 					Ok(iden) => {
///							let mut name = cobia::CapeStringImpl::new();
/// 						iden.get_component_name(&mut name).unwrap();
/// 						println!("PPM name: {}",name);
/// 					}
/// 					Err(_) => {
/// 						eprintln!("Object does not implement CAPEOPEN_1_2::ICapeIdentification");
/// 					}
/// 				}
/// 			}
/// 			Err(err) => {
/// 				eprintln!("Cannot find TEA property package manager (not installed?): {err}");
/// 			}
/// 		};
/// 	}
/// 	Err(err) => {
/// 		eprintln!("Cannot find TEA property package manager (not installed?): {err}");
/// 	}
/// };
/// cobia::cape_open_cleanup();
/// ```

#[cape_smart_pointer(ICAPEINTERFACE_UUID)]
pub struct CapeObject {
	pub(crate) interface: *mut C::ICapeInterface,
}

