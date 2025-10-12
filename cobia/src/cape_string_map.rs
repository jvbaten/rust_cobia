use crate::{CapeStringHashKey,C,CapeStringImpl,CapeStringConstProvider};

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
/// This construct however requires that the hash keys are given a
/// lifetime; the hash keys that are stored in the map are owned by the map
/// and are given a dummy life time of 'static. Hash keys that are borrowed
/// from a CapeStringConstProvider are given the lifetime of the provider.
/// This implies that the life time checker for any mutable references
/// to the hash map assumes that keys past to such references are of 
/// life time 'static. Hence, mutable members of the hash map cannot
/// take a borrowed key, but must use an owned key; in particular we cannot
/// remove a key from the map using a borrowed key, nor can we return
/// mutable references to the values in the map based on borrowed keys.
/// Some traits, like Index, do not allow for such life time constraints
/// and are not implemented. Use fn get() instead. Other than that, 
/// the CapeOpenMap generally provides the same interface as the
/// standard HashMap, on which it is based.
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
/// map.insert(cobia::CapeStringHashKey::from_string("idealGasEnthalpy"),1);
/// map.insert("idealGasEntropy".into(),2);
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

pub struct CapeOpenMap<V> (std::collections::HashMap<CapeStringHashKey<'static>,V,fxhash::FxBuildHasher>);

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

	pub fn keys(&self) -> std::collections::hash_map::Keys<'_, CapeStringHashKey<'static>, V> {
		self.0.keys()
	}

	pub fn into_keys(self) -> std::collections::hash_map::IntoKeys<CapeStringHashKey<'static>, V> {
		self.0.into_keys()
	}

	pub fn values(&self) -> std::collections::hash_map::Values<'_, CapeStringHashKey<'static>, V> {
		self.0.values()
	}

	pub fn values_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, CapeStringHashKey<'static>, V> {
		self.0.values_mut()
	}

	pub fn into_values(self) -> std::collections::hash_map::IntoValues<CapeStringHashKey<'static>, V> {
		self.0.into_values()
	}

	pub fn iter(&self) -> std::collections::hash_map::Iter<'_, CapeStringHashKey<'static>, V> {
		self.0.iter()
	}

	pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<'_, CapeStringHashKey<'static>, V> {
		self.0.iter_mut()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn drain(&mut self) -> std::collections::hash_map::Drain<'_, CapeStringHashKey<'static>, V> {
		self.0.drain()
	}

	pub fn retain<F>(&mut self, f: F) 
		where F: FnMut(&CapeStringHashKey<'static>, &mut V) -> bool {
		self.0.retain(f)
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

	pub fn entry(&mut self, key: CapeStringHashKey<'static>) -> std::collections::hash_map::Entry<'_, CapeStringHashKey<'static>, V> {
		self.0.entry(key)
	}

	pub fn get<'a,'b,Q:CapeStringConstProvider>(&'a self, k: &'b Q) -> Option<&'a V> where 'b:'a {
		let k = CapeStringHashKey::from_string_constant(k);
		self.0.get(&k)
	}

	pub fn get_mut<'a>(&'a mut self, k: CapeStringHashKey<'static>) -> Option<&'a mut V> {
		self.0.get_mut(&k)
	}

	pub fn contains_key<Q:CapeStringConstProvider>(&self, k: &Q) -> bool {
		let k = CapeStringHashKey::from_string_constant(k);
		self.0.contains_key(&k)
	}

	pub fn insert_from_cape_string_constant<T:CapeStringConstProvider>(&mut self, k: T, v: V) -> Option<V> {
		let (ptr,size)=k.as_capechar_const_with_length();
		self.0.insert(CapeStringHashKey::from_cape_char_const(ptr,size),v)
	}

	pub fn insert(&mut self, k: CapeStringHashKey<'static>, v: V) -> Option<V> {
		self.0.insert(k,v)
	}

	pub fn remove<'a>(&'a mut self, k: CapeStringHashKey<'static>) -> Option<V> {
		self.0.remove(&k)
	}

	pub fn remove_entry<'a>(&mut self, k: CapeStringHashKey<'static>) -> Option<(CapeStringHashKey<'static>, V)> {
		self.0.remove_entry(&k)
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

