use bitflags::bitflags;
use std::fmt;

///Registry value type
///
///The supported types of registry values.
///
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CapeRegistryValueType {
    String=0,
    Integer=1,
    UUID=2,
    Empty=3,
}

impl CapeRegistryValueType {
    ///convert from i32 to CapeRegistryValueType
    ///
    /// # Arguments
    ///
    /// * `value` - i32 value to be converted to CapeRegistryValueType
    ///
    /// # Examples
    ///
    /// ```
    /// use cobia;
    ///let v0=cobia::CapeRegistryValueType::from(0);
    ///assert_eq!(v0.unwrap(),cobia::CapeRegistryValueType::String);
    ///let v1=cobia::CapeRegistryValueType::from(1);
    ///assert_eq!(v1.unwrap(),cobia::CapeRegistryValueType::Integer);
    ///let v2=cobia::CapeRegistryValueType::from(2);
    ///assert_eq!(v2.unwrap(),cobia::CapeRegistryValueType::UUID);
    ///let v3=cobia::CapeRegistryValueType::from(3);
    ///assert_eq!(v3.unwrap(),cobia::CapeRegistryValueType::Empty);
    ///let v4=cobia::CapeRegistryValueType::from(-1);
    ///assert_eq!(v4,None);
    ///```
    pub fn from(value: i32) -> Option<CapeRegistryValueType> {
        match value {
            0 => Some(CapeRegistryValueType::String),
            1 => Some(CapeRegistryValueType::Integer),
            2 => Some(CapeRegistryValueType::UUID),
            3 => Some(CapeRegistryValueType::Empty),
            _ => None,
        }
    }
    /// Convert to string
    pub fn as_string(&self) -> &str {
        match self {
			Self::String => "String",
			Self::Integer => "Integer",
			Self::UUID => "UUID",
			Self::Empty => "Empty",
        }
    }
    ///get an iterator
    ///
    /// # Examples
    ///
    /// ```
    /// use cobia;
    /// for capeRegistryValueType in cobia::CapeRegistryValueType::iter() {
    ///     println!("{}={}",capeRegistryValueType,capeRegistryValueType as i32);
    /// }
    /// ```
    pub fn iter() -> CapeRegistryValueTypeIterator {
		CapeRegistryValueTypeIterator { current: 0 }
	}
}

/// CapeRegistryValueType iterator
///
/// Iterates over all CapeRegistryValueType values
///
/// Example:
/// ```
/// use cobia;
/// for capeRegistryValueType in cobia::CapeRegistryValueType::iter() {
///     println!("{}={}",capeRegistryValueType,capeRegistryValueType as i32);
/// }
/// ```
pub struct CapeRegistryValueTypeIterator {
    current: i32,
}
impl Iterator for CapeRegistryValueTypeIterator {
    type Item = CapeRegistryValueType;
	fn next(&mut self) -> Option<Self::Item> {
        if self.current >= 4 {
			None
		} else {
		    let result = CapeRegistryValueType::from(self.current);
		    self.current += 1;
		    result
        }
	}
}
impl fmt::Display for CapeRegistryValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.as_string())
	}
}

///CapeValue data types
///
///The supported types of CapeValue values.
///
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CapeValueType {
    String=0,
    Integer=1,
    Boolean=2,
    Real=3,
    Empty=4,
}

impl CapeValueType {
    ///convert from i32 to CapeValueType
    ///
    /// # Arguments
    ///
    /// * `value` - i32 value to be converted to CapeValueType
    ///
    /// # Examples
    ///
    /// ```
    /// use cobia;
    ///let v0=cobia::CapeValueType::from(0);
    ///assert_eq!(v0.unwrap(),cobia::CapeValueType::String);
    ///let v1=cobia::CapeValueType::from(1);
    ///assert_eq!(v1.unwrap(),cobia::CapeValueType::Integer);
    ///let v2=cobia::CapeValueType::from(2);
    ///assert_eq!(v2.unwrap(),cobia::CapeValueType::Boolean);
    ///let v3=cobia::CapeValueType::from(3);
    ///assert_eq!(v3.unwrap(),cobia::CapeValueType::Real);
    ///let v4=cobia::CapeValueType::from(4);
    ///assert_eq!(v4.unwrap(),cobia::CapeValueType::Empty);
    ///let v5=cobia::CapeValueType::from(-1);
    ///assert_eq!(v5,None);
    ///```
    pub fn from(value: i32) -> Option<CapeValueType> {
        match value {
            0 => Some(CapeValueType::String),
            1 => Some(CapeValueType::Integer),
            2 => Some(CapeValueType::Boolean),
            3 => Some(CapeValueType::Real),
            4 => Some(CapeValueType::Empty),
            _ => None,
        }
    }
    /// Convert to string
    pub fn as_string(&self) -> &str {
        match self {
			Self::String => "String",
			Self::Integer => "Integer",
			Self::Boolean => "Boolean",
			Self::Real => "Real",
			Self::Empty => "Empty",
        }
    }
    ///get an iterator
    ///
    /// # Examples
    ///
    /// ```
    /// use cobia;
    /// for capeValueType in cobia::CapeValueType::iter() {
    ///     println!("{}={}",capeValueType,capeValueType as i32);
    /// }
    /// ```
    pub fn iter() -> CapeValueTypeIterator {
		CapeValueTypeIterator { current: 0 }
	}
}

/// CapeValueType iterator
///
/// Iterates over all CapeValueType values
///
/// Example:
/// ```
/// use cobia;
/// for capeValueType in cobia::CapeValueType::iter() {
///     println!("{}={}",capeValueType,capeValueType as i32);
/// }
/// ```
pub struct CapeValueTypeIterator {
    current: i32,
}
impl Iterator for CapeValueTypeIterator {
    type Item = CapeValueType;
	fn next(&mut self) -> Option<Self::Item> {
        if self.current >= 5 {
			None
		} else {
		    let result = CapeValueType::from(self.current);
		    self.current += 1;
		    result
        }
	}
}
impl fmt::Display for CapeValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.as_string())
	}
}

///Service provider types
///
///Service type enumeration for PMC instantiation
///
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CapePMCServiceType {
    Inproc32=0,
    Inproc64=1,
    COM32=2,
    COM64=3,
    Remote=4,
    Local=5,
}

impl CapePMCServiceType {
    ///convert from i32 to CapePMCServiceType
    ///
    /// # Arguments
    ///
    /// * `value` - i32 value to be converted to CapePMCServiceType
    ///
    /// # Examples
    ///
    /// ```
    /// use cobia;
    ///let v0=cobia::CapePMCServiceType::from(0);
    ///assert_eq!(v0.unwrap(),cobia::CapePMCServiceType::Inproc32);
    ///let v1=cobia::CapePMCServiceType::from(1);
    ///assert_eq!(v1.unwrap(),cobia::CapePMCServiceType::Inproc64);
    ///let v2=cobia::CapePMCServiceType::from(2);
    ///assert_eq!(v2.unwrap(),cobia::CapePMCServiceType::COM32);
    ///let v3=cobia::CapePMCServiceType::from(3);
    ///assert_eq!(v3.unwrap(),cobia::CapePMCServiceType::COM64);
    ///let v4=cobia::CapePMCServiceType::from(4);
    ///assert_eq!(v4.unwrap(),cobia::CapePMCServiceType::Remote);
    ///let v5=cobia::CapePMCServiceType::from(5);
    ///assert_eq!(v5.unwrap(),cobia::CapePMCServiceType::Local);
    ///let v6=cobia::CapePMCServiceType::from(-1);
    ///assert_eq!(v6,None);
    ///```
    pub fn from(value: i32) -> Option<CapePMCServiceType> {
        match value {
            0 => Some(CapePMCServiceType::Inproc32),
            1 => Some(CapePMCServiceType::Inproc64),
            2 => Some(CapePMCServiceType::COM32),
            3 => Some(CapePMCServiceType::COM64),
            4 => Some(CapePMCServiceType::Remote),
            5 => Some(CapePMCServiceType::Local),
            _ => None,
        }
    }
    /// Convert to string
    pub fn as_string(&self) -> &str {
        match self {
			Self::Inproc32 => "Inproc32",
			Self::Inproc64 => "Inproc64",
			Self::COM32 => "COM32",
			Self::COM64 => "COM64",
			Self::Remote => "Remote",
			Self::Local => "Local",
        }
    }
    ///get an iterator
    ///
    /// # Examples
    ///
    /// ```
    /// use cobia;
    /// for capePMCServiceType in cobia::CapePMCServiceType::iter() {
    ///     println!("{}={}",capePMCServiceType,capePMCServiceType as i32);
    /// }
    /// ```
    pub fn iter() -> CapePMCServiceTypeIterator {
		CapePMCServiceTypeIterator { current: 0 }
	}
}

/// CapePMCServiceType iterator
///
/// Iterates over all CapePMCServiceType values
///
/// Example:
/// ```
/// use cobia;
/// for capePMCServiceType in cobia::CapePMCServiceType::iter() {
///     println!("{}={}",capePMCServiceType,capePMCServiceType as i32);
/// }
/// ```
pub struct CapePMCServiceTypeIterator {
    current: i32,
}
impl Iterator for CapePMCServiceTypeIterator {
    type Item = CapePMCServiceType;
	fn next(&mut self) -> Option<Self::Item> {
        if self.current >= 6 {
			None
		} else {
		    let result = CapePMCServiceType::from(self.current);
		    self.current += 1;
		    result
        }
	}
}
impl fmt::Display for CapePMCServiceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.as_string())
	}
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct CapePMCRegistrationFlags : i32 {
        const None = 0;
        const RestrictedThreading = 1;
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct CapePMCCreationFlags : i32 {
        const Default = 0;
        const AllowRestrictedThreading = 1;
    }
}

