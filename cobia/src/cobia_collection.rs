use crate::C;
use crate::*;
use std::marker::PhantomData;
use cape_smart_pointer::CapeSmartPointer;

const ICOBIACOLLECTION_UUID:CapeUUID=CapeUUID::from_slice(&[0xdeu8,0xafu8,0xeau8,0x79u8,0xd3u8,0x49u8,0x4du8,0x03u8,0xbbu8,0x05u8,0x37u8,0x7cu8,0x0du8,0x3eu8,0x47u8,0x8bu8]);

/// CobiaCollectionBase wraps an collection interface
/// 
/// The collection interface is returned in case muliple objects
/// are returned by a method. 
/// 
/// Collection items each must have a unique name, and the name
/// is exposed from the approproate identification interface.
/// 
/// Collection items are accessed by index or by name.
///
/// #Example
///
/// ```
/// use cobia;
/// use cobia::prelude::*;
/// cobia::cape_open_initialize().unwrap();
/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
/// let libraries = library_enumerator.libraries().unwrap(); //this is a CobiaCollection smart pointer
/// assert!(libraries.size() > 0); //normally the CAPE-OPEN type libraries are registered
/// cobia::cape_open_cleanup();
/// ```

#[cape_smart_pointer(ICOBIACOLLECTION_UUID)]
pub struct CobiaCollectionBase<Element:CapeSmartPointer> {
	interface: *mut C::ICobiaCollection, //the interface is cast to ICapeCollection - function signatures are identical
	element_type : PhantomData<Element>
}

impl<Element:CapeSmartPointer> CobiaCollectionBase<Element> {

	/// Create a new CobiaCollectionBase from an interface pointer
	///
	/// This member is not typically called. Instead, the CobiaCollectionBase is created by the API functions that return it.
	///
	/// # Safety
	///
	/// The interface pointer must be valid and must point to an object that implements the ICobiaCollection interface.
	///
	/// # Panics
	///
	/// This function panics if the interface pointer is null.

	pub(crate) fn from_interface_pointer(interface: *mut C::ICobiaCollection) ->  Self {
		if interface.is_null() {
			panic!("Null pointer in creation of CobiaCollectionBase<Element>");
		}
		unsafe {((*(*interface).vTbl).base.addReference.unwrap())((*interface).me)};
		Self {
			interface,
			element_type: PhantomData,
		}
	}

	/// Create a new CobiaCollectionBase from an interface pointer without adding a reference
	///
	/// This member is not typically called. Instead, the CobiaCollectionBase is created by the API functions that return it.
	///
	/// # Safety
	///
	/// The interface pointer must be valid and must point to an object that implements the ICobiaCollection interface.
	///
	/// # Panics
	///
	/// This function panics if the interface pointer is null.

	pub(crate) fn attach(interface: *mut C::ICobiaCollection) ->  Self {
		if interface.is_null() {
			panic!("Null pointer in creation of CobiaCollectionBase<Element>");
		}
		Self {
			interface,
			element_type: PhantomData,
		}
	}

	/// Get the number of elements in the collection
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let libraries = library_enumerator.libraries().unwrap(); //this is a CobiaCollection smart pointer
	/// assert!(libraries.size() > 0); //normally the CAPE-OPEN type libraries are registered
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn size(&self) -> usize {
		let mut size: C::CapeInteger = 0;
		let res=unsafe { (*(*self.interface).vTbl).getCount.unwrap()((*self.interface).me, &mut size as *mut C::CapeInteger) };
		if res == COBIAERR_NOERROR { 
			size as usize
		} else {
			//getCount should not fail... if it does, assume that 0 is a reasonable default
			debug_assert!(false);
			0
		}
	}

	/// Get a collection item by index in the collection
	///
	/// The index is zero-based, and must be 0 <= index < size().
	///
	/// Note that the collection implements iterators, but not the Index
	/// or IndexMut trait, as both these traits require returning the 
	/// item by reference, which is not possible in this context, as 
	/// the returned object owns the interface pointer.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let libraries = library_enumerator.libraries().unwrap(); //this is a CobiaCollection smart pointer
	/// assert!(libraries.size() > 0); //normally the CAPE-OPEN type libraries are registered
	/// let library = libraries.at(0).unwrap();
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn at(&self,index : usize) -> Result<Element, COBIAError> {
		let mut el : *mut C::ICapeInterface=std::ptr::null_mut();
		let index = index as C::CapeInteger;
		let res=unsafe { (*(*self.interface).vTbl).ItemByIndex.unwrap()((*self.interface).me,index,&mut el as *mut *mut C::ICapeInterface) };
		if res == COBIAERR_NOERROR {
			let el=el as *mut C::ICapeInterface as *mut Element::Interface;
			if el.is_null() {
				Err(COBIAError::Code(COBIAERR_NULLPOINTER))
			} else {
				Ok(Element::attach(unsafe{&mut*el as &mut Element::Interface}))
			}
		} else {
			Err(COBIAError::from_object(res,self))
		}
	}

	/// Get a collection item by name
	///
	/// The name is case insentive, but must correspond to one of the items
	/// in the collection.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let libraries = library_enumerator.libraries().unwrap(); //this is a CobiaCollection smart pointer
	/// assert!(libraries.size() > 0); //normally the CAPE-OPEN type libraries are registered
	/// let library = libraries.at(0).unwrap();
	/// let lib_name = library.get_name().unwrap();
	/// let library1 = libraries.get(&lib_name).unwrap();
	/// assert_eq!(library.get_uuid().unwrap(),library1.get_uuid().unwrap());
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn get(&self,id : &str) -> Result<Element, COBIAError> {
		let mut el : *mut C::ICapeInterface=std::ptr::null_mut();
		let id=CapeStringImpl::from_string(id);
		let res=unsafe { (*(*self.interface).vTbl).ItemByName.unwrap()((*self.interface).me,(&id.as_cape_string_in() as *const C::ICapeString).cast_mut(),&mut el as *mut *mut C::ICapeInterface) };
		if res == COBIAERR_NOERROR {
			let el=el as *mut C::ICapeInterface as *mut Element::Interface;
			if el.is_null() {
				Err(COBIAError::Code(COBIAERR_NULLPOINTER))
			} else {
				Ok(Element::attach(unsafe{&mut*el as &mut Element::Interface}))
			}
		} else {
			Err(COBIAError::from_object(res,self))
		}
	}

	///Get an iterator for the collection by reference
	///
	///This iterator does not consume the collection.
	///
	/// # Examples
	///
	/// ```
	/// use cobia;
	/// use cobia::prelude::*;
	/// cobia::cape_open_initialize().unwrap();
	/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
	/// let libraries = library_enumerator.libraries().unwrap(); //this is a CobiaCollection smart pointer
	/// let mut found=false;
	/// for library in libraries.iter() {
	///     let lib_name = library.get_name().unwrap();
	///     if lib_name=="CAPEOPEN_1_2" { //performance note: this is not efficient, but it is an example
	///         found=true;
	///			break;
	///     }
	/// }
	/// assert!(found);
	/// cobia::cape_open_cleanup();
	/// ```

	pub fn iter(&self) -> CobiaCollectionBaseRefIterator<'_,Element> {
		CobiaCollectionBaseRefIterator {
			collection:self,
			index:0
		}
	}

}

/// Iterator that consumes a CobiaCollectionBase
///
/// This iterator consumes the collection and returns the elements.
///
/// # Examples
///
/// ```
/// use cobia;
/// use cobia::prelude::*;
/// cobia::cape_open_initialize().unwrap();
/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
/// let libraries = library_enumerator.libraries().unwrap(); //this is a CobiaCollection smart pointer
/// let mut found=false;
/// for library in libraries {
///     let lib_name = library.get_name().unwrap();
///     if lib_name=="CAPEOPEN_1_2" { //performance note: this is not efficient, but it is an example
///         found=true;
///			break;
///     }
/// }
/// assert!(found);
/// cobia::cape_open_cleanup();
/// ```

pub struct CobiaCollectionBaseIterator<Element:CapeSmartPointer> {
	collection:CobiaCollectionBase<Element>,
	index:usize
}

impl<Element:CapeSmartPointer> Iterator for CobiaCollectionBaseIterator<Element> {
	type Item=Element;
	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.collection.size() {
			let res=self.collection.at(self.index);
			self.index+=1;
			match res {
				Ok(el) => Some(el),
				Err(_) => None
			}
		} else {
			None
		}
	}
}

impl<Element:CapeSmartPointer> IntoIterator for CobiaCollectionBase<Element> {
	type Item=Element;
	type IntoIter=CobiaCollectionBaseIterator<Element>;
	fn into_iter(self) -> Self::IntoIter {
		CobiaCollectionBaseIterator {
			collection:self,
			index:0
		}
	}
}

/// Iterator that using a reference to CobiaCollectionBase
///
/// This iterator returns the elements and does not consume the collection.
///
/// # Examples
///
/// ```
/// use cobia;
/// use cobia::prelude::*;
/// cobia::cape_open_initialize().unwrap();
/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
/// let libraries = library_enumerator.libraries().unwrap(); //this is a CobiaCollection smart pointer
/// let mut found=false;
/// for library in &libraries {
///     let lib_name = library.get_name().unwrap();
///     if lib_name=="CAPEOPEN_1_2" { //performance note: this is not efficient, but it is an example
///         found=true;
///			break;
///     }
/// }
/// assert!(found);
/// cobia::cape_open_cleanup();
/// ```

pub struct CobiaCollectionBaseRefIterator<'a,Element:CapeSmartPointer> {
	collection:&'a CobiaCollectionBase<Element>,
	index:usize
}

impl<'a,Element:CapeSmartPointer> Iterator for CobiaCollectionBaseRefIterator<'a,Element> {
	type Item=Element;
	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.collection.size() {
			let res=self.collection.at(self.index);
			self.index+=1;
			match res {
				Ok(el) => Some(el),
				Err(_) => None
			}
		} else {
			None
		}
	}
}

impl<'a,Element:CapeSmartPointer> IntoIterator for &'a CobiaCollectionBase<Element> {
	type Item=Element;
	type IntoIter=CobiaCollectionBaseRefIterator<'a,Element>;
	fn into_iter(self) -> Self::IntoIter {
		CobiaCollectionBaseRefIterator {
			collection:&self,
			index:0
		}
	}
}

/// CobiaCollection wraps an ICobiaCollection interface
///
/// This interface is returned by API functions that return multiple objects.
///
/// #Example
///
/// ```
/// use cobia;
/// use cobia::prelude::*;
/// cobia::cape_open_initialize().unwrap();
/// let library_enumerator = cobia::CapeTypeLibraries::new().unwrap();
/// let libraries = library_enumerator.libraries().unwrap(); //this is a CobiaCollection smart pointer
/// assert!(libraries.size() > 0); //normally the CAPE-OPEN type libraries are registered
/// cobia::cape_open_cleanup();
/// ```

pub type CobiaCollection<Element> = CobiaCollectionBase<Element>;

