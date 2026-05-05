use std::fmt;

///CapeWindowType enumeration
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CapeWindowType {
    None=0,
    X11=1,
    Wayland=2,
    COCOA=3,
}

impl CapeWindowType {
    ///convert from i32 to CapeWindowType
    ///
    /// # Arguments
    ///
    /// * `value` - i32 value to be converted to CapeWindowType
    ///
    /// # Examples
    ///
    /// ```
    /// use cobia;
    ///let v0=cobia::CapeWindowType::from(0);
    ///assert_eq!(v0.unwrap(),cobia::CapeWindowType::None);
    ///let v1=cobia::CapeWindowType::from(1);
    ///assert_eq!(v1.unwrap(),cobia::CapeWindowType::X11);
    ///let v2=cobia::CapeWindowType::from(2);
    ///assert_eq!(v2.unwrap(),cobia::CapeWindowType::Wayland);
    ///let v3=cobia::CapeWindowType::from(3);
    ///assert_eq!(v3.unwrap(),cobia::CapeWindowType::COCOA);
    ///let v4=cobia::CapeWindowType::from(-1);
    ///assert_eq!(v4,None);
    ///```
    pub fn from(value: i32) -> Option<CapeWindowType> {
        match value {
            0 => Some(CapeWindowType::None),
            1 => Some(CapeWindowType::X11),
            2 => Some(CapeWindowType::Wayland),
            3 => Some(CapeWindowType::COCOA),
            _ => None,
        }
    }
    /// Convert to string
    pub fn as_string(&self) -> &str {
        match self {
			Self::None => "None",
			Self::X11 => "X11",
			Self::Wayland => "Wayland",
			Self::COCOA => "COCOA",
        }
    }
    ///get an iterator
    ///
    /// # Examples
    ///
    /// ```
    /// use cobia;
    /// for capeWindowType in cobia::CapeWindowType::iter() {
    ///     println!("{}={}",capeWindowType,capeWindowType as i32);
    /// }
    /// ```
    pub fn iter() -> CapeWindowTypeIterator {
		CapeWindowTypeIterator { current: 0 }
	}
}

/// CapeWindowType iterator
///
/// Iterates over all CapeWindowType values
///
/// Example:
/// ```
/// use cobia;
/// for capeWindowType in cobia::CapeWindowType::iter() {
///     println!("{}={}",capeWindowType,capeWindowType as i32);
/// }
/// ```
pub struct CapeWindowTypeIterator {
    current: i32,
}
impl Iterator for CapeWindowTypeIterator {
    type Item = CapeWindowType;
	fn next(&mut self) -> Option<Self::Item> {
        if self.current >= 4 {
			None
		} else {
		    let result = CapeWindowType::from(self.current);
		    self.current += 1;
		    result
        }
	}
}
impl fmt::Display for CapeWindowType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.as_string())
	}
}

///WaylandForeignProtocol enumeration
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WaylandForeignProtocol {
    XDGforeignv1=1,
    XDGforeignv2=2,
}

impl WaylandForeignProtocol {
    ///convert from i32 to WaylandForeignProtocol
    ///
    /// # Arguments
    ///
    /// * `value` - i32 value to be converted to WaylandForeignProtocol
    ///
    /// # Examples
    ///
    /// ```
    /// use cobia;
    ///let v0=cobia::WaylandForeignProtocol::from(1);
    ///assert_eq!(v0.unwrap(),cobia::WaylandForeignProtocol::XDGforeignv1);
    ///let v1=cobia::WaylandForeignProtocol::from(2);
    ///assert_eq!(v1.unwrap(),cobia::WaylandForeignProtocol::XDGforeignv2);
    ///let v2=cobia::WaylandForeignProtocol::from(-1);
    ///assert_eq!(v2,None);
    ///```
    pub fn from(value: i32) -> Option<WaylandForeignProtocol> {
        match value {
            1 => Some(WaylandForeignProtocol::XDGforeignv1),
            2 => Some(WaylandForeignProtocol::XDGforeignv2),
            _ => None,
        }
    }
    /// Convert to string
    pub fn as_string(&self) -> &str {
        match self {
			Self::XDGforeignv1 => "XDGforeignv1",
			Self::XDGforeignv2 => "XDGforeignv2",
        }
    }
    ///get an iterator
    ///
    /// # Examples
    ///
    /// ```
    /// use cobia;
    /// for waylandForeignProtocol in cobia::WaylandForeignProtocol::iter() {
    ///     println!("{}={}",waylandForeignProtocol,waylandForeignProtocol as i32);
    /// }
    /// ```
    pub fn iter() -> WaylandForeignProtocolIterator {
		WaylandForeignProtocolIterator { current: 0 }
	}
}

/// WaylandForeignProtocol iterator
///
/// Iterates over all WaylandForeignProtocol values
///
/// Example:
/// ```
/// use cobia;
/// for waylandForeignProtocol in cobia::WaylandForeignProtocol::iter() {
///     println!("{}={}",waylandForeignProtocol,waylandForeignProtocol as i32);
/// }
/// ```
pub struct WaylandForeignProtocolIterator {
    current: i32,
}
impl Iterator for WaylandForeignProtocolIterator {
    type Item = WaylandForeignProtocol;
	fn next(&mut self) -> Option<Self::Item> {
        if self.current >= 2 {
			None
		} else {
		    let result = WaylandForeignProtocol::from(self.current);
		    self.current += 1;
		    result
        }
	}
}
impl fmt::Display for WaylandForeignProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.as_string())
	}
}

