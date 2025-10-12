use crate::C;

/// Any object that implements this trait can be converted to an ICapeString input argument
///
/// This trait is implemented by any object that can be passed as an ICapeString input
/// argument
pub trait CapeStringProviderIn {
	/// Convert to ICapeString
	fn as_cape_string_in(&self) -> C::ICapeString;
}

/// Any object that implements this trait can be converted to an ICapeString output argument
///
/// This trait is implemented by any object that can be passed as an ICapeString output
/// argument
pub trait CapeStringProviderOut {
	/// Convert to ICapeString
	fn as_cape_string_out(&mut self) -> C::ICapeString;
}

/// Any object that implements this trait can be compared to any CapeString type object
///
pub trait CapeStringConstProvider {
	/// Convert to const CapeCharacter pointer (null terminated)
	///
	/// Note that the caller must ensure that life span of the pointer does not exceed the life
	/// span of the object and note that the pointer becomes invalid upon assiging a new string
	/// to the object
	/// 
	fn as_capechar_const(&self) -> *const C::CapeCharacter;
	/// Convert to const CapeCharacter pointer (null terminated) and length (without null terminator)
	///
	/// Note that the caller must ensure that life span of the pointer does not exceed the life
	/// span of the object and note that the pointer becomes invalid upon assiging a new string
	/// to the object
	/// 
	fn as_capechar_const_with_length(&self) -> (*const C::CapeCharacter, C::CapeSize);
}

/// Any object that implements this trait can be converted to an ICapeArrayString input argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayString
/// input argument
pub trait CapeArrayStringProviderIn {
	/// Convert to ICapeArrayString
	fn as_cape_array_string_in(&self) -> C::ICapeArrayString;
}

/// Any object that implements this trait can be converted to an ICapeArrayString output argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayString
/// output argument
pub trait CapeArrayStringProviderOut {
	/// Convert to ICapeArrayString
	fn as_cape_array_string_out(&mut self) -> C::ICapeArrayString;
}

/// Any object that implements this trait can be converted to an ICapeArrayReal input argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayReal
/// input argument
pub trait CapeArrayRealProviderIn {
	/// Convert to ICapeArrayReal
	fn as_cape_array_real_in(&self) -> C::ICapeArrayReal;
}

/// Any object that implements this trait can be converted to an ICapeArrayReal output argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayReal
/// output argument
pub trait CapeArrayRealProviderOut {
	/// Convert to ICapeArrayReal
	fn as_cape_array_real_out(&mut self) -> C::ICapeArrayReal;
}

/// Any object that implements this trait can be converted to an ICapeArrayInteger input argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayInteger
/// input argument
pub trait CapeArrayIntegerProviderIn {
	/// Convert to ICapeArrayInteger
	fn as_cape_array_integer_in(&self) -> C::ICapeArrayInteger;
}

/// Any object that implements this trait can be converted to an ICapeArrayInteger output argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayInteger
/// output argument
pub trait CapeArrayIntegerProviderOut {
	/// Convert to ICapeArrayInteger
	fn as_cape_array_integer_out(&mut self) -> C::ICapeArrayInteger;
}

/// Any object that implements this trait can be converted to an ICapeArrayByte input argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayByte
/// input argument
pub trait CapeArrayByteProviderIn {
	/// Convert to ICapeArrayByte
	fn as_cape_array_byte_in(&self) -> C::ICapeArrayByte;
}

/// Any object that implements this trait can be converted to an ICapeArrayByte output argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayByte
/// output argument
pub trait CapeArrayByteProviderOut {
	/// Convert to ICapeArrayByte
	fn as_cape_array_byte_out(&mut self) -> C::ICapeArrayByte;
}

/// Any object that implements this trait can be converted to an ICapeArrayBoolean input argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayBoolean
/// input argument
pub trait CapeArrayBooleanProviderIn {
	/// Convert to ICapeArrayBoolean
	fn as_cape_array_boolean_in(&self) -> C::ICapeArrayBoolean;
}

/// Any object that implements this trait can be converted to an ICapeArrayBoolean output argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayBoolean
/// output argument
pub trait CapeArrayBooleanProviderOut {
	/// Convert to ICapeArrayBoolean
	fn as_cape_array_boolean_out(&mut self) -> C::ICapeArrayBoolean;
}

/// Any object that implements this trait can be converted to an ICapeArrayEnumeration input argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayEnumeration
/// input argument
pub trait CapeArrayEnumerationProviderIn {
	/// Convert to ICapeArrayEnumeration
	fn as_cape_array_enumeration_in(&self) -> C::ICapeArrayEnumeration;
}

/// Any object that implements this trait can be converted to an ICapeArrayEnumeration output argument
///
/// This trait is implemented by any object that can be passed as an ICapeArrayEnumeration
/// output argument
pub trait CapeArrayEnumerationProviderOut {
	/// Convert to ICapeArrayEnumeration
	fn as_cape_array_enumeration_out(&mut self) -> C::ICapeArrayEnumeration;
}

/// Any object that implements this trait can be converted to an ICapeValue input argument
///
/// This trait is implemented by any object that can be passed as an ICapeValue
/// input argument
pub trait CapeValueProviderIn {
	/// Convert to ICapeValue
	fn as_cape_value_in(&self) -> C::ICapeValue;
}

/// Any object that implements this trait can be converted to an ICapeValue output argument
///
/// This trait is implemented by any object that can be passed as an ICapeValue
/// output argument
pub trait CapeValueProviderOut {
	/// Convert to ICapeValue
	fn as_cape_value_out(&mut self) -> C::ICapeValue;
}


/// Any object that implements this trait can be converted to an ICapeArrayValue input argument
/// 
/// This trait is implemented by any object that can be passed as an ICapeArrayValue
/// input argument
pub trait CapeArrayValueProviderIn {
	/// Convert to ICapeArrayValue
	fn as_cape_array_value_in(&self) -> C::ICapeArrayValue;
}

/// Any object that implements this trait can be converted to an ICapeArrayValue output argument
/// 
/// This trait is implemented by any object that can be passed as an ICapeArrayValue
/// output argument
pub trait CapeArrayValueProviderOut {
	/// Convert to ICapeArrayValue
	fn as_cape_array_value_out(&mut self) -> C::ICapeArrayValue;
}
