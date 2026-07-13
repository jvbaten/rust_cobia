use base64::prelude::*;
use cobia::*;

/// A CapeValue as stored inside a persistence writer/reader.
///
pub(crate) enum PersistedCapeValue {
    Empty,
    Real(f64),
    Integer(i32),
    Boolean(bool),
    String(String),
}

/// Any persistable value that a CAPE-OPEN component can write via
/// ICapePersistWriter.
///
/// Scalar variants correspond to the primitive CAPE types. `Value` wraps the
/// variant [`PersistedCapeValue`] type. `Array*` variants hold homogeneous
/// sequences.
pub(crate) enum PersistedValue {
    Real(f64),
    Integer(i32),
    Boolean(bool),
    String(String),
    Enumeration(u32),
    Value(PersistedCapeValue),
    ArrayReal(Vec<f64>),
    ArrayInteger(Vec<i32>),
    ArrayBoolean(Vec<bool>),
    ArrayString(Vec<String>),
    ArrayEnumeration(Vec<u32>),
    ArrayValue(Vec<PersistedCapeValue>),
    ArrayByte(Vec<u8>),
}

/// In-memory write-side of the CAPE-OPEN persistence protocol.
///
/// Collects named values and named sub-nodes written by a CAPE component via
/// ICapePersistWriter, then serialises the tree to XML with
/// [`PersistWriter::write_xml`].
#[cape_object_implementation(
		interfaces = {
			cape_open_1_2::ICapeIdentification,
			cape_open_1_2::ICapePersistWriter,
		},
        create_arguments = {path}
  )]
pub(crate) struct PersistWriter {
    /// Path of this node within the persistence tree, for identification and debugging purposes.
    path: String,
    /// Values directly written to this node, keyed by name.
    values: std::collections::HashMap<String, PersistedValue>,
    /// Sub-nodes directly written to this node, keyed by name.
    sub_nodes: std::collections::HashMap<String, cobia::CapeObject>,
}

impl std::fmt::Display for PersistWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PeristWriter \"{}\"", self.path)
    }
}

impl PersistWriter {
    /// Formats a `f64` for XML storage, including the special tokens `INF`,
    /// `-INF`, and `NaN`.
    ///
    /// # Parameters
    /// - `value`: Any finite or non-finite `f64`.
    ///
    /// # Returns
    /// The Rust `Display` representation for finite values, or `"INF"`,
    /// `"-INF"`, `"NaN"` for the respective special values.
    fn format_f64(value: f64) -> String {
        if value.is_finite() {
            value.to_string()
        } else if value.is_infinite() {
            if value.is_sign_positive() {
                "INF".to_string()
            } else {
                "-INF".to_string()
            }
        } else {
            "NaN".to_string()
        }
    }

    /// Creates a root-level writer accessible as a CAPE CapePersistWriter
    /// smart pointer, using `"/"` as the path.
    ///
    /// # Returns
    /// A CapePersistWriter wrapping a fresh empty [`PersistWriter`].
    pub fn root_1_2() -> cape_open_1_2::CapePersistWriter {
        Self::create("/".to_string())
    }

    /// Serialises the collected values and sub-nodes into the supplied XML
    /// writer.
    ///
    /// Each [`PersistedValue`] variant is emitted as a typed XML element
    /// carrying a `name` attribute. Arrays of `Real`, `Integer`, `Boolean`,
    /// and `Enumeration` use semicolon-separated text content; `ArrayString`
    /// and `ArrayValue` use child `<Element>` elements. `ArrayByte` is
    /// base-64 encoded. Sub-nodes become `<Node name="…">` wrappers.
    ///
    /// # Parameters
    /// - `writer`: A `quick_xml` writer to write elements into.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(e)` if any XML write or nested-node serialisation fails.
    pub fn write_xml<T>(
        &self,
        writer: &mut quick_xml::writer::Writer<T>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        T: std::io::Write,
    {
        for (value_name, value) in &self.values {
            match value {
                PersistedValue::Real(value) => {
                    writer
                        .create_element("Real")
                        .with_attribute(("name", value_name.as_str()))
                        .write_text_content(quick_xml::events::BytesText::new(
                            &Self::format_f64(*value),
                        ))?;
                }
                PersistedValue::Integer(value) => {
                    writer
                        .create_element("Integer")
                        .with_attribute(("name", value_name.as_str()))
                        .write_text_content(quick_xml::events::BytesText::new(
                            &value.to_string(),
                        ))?;
                }
                PersistedValue::Boolean(value) => {
                    writer
                        .create_element("Boolean")
                        .with_attribute(("name", value_name.as_str()))
                        .write_text_content(quick_xml::events::BytesText::new(if *value {
                            "true"
                        } else {
                            "false"
                        }))?;
                }
                PersistedValue::String(value) => {
                    writer
                        .create_element("String")
                        .with_attribute(("name", value_name.as_str()))
                        .write_text_content(quick_xml::events::BytesText::new(value.as_str()))?;
                }
                PersistedValue::Enumeration(value) => {
                    writer
                        .create_element("Enumeration")
                        .with_attribute(("name", value_name.as_str()))
                        .write_text_content(quick_xml::events::BytesText::new(
                            &value.to_string(),
                        ))?;
                }
                PersistedValue::Value(value) => {
                    let value_writer = writer
                        .create_element("Value")
                        .with_attribute(("name", value_name.as_str()));
                    match value {
                        PersistedCapeValue::Real(v) => {
                            value_writer
                                .with_attribute(("value_type", "Real"))
                                .write_text_content(quick_xml::events::BytesText::new(
                                    &Self::format_f64(*v),
                                ))?;
                        }
                        PersistedCapeValue::Integer(v) => {
                            value_writer
                                .with_attribute(("value_type", "Integer"))
                                .write_text_content(quick_xml::events::BytesText::new(
                                    &v.to_string(),
                                ))?;
                        }
                        PersistedCapeValue::Boolean(v) => {
                            value_writer
                                .with_attribute(("value_type", "Boolean"))
                                .write_text_content(quick_xml::events::BytesText::new(if *v {
                                    "true"
                                } else {
                                    "false"
                                }))?;
                        }
                        PersistedCapeValue::String(v) => {
                            value_writer
                                .with_attribute(("value_type", "String"))
                                .write_text_content(quick_xml::events::BytesText::new(
                                    v.as_str(),
                                ))?;
                        }
                        PersistedCapeValue::Empty => {
                            value_writer
                                .with_attribute(("value_type", "Empty"))
                                .write_empty()?;
                        }
                    }
                }
                PersistedValue::ArrayReal(value) => {
                    writer
                        .create_element("ArrayReal")
                        .with_attribute(("name", value_name.as_str()))
                        .write_text_content(quick_xml::events::BytesText::new(
                            &value
                                .iter()
                                .map(|v| Self::format_f64(*v))
                                .collect::<Vec<String>>()
                                .join(";"),
                        ))?;
                }
                PersistedValue::ArrayInteger(value) => {
                    writer
                        .create_element("ArrayInteger")
                        .with_attribute(("name", value_name.as_str()))
                        .write_text_content(quick_xml::events::BytesText::new(
                            &value
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(";"),
                        ))?;
                }
                PersistedValue::ArrayBoolean(value) => {
                    writer
                        .create_element("ArrayBoolean")
                        .with_attribute(("name", value_name.as_str()))
                        .write_text_content(quick_xml::events::BytesText::new(
                            &value
                                .iter()
                                .map(|v| if *v { "true" } else { "false" })
                                .collect::<Vec<&str>>()
                                .join(";"),
                        ))?;
                }
                PersistedValue::ArrayString(value) => {
                    //cannot escape string, write as multiple Element elements
                    writer
                        .create_element("ArrayString")
                        .with_attribute(("name", value_name.as_str()))
                        .write_inner_content(|f| {
                            for s in value {
                                f.create_element("Element").write_text_content(
                                    quick_xml::events::BytesText::new(s.as_str()),
                                )?;
                            }
                            Ok(())
                        })?;
                }
                PersistedValue::ArrayEnumeration(value) => {
                    writer
                        .create_element("ArrayEnumeration")
                        .with_attribute(("name", value_name.as_str()))
                        .write_text_content(quick_xml::events::BytesText::new(
                            &value
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(";"),
                        ))?;
                }
                PersistedValue::ArrayValue(value) => {
                    //cannot escape string, write as multiple Element elements
                    writer
                        .create_element("ArrayValue")
                        .with_attribute(("name", value_name.as_str()))
                        .write_inner_content(|f| {
                            for s in value {
                                let value_writer = f.create_element("Element");
                                match s {
                                    PersistedCapeValue::Real(v) => {
                                        value_writer
                                            .with_attribute(("value_type", "Real"))
                                            .write_text_content(
                                                quick_xml::events::BytesText::new(
                                                    &Self::format_f64(*v),
                                                ),
                                            )?;
                                    }
                                    PersistedCapeValue::Integer(v) => {
                                        value_writer
                                            .with_attribute(("value_type", "Integer"))
                                            .write_text_content(
                                                quick_xml::events::BytesText::new(&v.to_string()),
                                            )?;
                                    }
                                    PersistedCapeValue::Boolean(v) => {
                                        value_writer
                                            .with_attribute(("value_type", "Boolean"))
                                            .write_text_content(
                                                quick_xml::events::BytesText::new(if *v {
                                                    "true"
                                                } else {
                                                    "false"
                                                }),
                                            )?;
                                    }
                                    PersistedCapeValue::String(v) => {
                                        value_writer
                                            .with_attribute(("value_type", "String"))
                                            .write_text_content(
                                                quick_xml::events::BytesText::new(v.as_str()),
                                            )?;
                                    }
                                    PersistedCapeValue::Empty => {
                                        value_writer
                                            .with_attribute(("value_type", "Empty"))
                                            .write_empty()?;
                                    }
                                }
                            }
                            Ok(())
                        })?;
                }
                PersistedValue::ArrayByte(value) => {
                    writer
                        .create_element("ArrayByte")
                        .with_attribute(("name", value_name.as_str()))
                        .write_text_content(quick_xml::events::BytesText::new(
                            &BASE64_STANDARD.encode(value),
                        ))?;
                }
            }
        }
        for (node_name, node) in &self.sub_nodes {
            let node_writer = writer
                .create_element("Node")
                .with_attribute(("name", node_name.as_str()));
            node_writer.write_inner_content(|w| {
                match unsafe { PersistWriter::borrow(node) }.write_xml(w) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        Err(std::io::Error::other(
                            format!("Failed to write node {}: {}", node_name, e),
                        ))
                    }
                }
            })?;
        }
        Ok(())
    }
}

impl cape_open_1_2::ICapeIdentification for PersistWriter {
    fn get_component_name(&mut self, name: &mut CapeStringOut) -> Result<(), COBIAError> {
        name.set_string(&self.path)
    }

    fn set_component_name(&mut self, _name: &CapeStringIn) -> Result<(), COBIAError> {
        Err(COBIAError::Code(COBIAERR_DENIED))
    }

    fn get_component_description(&mut self, desc: &mut CapeStringOut) -> Result<(), COBIAError> {
        desc.set_string(format!("PeristWriter {}", self.path))
    }

    fn set_component_description(&mut self, _desc: &CapeStringIn) -> Result<(), COBIAError> {
        Err(COBIAError::Code(COBIAERR_DENIED))
    }
}

impl cape_open_1_2::ICapePersistWriter for PersistWriter {
    fn add_real(&mut self, value_name: &CapeStringIn, value: CapeReal) -> Result<(), COBIAError> {
        match self
            .values
            .insert(value_name.to_string(), PersistedValue::Real(value))
        {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_integer(
        &mut self,
        value_name: &CapeStringIn,
        value: CapeInteger,
    ) -> Result<(), COBIAError> {
        match self
            .values
            .insert(value_name.to_string(), PersistedValue::Integer(value))
        {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_boolean(
        &mut self,
        value_name: &CapeStringIn,
        value: CapeBoolean,
    ) -> Result<(), COBIAError> {
        match self
            .values
            .insert(value_name.to_string(), PersistedValue::Boolean(value != 0))
        {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_string(
        &mut self,
        value_name: &CapeStringIn,
        value: &CapeStringIn,
    ) -> Result<(), COBIAError> {
        match self.values.insert(
            value_name.to_string(),
            PersistedValue::String(value.to_string()),
        ) {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_enumeration(
        &mut self,
        value_name: &CapeStringIn,
        value: cobia::CapeEnumeration,
    ) -> Result<(), COBIAError> {
        match self.values.insert(
            value_name.to_string(),
            PersistedValue::Enumeration(value as u32),
        ) {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_value(
        &mut self,
        value_name: &CapeStringIn,
        value: &CapeValueIn,
    ) -> Result<(), COBIAError> {
        let cape_value = match value.get_type() {
            Ok(value_type) => match value_type {
                CapeValueType::Real => PersistedCapeValue::Real(value.get_real()?),
                CapeValueType::Integer => PersistedCapeValue::Integer(value.get_integer()?),
                CapeValueType::Boolean => PersistedCapeValue::Boolean(value.get_boolean()?),
                CapeValueType::String => PersistedCapeValue::String(value.get_string()?),
                CapeValueType::Empty => PersistedCapeValue::Empty,
            },
            Err(e) => {
                return Err(COBIAError::Message(format!(
                    "Failed to get value type for value {}: {}",
                    value_name, e
                )));
            }
        };
        match self
            .values
            .insert(value_name.to_string(), PersistedValue::Value(cape_value))
        {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_array_real(
        &mut self,
        value_name: &CapeStringIn,
        value: &CapeArrayRealIn,
    ) -> Result<(), COBIAError> {
        match self.values.insert(
            value_name.to_string(),
            PersistedValue::ArrayReal(value.as_vec()),
        ) {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_array_integer(
        &mut self,
        value_name: &CapeStringIn,
        value: &CapeArrayIntegerIn,
    ) -> Result<(), COBIAError> {
        match self.values.insert(
            value_name.to_string(),
            PersistedValue::ArrayInteger(value.as_vec()),
        ) {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_array_boolean(
        &mut self,
        value_name: &CapeStringIn,
        value: &CapeArrayBooleanIn,
    ) -> Result<(), COBIAError> {
        match self.values.insert(
            value_name.to_string(),
            PersistedValue::ArrayBoolean(value.as_vec().into_iter().map(|v| v != 0).collect()),
        ) {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_array_string(
        &mut self,
        value_name: &CapeStringIn,
        value: &CapeArrayStringIn,
    ) -> Result<(), COBIAError> {
        match self.values.insert(
            value_name.to_string(),
            PersistedValue::ArrayString(value.as_string_vec()),
        ) {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_array_enumeration(
        &mut self,
        value_name: &CapeStringIn,
        value: &CapeArrayEnumerationIn<CapeEnumeration>,
    ) -> Result<(), COBIAError> {
        match self.values.insert(
            value_name.to_string(),
            PersistedValue::ArrayEnumeration(
                value.as_vec().into_iter().map(|v| v as u32).collect(),
            ),
        ) {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_array_value(
        &mut self,
        value_name: &CapeStringIn,
        value: &CapeArrayValueIn,
    ) -> Result<(), COBIAError> {
        let mut vec = Vec::new();
        for i in 0..value.size() {
            let element = value.at(i)?;
            let cape_value = match element.get_type() {
                Ok(value_type) => match value_type {
                    CapeValueType::Real => PersistedCapeValue::Real(element.get_real()?),
                    CapeValueType::Integer => PersistedCapeValue::Integer(element.get_integer()?),
                    CapeValueType::Boolean => PersistedCapeValue::Boolean(element.get_boolean()?),
                    CapeValueType::String => PersistedCapeValue::String(element.get_string()?),
                    CapeValueType::Empty => PersistedCapeValue::Empty,
                },
                Err(e) => {
                    return Err(COBIAError::Message(format!(
                        "Failed to get value type for value {} at index {}: {}",
                        value_name, i, e
                    )));
                }
            };
            vec.push(cape_value);
        }
        match self
            .values
            .insert(value_name.to_string(), PersistedValue::ArrayValue(vec))
        {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_array_byte(
        &mut self,
        value_name: &CapeStringIn,
        value: &CapeArrayByteIn,
    ) -> Result<(), COBIAError> {
        match self.values.insert(
            value_name.to_string(),
            PersistedValue::ArrayByte(value.as_vec()),
        ) {
            Some(_) => Err(COBIAError::Message(format!(
                "Value with name {} already exists",
                value_name
            ))),
            None => Ok(()),
        }
    }

    fn add_node(
        &mut self,
        node_name: &CapeStringIn,
    ) -> Result<cape_open_1_2::CapePersistWriter, COBIAError> {
        let path = format!("{}/{}", self.path, node_name);
        let obj = Self::create(path);
        let new_node = cape_open_1_2::CapePersistWriter::from_object(&obj)?;
        match self.sub_nodes.insert(node_name.to_string(), obj) {
            Some(_) => Err(COBIAError::Message(format!(
                "Node with name {} already exists",
                node_name
            ))),
            None => Ok(new_node),
        }
    }
}

/// In-memory read-side of the CAPE-OPEN persistence protocol.
///
/// Provides values and sub-nodes read back from a [`PersistWriter`] (via
/// `PersistReader::from_writer`]) or from XML (via `PersistReader::from_xml`).
/// CAPE components retrieve values through the ICapePersistReader interface.
#[cape_object_implementation(
		interfaces = {
			cape_open_1_2::ICapeIdentification,
			cape_open_1_2::ICapePersistReader,
		},
        create_arguments = {path}
  )]
pub(crate) struct PersistReader {
    /// Path of this node within the persistence tree, for identification and debugging purposes.
    path: String,
    /// Values directly read from this node, keyed by name.
    values: std::collections::HashMap<String, PersistedValue>,
    /// Sub-nodes directly read from this node, keyed by name.
    sub_nodes: std::collections::HashMap<String, cobia::CapeObject>,
}

impl std::fmt::Display for PersistReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PersistReader \"{}\"", self.path)
    }
}

impl cape_open_1_2::ICapeIdentification for PersistReader {
    fn get_component_name(&mut self, name: &mut CapeStringOut) -> Result<(), COBIAError> {
        name.set_string(&self.path)?;
        Ok(())
    }

    fn set_component_name(&mut self, _name: &CapeStringIn) -> Result<(), COBIAError> {
        Err(COBIAError::Code(COBIAERR_DENIED))
    }

    fn get_component_description(&mut self, desc: &mut CapeStringOut) -> Result<(), COBIAError> {
        desc.set_string(format!("PersistReader {}", self.path))?;
        Ok(())
    }

    fn set_component_description(&mut self, _desc: &CapeStringIn) -> Result<(), COBIAError> {
        Err(COBIAError::Code(COBIAERR_DENIED))
    }
}

impl cape_open_1_2::ICapePersistReader for PersistReader {
    fn get_value_names(&mut self, value_names: &mut CapeArrayStringOut) -> Result<(), COBIAError> {
        value_names.resize(self.values.len())?;
        for (i, key) in self.values.keys().enumerate() {
            value_names.set_string(i, key)?;
        }
        Ok(())
    }

    fn get_value_type(
        &mut self,
        value_name: &CapeStringIn,
    ) -> Result<cape_open_1_2::CapePersistedDataType, COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(value) => {
                let data_type = match value {
                    PersistedValue::Real(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedReal
                    }
                    PersistedValue::Integer(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedInteger
                    }
                    PersistedValue::Boolean(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedBoolean
                    }
                    PersistedValue::String(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedString
                    }
                    PersistedValue::Enumeration(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedEnumeration
                    }
                    PersistedValue::Value(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedValue
                    }
                    PersistedValue::ArrayReal(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedArrayReal
                    }
                    PersistedValue::ArrayInteger(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedArrayInteger
                    }
                    PersistedValue::ArrayBoolean(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedArrayBoolean
                    }
                    PersistedValue::ArrayString(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedArrayString
                    }
                    PersistedValue::ArrayEnumeration(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedArrayEnumeration
                    }
                    PersistedValue::ArrayValue(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedArrayValue
                    }
                    PersistedValue::ArrayByte(_) => {
                        cape_open_1_2::CapePersistedDataType::CapePersistedArrayByte
                    }
                };
                Ok(data_type)
            }
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" found",
                value_name
            ))),
        }
    }

    fn get_real(&mut self, value_name: &CapeStringIn) -> Result<CapeReal, COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(value) => match value {
                PersistedValue::Real(v) => Ok(*v),
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not a real",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" found",
                value_name
            ))),
        }
    }

    fn get_integer(&mut self, value_name: &CapeStringIn) -> Result<CapeInteger, COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(value) => match value {
                PersistedValue::Integer(v) => Ok(*v),
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not an integer",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" found",
                value_name
            ))),
        }
    }

    fn get_boolean(&mut self, value_name: &CapeStringIn) -> Result<CapeBoolean, COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(value) => match value {
                PersistedValue::Boolean(v) => Ok(*v as CapeBoolean),
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not a boolean",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" found",
                value_name
            ))),
        }
    }

    fn get_string(
        &mut self,
        value_name: &CapeStringIn,
        string_value: &mut CapeStringOut,
    ) -> Result<(), COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(value) => match value {
                PersistedValue::String(v) => {
                    string_value.set_string(v)?;
                    Ok(())
                }
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not a string",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" found",
                value_name
            ))),
        }
    }

    fn get_enumeration(
        &mut self,
        value_name: &CapeStringIn,
    ) -> Result<cobia::CapeEnumeration, COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(value) => match value {
                PersistedValue::Enumeration(v) => Ok(*v as cobia::CapeEnumeration),
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not an enumeration",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" found",
                value_name
            ))),
        }
    }

    fn get_value(
        &mut self,
        value_name: &CapeStringIn,
        value: &mut CapeValueOut,
    ) -> Result<(), COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(v) => {
                match v {
                    PersistedValue::Value(v) => match v {
                        PersistedCapeValue::Real(v) => value.set_real(*v)?,
                        PersistedCapeValue::Integer(v) => value.set_integer(*v)?,
                        PersistedCapeValue::Boolean(v) => value.set_boolean(*v)?,
                        PersistedCapeValue::String(v) => value.set_string(v)?,
                        PersistedCapeValue::Empty => value.set_empty()?,
                    },
                    _ => {
                        return Err(COBIAError::Message(format!(
                            "Value \"{}\" is not a CapeValue",
                            value_name
                        )));
                    }
                }
                Ok(())
            }
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" not found",
                value_name
            ))),
        }
    }

    fn get_array_real(
        &mut self,
        value_name: &CapeStringIn,
        value: &mut CapeArrayRealOut,
    ) -> Result<(), COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(v) => match v {
                PersistedValue::ArrayReal(vec) => {
                    value.put_array(vec)?;
                    Ok(())
                }
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not an array of real",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" not found",
                value_name
            ))),
        }
    }

    fn get_array_integer(
        &mut self,
        value_name: &CapeStringIn,
        value: &mut CapeArrayIntegerOut,
    ) -> Result<(), COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(v) => match v {
                PersistedValue::ArrayInteger(vec) => {
                    value.put_array(vec)?;
                    Ok(())
                }
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not an array of integer",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" not found",
                value_name
            ))),
        }
    }

    fn get_array_boolean(
        &mut self,
        value_name: &CapeStringIn,
        value: &mut CapeArrayBooleanOut,
    ) -> Result<(), COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(v) => match v {
                PersistedValue::ArrayBoolean(vec) => {
                    value.resize(vec.len())?;
                    for (i, b) in vec.iter().enumerate() {
                        value[i] = if *b { 1 } else { 0 };
                    }
                    Ok(())
                }
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not an array of boolean",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" not found",
                value_name
            ))),
        }
    }

    fn get_array_string(
        &mut self,
        value_name: &CapeStringIn,
        value: &mut CapeArrayStringOut,
    ) -> Result<(), COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(v) => match v {
                PersistedValue::ArrayString(vec) => {
                    value.put_array(vec)?;
                    Ok(())
                }
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not an array of string",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" not found",
                value_name
            ))),
        }
    }

    fn get_array_enumeration(
        &mut self,
        value_name: &CapeStringIn,
        value: &mut CapeArrayEnumerationOut<CapeEnumeration>,
    ) -> Result<(), COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(v) => match v {
                PersistedValue::ArrayEnumeration(vec) => {
                    value.resize(vec.len())?;
                    for (i, e) in vec.iter().enumerate() {
                        value[i] = *e as cobia::CapeEnumeration;
                    }
                    Ok(())
                }
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not an array of enumeration",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" not found",
                value_name
            ))),
        }
    }

    fn get_array_value(
        &mut self,
        value_name: &CapeStringIn,
        value: &mut CapeArrayValueOut,
    ) -> Result<(), COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(v) => match v {
                PersistedValue::ArrayValue(vec) => {
                    value.resize(vec.len())?;
                    for (i, e) in vec.iter().enumerate() {
                        match e {
                            PersistedCapeValue::Real(v) => {
                                value.put_value(i, CapeValueContent::Real(*v))?
                            }
                            PersistedCapeValue::Integer(v) => {
                                value.put_value(i, CapeValueContent::Integer(*v))?
                            }
                            PersistedCapeValue::Boolean(v) => {
                                value.put_value(i, CapeValueContent::Boolean(*v))?
                            }
                            PersistedCapeValue::String(v) => {
                                value.put_value(i, CapeValueContent::String(v.clone()))?
                            }
                            PersistedCapeValue::Empty => {
                                value.put_value(i, CapeValueContent::Empty)?
                            }
                        }
                    }
                    Ok(())
                }
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not an array of CapeValue",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" not found",
                value_name
            ))),
        }
    }

    fn get_array_byte(
        &mut self,
        value_name: &CapeStringIn,
        value: &mut CapeArrayByteOut,
    ) -> Result<(), COBIAError> {
        match self.values.get(&value_name.as_string()) {
            Some(v) => match v {
                PersistedValue::ArrayByte(vec) => {
                    value.put_array(vec)?;
                    Ok(())
                }
                _ => Err(COBIAError::Message(format!(
                    "Value \"{}\" is not an array of byte",
                    value_name
                ))),
            },
            None => Err(COBIAError::Message(format!(
                "Value \"{}\" not found",
                value_name
            ))),
        }
    }

    fn get_node_names(&mut self, node_names: &mut CapeArrayStringOut) -> Result<(), COBIAError> {
        node_names.resize(self.sub_nodes.len())?;
        for (i, key) in self.sub_nodes.keys().enumerate() {
            node_names.set_string(i, key)?;
        }
        Ok(())
    }

    fn get_node(
        &mut self,
        node_name: &CapeStringIn,
    ) -> Result<cape_open_1_2::CapePersistReader, COBIAError> {
        match self.sub_nodes.get(&node_name.as_string()) {
            Some(node) => {
                let reader = cape_open_1_2::CapePersistReader::from_object(node)?;
                Ok(reader)
            }
            None => Err(COBIAError::Message(format!(
                "Node \"{}\" not found",
                node_name.as_string()
            ))),
        }
    }
}

impl PersistReader {
    /// Parses a `f64` from a string, handling the special tokens written by
    /// [`PersistWriter::format_f64`].
    ///
    /// # Parameters
    /// - `s`: Source text, which may be `"INF"`, `"-INF"`, `"NaN"`, or any
    ///   string accepted by `f64::parse`.
    ///
    /// # Returns
    /// - `Ok(f64)` on success.
    /// - `Err(ParseFloatError)` if the string is not parseable.
    fn parse_f64(s: &str) -> Result<f64, std::num::ParseFloatError> {
        match s {
            "INF" => Ok(f64::INFINITY),
            "-INF" => Ok(f64::NEG_INFINITY),
            "NaN" => Ok(f64::NAN),
            _ => s.parse::<f64>(),
        }
    }

    /// Constructs a [`PersistReader`] by consuming the data from a
    /// [`PersistWriter`].
    ///
    /// Ownership of values and sub-nodes is moved from the writer into the
    /// reader, leaving the writer empty. Sub-nodes are converted recursively.
    ///
    /// # Parameters
    /// - `writer`: Mutable reference to the populated writer.
    ///
    /// # Returns
    /// - `Ok(CapeObject)`: A CAPE object exposing ICapePersistReader.
    /// - `Err(e)`: If any recursive sub-node conversion fails.
    #[cfg(not(target_os = "windows"))]
    fn from_writer(writer: &mut PersistWriter) -> Result<CapeObject, Box<dyn std::error::Error>> {
        let node = PersistReader::create(writer.path.clone());
        let reader = unsafe { PersistReader::borrow_mut(&node) };
        std::mem::swap(&mut reader.path, &mut writer.path);
        std::mem::swap(&mut reader.values, &mut writer.values);
        for (node_name, writer_node) in &writer.sub_nodes {
            let reader_node = Self::from_writer(unsafe { PersistWriter::borrow_mut(writer_node) })?;
            reader.sub_nodes.insert(node_name.clone(), reader_node);
        }
        Ok(node)
    }

    /// Save data of a PMC, and return as reader for use in another copy of the PMC (e.g. isolated edit instance).
    ///
    /// # Returns
    /// - `Ok(Some(reader))` when data is successfully saved and a reader is created.
    #[cfg(not(target_os = "windows"))]
    pub fn reader_from_pmc_persistence(
        pmc: &cape_open_1_2::CapeUtilities,
    ) -> Result<Option<CapeObject>, Box<dyn std::error::Error>> {
        match cape_open_1_2::CapePersist::from_object(pmc) {
            Ok(persist) => {
                let mut writer = PersistWriter::root_1_2();
                match persist.save(&mut writer, false as CapeBoolean) {
                    Ok(_) => {
                        let mut writer = unsafe { PersistWriter::borrow_mut(&writer) };
                        match PersistReader::from_writer(&mut writer) {
                            Ok(r) => Ok(Some(r)),
                            Err(e) => {
                                return Err(format!(
                                    "Error creating reader for property package private data: {}",
                                    e
                                )
                                .into());
                            }
                        }
                    }
                    Err(e) => {
                        return Err(
                            format!("Error saving property package private data: {}", e).into()
                        );
                    }
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// Constructs a [`PersistReader`] by parsing an XML sub-tree.
    ///
    /// Reads typed child elements (`Real`, `Integer`, `Boolean`, `String`,
    /// `Enumeration`, `Value`, `Array*`, `Node`) and populates the reader's
    /// value map and sub-node map. Sub-nodes are parsed recursively. The path
    /// is used only for diagnostics and sub-node naming.
    ///
    /// # Parameters
    /// - `xml`: The XMLNode cursor positioned at the container to read.
    /// - `path`: Logical path for this node (e.g. `"/"` or `"/foo/bar"`).
    ///
    /// # Returns
    /// - `Ok(CapeObject)`: A CAPE object exposing ICapePersistReader.
    /// - `Err(e)`: On unknown element names, missing attributes, parse
    ///   errors, or XML read failures.
    pub fn from_xml(
        mut xml: crate::xml_reader_node::XMLNode,
        path: String,
    ) -> Result<CapeObject, Box<dyn std::error::Error>> {
        let base_path = path.clone();
        let node = PersistReader::create(path);
        let reader = unsafe { PersistReader::borrow_mut(&node) };
        loop {
            match xml.next_node() {
                Ok(Some(mut child)) => {
                    enum NodeType {
                        Real,
                        Integer,
                        Boolean,
                        String,
                        Enumeration,
                        Value,
                        ArrayReal,
                        ArrayInteger,
                        ArrayBoolean,
                        ArrayString,
                        ArrayEnumeration,
                        ArrayValue,
                        ArrayByte,
                        Node,
                    }
                    let node_type = match child.get_name() {
                        b"Real" => NodeType::Real,
                        b"Integer" => NodeType::Integer,
                        b"Boolean" => NodeType::Boolean,
                        b"String" => NodeType::String,
                        b"Enumeration" => NodeType::Enumeration,
                        b"Value" => NodeType::Value,
                        b"ArrayReal" => NodeType::ArrayReal,
                        b"ArrayInteger" => NodeType::ArrayInteger,
                        b"ArrayBoolean" => NodeType::ArrayBoolean,
                        b"ArrayString" => NodeType::ArrayString,
                        b"ArrayEnumeration" => NodeType::ArrayEnumeration,
                        b"ArrayValue" => NodeType::ArrayValue,
                        b"ArrayByte" => NodeType::ArrayByte,
                        b"Node" => NodeType::Node,
                        unknown_name => {
                            return Err(format!(
                                "Unknown element name: {}",
                                String::from_utf8_lossy(unknown_name)
                            )
                            .into());
                        }
                    };
                    let mut name = None;
                    let mut value_type = None;
                    for (key, value) in child.get_attributes() {
                        match key.as_ref() {
                            "name" => {
                                name = Some(value.clone());
                            }
                            "value_type" => {
                                value_type = Some(value.clone());
                            }
                            unknown_attr => {
                                return Err(format!(
                                    "Unknown attribute {} for Value value",
                                    unknown_attr
                                )
                                .into());
                            }
                        }
                    }
                    if name.is_none() {
                        return Err(format!(
                            "Missing name attribute for {} value",
                            String::from_utf8_lossy(child.get_name())
                        )
                        .into());
                    }
                    match node_type {
                        NodeType::Real => {
                            let value = Self::parse_f64(&child.get_text()?).map_err(|e| {
                                format!(
                                    "Failed to parse Real value {}: {}",
                                    name.as_ref().unwrap(),
                                    e
                                )
                            })?;
                            reader
                                .values
                                .insert(name.unwrap(), PersistedValue::Real(value));
                        }
                        NodeType::Integer => {
                            let value = child.get_text()?.parse::<i32>().map_err(|e| {
                                format!(
                                    "Failed to parse Integer value {}: {}",
                                    name.as_ref().unwrap(),
                                    e
                                )
                            })?;
                            reader
                                .values
                                .insert(name.unwrap(), PersistedValue::Integer(value));
                        }
                        NodeType::Boolean => {
                            let value = match child.get_text()?.as_str() {
                                "true" => true,
                                "false" => false,
                                _ => {
                                    return Err(format!(
                                        "Invalid text content for Boolean value {}: {}",
                                        name.as_ref().unwrap(),
                                        child.get_text()?
                                    )
                                    .into());
                                }
                            };
                            reader
                                .values
                                .insert(name.unwrap(), PersistedValue::Boolean(value));
                        }
                        NodeType::String => {
                            reader
                                .values
                                .insert(name.unwrap(), PersistedValue::String(child.get_text()?));
                        }
                        NodeType::Enumeration => {
                            let value = child.get_text()?.parse::<u32>().map_err(|e| {
                                format!(
                                    "Failed to parse Enumeration value {}: {}",
                                    name.as_ref().unwrap(),
                                    e
                                )
                            })?;
                            reader
                                .values
                                .insert(name.unwrap(), PersistedValue::Enumeration(value));
                        }
                        NodeType::Value => {
                            let cape_value = match value_type {
                                None => {
                                    return Err(
                                        "Missing value_type attribute for Value value".into()
                                    );
                                }
                                Some(vt) => match vt.as_ref() {
                                    "Real" => {
                                        let value =
                                            Self::parse_f64(&child.get_text()?).map_err(|e| {
                                                format!(
                                                    "Failed to parse real value {}: {}",
                                                    name.as_ref().unwrap(),
                                                    e
                                                )
                                            })?;
                                        PersistedCapeValue::Real(value)
                                    }
                                    "Integer" => {
                                        let value = child.get_text()?;
                                        let value = value.parse::<i32>().map_err(|e| {
                                            format!(
                                                "Failed to parse integer value {}: {}",
                                                name.as_ref().unwrap(),
                                                e
                                            )
                                        })?;
                                        PersistedCapeValue::Integer(value)
                                    }
                                    "Boolean" => {
                                        let value = child.get_text()?;
                                        let value =
                                            match value.as_ref() {
                                                "true" => true,
                                                "false" => false,
                                                _ => return Err(format!(
                                                    "Invalid text content for boolean value {}: {}",
                                                    name.as_ref().unwrap(),
                                                    value
                                                )
                                                .into()),
                                            };
                                        PersistedCapeValue::Boolean(value)
                                    }
                                    "String" => {
                                        let value = child.get_text()?;
                                        PersistedCapeValue::String(value)
                                    }
                                    "Empty" => PersistedCapeValue::Empty,
                                    unknown_type => {
                                        return Err(format!(
                                            "Unknown value_type {} for Value value {}",
                                            unknown_type,
                                            name.as_ref().unwrap()
                                        )
                                        .into());
                                    }
                                },
                            };
                            reader
                                .values
                                .insert(name.unwrap(), PersistedValue::Value(cape_value));
                        }
                        NodeType::ArrayReal => {
                            let mut values = Vec::new();
                            let text = child.get_text()?;
                            for part in text.split(';') {
                                let value = Self::parse_f64(part).map_err(|e| {
                                    format!(
                                        "Failed to parse Real value {}: {}",
                                        name.as_ref().unwrap(),
                                        e
                                    )
                                })?;
                                values.push(value);
                            }
                            reader
                                .values
                                .insert(name.unwrap().clone(), PersistedValue::ArrayReal(values));
                        }
                        NodeType::ArrayInteger => {
                            let mut values = Vec::new();
                            let text = child.get_text()?;
                            for part in text.split(';') {
                                let value = part.parse::<i32>().map_err(|e| {
                                    format!(
                                        "Failed to parse Integer value {}: {}",
                                        name.as_ref().unwrap(),
                                        e
                                    )
                                })?;
                                values.push(value);
                            }
                            reader.values.insert(
                                name.unwrap().clone(),
                                PersistedValue::ArrayInteger(values),
                            );
                        }
                        NodeType::ArrayBoolean => {
                            let mut values = Vec::new();
                            let text = child.get_text()?;
                            for part in text.split(';') {
                                let value = match part {
                                    "true" => true,
                                    "false" => false,
                                    _ => {
                                        return Err(format!(
                                            "Invalid text content for boolean value {}: {}",
                                            name.unwrap(),
                                            part
                                        )
                                        .into());
                                    }
                                };
                                values.push(value);
                            }
                            reader.values.insert(
                                name.unwrap().clone(),
                                PersistedValue::ArrayBoolean(values),
                            );
                        }
                        NodeType::ArrayEnumeration => {
                            let mut values = Vec::new();
                            let text = child.get_text()?;
                            for part in text.split(';') {
                                let value = part.parse::<u32>().map_err(|e| {
                                    format!(
                                        "Failed to parse Enumeration value {}: {}",
                                        name.as_ref().unwrap(),
                                        e
                                    )
                                })?;
                                values.push(value);
                            }
                            reader.values.insert(
                                name.unwrap().clone(),
                                PersistedValue::ArrayEnumeration(values),
                            );
                        }
                        NodeType::ArrayString => {
                            let mut values = Vec::new();
                            loop {
                                match child.next_node() {
                                    Ok(Some(mut el)) => {
                                        if el.get_name() != b"Element" {
                                            return Err(format!(
                                                "Unexpected element {} in ArrayString value",
                                                String::from_utf8_lossy(el.get_name())
                                            )
                                            .into());
                                        }
                                        values.push(el.get_text()?.to_string());
                                    }
                                    Ok(None) => break,
                                    Err(e) => {
                                        return Err(format!(
                                            "Error reading ArrayString value {}: {}",
                                            name.as_ref().unwrap(),
                                            e
                                        )
                                        .into());
                                    }
                                }
                            }
                            reader
                                .values
                                .insert(name.unwrap(), PersistedValue::ArrayString(values));
                        }
                        NodeType::ArrayValue => {
                            let mut values = Vec::new();
                            loop {
                                match child.next_node() {
                                    Ok(Some(mut el)) => {
                                        if el.get_name() != b"Element" {
                                            return Err(format!(
                                                "Unexpected element {} in ArrayValue value",
                                                String::from_utf8_lossy(el.get_name())
                                            )
                                            .into());
                                        }
                                        enum ValueType {
                                            Real,
                                            Integer,
                                            Boolean,
                                            String,
                                            Empty,
                                        }
                                        let mut value_type = None;
                                        for (key, value) in el.get_attributes() {
                                            match key.as_ref() {
                                                "value_type" => {
                                                    value_type = Some(match value.as_ref() {
                                                        "Real" => ValueType::Real,
                                                        "Integer" => ValueType::Integer,
                                                        "Boolean" => ValueType::Boolean,
                                                        "String" => ValueType::String,
                                                        "Empty" => ValueType::Empty,
                                                        unknown_type => return Err(format!("Unknown value_type {} for Element in ArrayValue value {}", unknown_type, name.as_ref().unwrap()).into()),
                                                    });
                                                }
                                                unknown_attr => {
                                                    return Err(format!("Unknown attribute {} for Element in ArrayValue value", unknown_attr).into());
                                                }
                                            };
                                        }
                                        match value_type {
                                            Some(v) => {
                                                match v {
                                                    ValueType::Real => {
                                                        let value=Self::parse_f64(&el.get_text()?).map_err(|e| format!("Failed to parse real value {}: {}", name.as_ref().unwrap(), e))?;
                                                        values.push(PersistedCapeValue::Real(value));
                                                    },
                                                    ValueType::Integer => {
                                                        let value=el.get_text()?;
                                                        let value=value.parse::<i32>().map_err(|e| format!("Failed to parse integer value {}: {}", name.as_ref().unwrap(), e))?;
                                                        values.push(PersistedCapeValue::Integer(value));
                                                    },
                                                    ValueType::Boolean => {
                                                        let value=el.get_text()?;
                                                        let value=match value.as_ref() {
                                                            "true" => true,
                                                            "false" => false,
                                                            _ => return Err(format!("Invalid text content for boolean value {}: {}", name.as_ref().unwrap(), value).into()),
                                                        };
                                                        values.push(PersistedCapeValue::Boolean(value));
                                                    },
                                                    ValueType::String => {
                                                        let value=el.get_text()?;
                                                        values.push(PersistedCapeValue::String(value));
                                                    },
                                                    ValueType::Empty => {
                                                        values.push(PersistedCapeValue::Empty);
                                                    },
                                                }   
                                            }
                                            None => return Err(format!("Missing value_type attribute for Element in ArrayValue value {}", name.as_ref().unwrap()).into()),
                                        };
                                    }
                                    Ok(None) => break,
                                    Err(e) => {
                                        return Err(format!(
                                            "Error reading ArrayValue value {}: {}",
                                            name.as_ref().unwrap(),
                                            e
                                        )
                                        .into());
                                    }
                                }
                            }
                            reader
                                .values
                                .insert(name.unwrap(), PersistedValue::ArrayValue(values));
                        }
                        NodeType::ArrayByte => {
                            let bytes = BASE64_STANDARD.decode(child.get_text()?)?;
                            reader
                                .values
                                .insert(name.unwrap(), PersistedValue::ArrayByte(bytes));
                        }
                        NodeType::Node => {
                            let sub_node = PersistReader::from_xml(
                                child,
                                format!("{}/{}", base_path, name.as_ref().unwrap()),
                            )?;
                            reader.sub_nodes.insert(name.unwrap(), sub_node);
                        }
                    }
                }
                Ok(None) => break,
                Err(e) => return Err(format!("Error reading XML: {}", e).into()),
            }
        }
        Ok(node)
    }
}
