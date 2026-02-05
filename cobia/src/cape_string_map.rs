use crate::{cape_string_const::CapeStringHashKey,C,CapeStringImpl,CapeStringConstProvider};

/// Class to store a platform dependent string encoding for use in case
/// insensitive hash maps or for use of case insensitive comparisons.
///
/// For COBIA, strings go over the pipeline
///  as null terminated. For Windows, COBIA requires UTF-16 encoding.
///
/// The [`CapeStringHashKey`] implementation uses a `Vec<u16>` to store,
/// owned string data, or allows reference to data provided by any 
/// class that implements CapeStringConstProvider, so that a copy of 
/// the data is not needed for hash lookups.
///
/// A common use case in CAPE-OPEN is to make hash maps of
/// strings for case-insentive lookups: CapeOpenMap` uses the 
/// more performant hasher in the FxHasmap class.
///
/// As the constructors are not const, a construct like for example
/// LazyLock can be used to make static instances of this class, 
/// or any class that contains this class.
/// 
/// # Examples
///
/// ```
/// use cobia::*;
/// use cobia::prelude::*;
/// let mut map=cobia::CapeOpenMap::new();
/// map.insert("idealGasEnthalpy",1);
/// map.insert("idealGasEntropy",2);
///
/// assert_eq!(map.get(&cobia::CapeStringConstNoCase::from_string("IDEALGASENTHALPY")),Some(&1));
///
/// assert_eq!(map.get(&cobia::CapeStringImpl::from_string("IDEALGASENTROPY")),Some(&2));
///
/// let s2=cobia::CapeStringImpl::from_string("IDEALGASENTHALPY");
/// fn test_string_in(map:&cobia::CapeOpenMap<i32>,s: &cobia::CapeStringIn) {
///     assert_eq!(map.get(s),Some(&1));
/// }
/// test_string_in(&map,&CapeStringInFromProvider::from(&s2).as_cape_string_in());
///
/// assert_eq!(map.get(&cobia::CapeStringImpl::from_string("CriticalTemperature")),None);
/// ```

pub struct CapeOpenMap<V> (std::collections::HashMap<CapeStringHashKey,V,fxhash::FxBuildHasher>);

impl<V> CapeOpenMap<V> {

	pub fn new() -> Self {
		Self(std::collections::HashMap::with_hasher(fxhash::FxBuildHasher::default()))
	}

	pub fn with_capacity(capacity: usize) -> Self {
		Self(std::collections::HashMap::with_capacity_and_hasher(capacity,fxhash::FxBuildHasher::default()))
	}

	pub fn capacity(&self) -> usize {
		self.0.capacity()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn clear(&mut self) {
		self.0.clear()
	}

	pub fn reserve(&mut self, additional: usize) {
		self.0.reserve(additional)
	}

	pub fn try_reserve(&mut self, additional: usize) -> Result<(), std::collections::TryReserveError> {
		self.0.try_reserve(additional)
	}

	pub fn shrink_to_fit(&mut self) {
		self.0.shrink_to_fit()
	}

	pub fn shrink_to(&mut self, min_capacity: usize) {
		self.0.shrink_to(min_capacity)
	}

	pub fn get<'a,'b,Q:CapeStringConstProvider>(&'a self, k: &'b Q) -> Option<&'a V> {
		let k = CapeStringHashKey::borrowed_from_string_constant(k);
		self.0.get(&k)
	}

	pub fn get_mut<'a,'b,Q:CapeStringConstProvider>(&'a mut self, k: &'b Q) -> Option<&'a mut V> {
		let k = CapeStringHashKey::borrowed_from_string_constant(k);
		self.0.get_mut(&k)
	}

	pub fn contains_key<Q:CapeStringConstProvider>(&self, k: &Q) -> bool {
		let k = CapeStringHashKey::borrowed_from_string_constant(k);
		self.0.contains_key(&k)
	}

	pub fn insert_from_cape_string<T:CapeStringConstProvider>(&mut self, k: &T, v: V) -> Option<V> {
		let (ptr,size)=k.as_capechar_const_with_length();
		self.0.insert(CapeStringHashKey::from_cape_char_const(ptr,size),v)
	}

	pub fn insert(&mut self, k: &str, v: V) -> Option<V> {
		self.0.insert(CapeStringHashKey::from_string(k),v)
	}

	pub fn remove<Q:CapeStringConstProvider>(&mut self, k: &Q) -> Option<V> {
		let k = CapeStringHashKey::borrowed_from_string_constant(k);
		self.0.remove(&k)
	}

}

impl<V:Clone> Clone for CapeOpenMap<V> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<V:std::fmt::Debug> std::fmt::Debug for CapeOpenMap<V> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl <V> std::default::Default for CapeOpenMap<V> {
	fn default() -> Self {
		Self::new()
	}
}

impl <V:std::cmp::PartialEq> std::cmp::PartialEq for CapeOpenMap<V> {
	fn eq(&self, other: &Self) -> bool {
		self.0.eq(&other.0)
	}
}

impl <V:std::cmp::Eq> std::cmp::Eq for CapeOpenMap<V> {}

