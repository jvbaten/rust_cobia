use std::hash::Hash;

use crate::xml_reader_node;

/// Identifies which kind of compound specification is active on a stream.
///
/// Only one family can be active at a time; mixed families are rejected during
/// `apply_specifications`.
#[derive(Clone, PartialEq, Eq)]
#[derive(Default)]
pub(crate) enum CompoundSpecUsed {
    /// No compound specification has been entered yet.
    #[default]
    None,
    /// Composition is specified as mole fractions.
    MoleFraction,
    /// Composition is specified as mass fractions.
    MassFraction,
    /// Composition and total flow are specified as molar per-compound flow rates.
    MolarCompoundFlow,
    /// Composition and total flow are specified as mass per-compound flow rates.
    MassCompoundFlow,
}


#[derive(Clone, PartialEq, Eq)]
/// Identifies which kind of total-flow specification is active on a stream.
///
/// Only one family can be active at a time; mixed families are rejected during
/// `apply_specifications`.
#[derive(Default)]
pub(crate) enum FlowRateSpecUsed {
    /// No flow-rate specification has been entered yet.
    #[default]
    None,
    /// Total flow is specified as a molar flow rate.
    MolarFlow,
    /// Total flow is specified as a mass flow rate.
    MassFlow,
    /// Flow is implicitly given by per-compound molar flow rates.
    MolarCompoundFlow,
    /// Flow is implicitly given by per-compound mass flow rates.
    MassCompoundFlow,
}


#[derive(Clone)]
/// The kind of one stream specification entry.
///
/// Scalar variants are unit-less discriminants. Compound-specific variants
/// carry the compound ID string. `PartialEq`, `Eq`, and `Hash` treat
/// compound-carrying variants as equal only when their compound IDs also match;
/// all others compare by discriminant.
pub(crate) enum StreamSpecificationType {
    /// A named parameter value on an information/energy stream. The inner string is the parameter name.
    ParameterValue(String),
    /// Temperature \[K\].
    Temperature,
    /// Pressure \[Pa\].
    Pressure,
    /// Molar enthalpy \[J/mol\].
    MolarEnthalpy,
    /// Molar entropy \[J/(mol·K)\].
    MolarEntropy,
    /// Molar volume \[m³/mol\].
    MolarVolume,
    /// Specific (mass) enthalpy \[J/kg\].
    SpecificEnthalpy,
    /// Specific (mass) entropy \[J/(kg·K)\].
    SpecificEntropy,
    /// Specific (mass) volume \[m³/kg\].
    SpecificVolume,
    /// Molar density \[mol/m³\].
    MolarDensity,
    /// Mass density \[kg/m³\].
    MassDensity,
    /// Vapour mole fraction \[-\].
    MolarVaporFraction,
    /// Vapour mass fraction \[-\].
    MassVaporFraction,
    /// Mole fraction of one compound. Inner string is the compound ID.
    MolarFraction(String),
    /// Mass fraction of one compound. Inner string is the compound ID.
    MassFraction(String),
    /// Total mass flow rate \[kg/s\].
    MassFlowRate,
    /// Total molar flow rate \[mol/s\].
    MolarFlowRate,
    /// Mass flow rate of one compound \[kg/s\]. Inner string is the compound ID.
    MassCompoundFlowRate(String),
    /// Molar flow rate of one compound \[mol/s\]. Inner string is the compound ID.
    MolarCompoundFlowRate(String),
}

impl PartialEq for StreamSpecificationType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                StreamSpecificationType::ParameterValue(name1),
                StreamSpecificationType::ParameterValue(name2),
            ) => name1 == name2,
            (
                StreamSpecificationType::MolarFraction(compound_id1),
                StreamSpecificationType::MolarFraction(compound_id2),
            ) => compound_id1 == compound_id2,
            (
                StreamSpecificationType::MassFraction(compound_id1),
                StreamSpecificationType::MassFraction(compound_id2),
            ) => compound_id1 == compound_id2,
            (
                StreamSpecificationType::MassCompoundFlowRate(compound_id1),
                StreamSpecificationType::MassCompoundFlowRate(compound_id2),
            ) => compound_id1 == compound_id2,
            (
                StreamSpecificationType::MolarCompoundFlowRate(compound_id1),
                StreamSpecificationType::MolarCompoundFlowRate(compound_id2),
            ) => compound_id1 == compound_id2,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

impl Eq for StreamSpecificationType {}

impl Hash for StreamSpecificationType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            StreamSpecificationType::ParameterValue(name) => {
                1.hash(state);
                name.hash(state);
            }
            StreamSpecificationType::Temperature => 2.hash(state),
            StreamSpecificationType::Pressure => 3.hash(state),
            StreamSpecificationType::MolarEnthalpy => 4.hash(state),
            StreamSpecificationType::MolarEntropy => 5.hash(state),
            StreamSpecificationType::MolarVolume => 6.hash(state),
            StreamSpecificationType::SpecificEnthalpy => 7.hash(state),
            StreamSpecificationType::SpecificEntropy => 8.hash(state),
            StreamSpecificationType::SpecificVolume => 9.hash(state),
            StreamSpecificationType::MolarDensity => 10.hash(state),
            StreamSpecificationType::MassDensity => 11.hash(state),
            StreamSpecificationType::MolarVaporFraction => 12.hash(state),
            StreamSpecificationType::MassVaporFraction => 13.hash(state),
            StreamSpecificationType::MolarFraction(compound_id) => {
                14.hash(state);
                compound_id.hash(state);
            }
            StreamSpecificationType::MassFraction(compound_id) => {
                15.hash(state);
                compound_id.hash(state);
            }
            StreamSpecificationType::MassFlowRate => 16.hash(state),
            StreamSpecificationType::MolarFlowRate => 17.hash(state),
            StreamSpecificationType::MassCompoundFlowRate(compound_id) => {
                18.hash(state);
                compound_id.hash(state);
            }
            StreamSpecificationType::MolarCompoundFlowRate(compound_id) => {
                19.hash(state);
                compound_id.hash(state);
            }
        }
    }
}

/// A single user-entered specification for a stream.
///
/// The combination of `specification_type` and `value` fully describes one
/// constraint. Multiple [`StreamSpecification`] entries together drive flash
/// calculations and composition/flow resolution in
/// `apply_specifications`.
pub(crate) struct StreamSpecification {
    /// Discriminant identifying what physical quantity is being specified.
    pub specification_type: StreamSpecificationType,
    /// Numeric value in SI base units corresponding to `specification_type`.
    pub value: f64,
}

impl StreamSpecification {
    /// Serialises this specification as a `<Specification>` XML element.
    ///
    /// Compound-carrying and parameter-carrying variants emit additional
    /// attributes (`compound` / `name`). The numeric value is written as the
    /// element text content.
    ///
    /// # Parameters
    /// - `writer`: A `quick_xml` writer to write into.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(e)` if any XML write fails.
    pub fn write_xml<T>(
        &self,
        writer: &mut quick_xml::writer::Writer<T>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        T: std::io::Write,
    {
        let mut writer = writer.create_element("Specification");
        match &self.specification_type {
            StreamSpecificationType::ParameterValue(name) => {
                writer = writer
                    .with_attribute(("type", "ParameterValue"))
                    .with_attribute(("name", name.as_str()));
            }
            StreamSpecificationType::Temperature => {
                writer = writer.with_attribute(("type", "Temperature"));
            }
            StreamSpecificationType::Pressure => {
                writer = writer.with_attribute(("type", "Pressure"));
            }
            StreamSpecificationType::MolarEnthalpy => {
                writer = writer.with_attribute(("type", "MolarEnthalpy"));
            }
            StreamSpecificationType::MolarEntropy => {
                writer = writer.with_attribute(("type", "MolarEntropy"));
            }
            StreamSpecificationType::MolarVolume => {
                writer = writer.with_attribute(("type", "MolarVolume"));
            }
            StreamSpecificationType::SpecificEnthalpy => {
                writer = writer.with_attribute(("type", "SpecificEnthalpy"));
            }
            StreamSpecificationType::SpecificEntropy => {
                writer = writer.with_attribute(("type", "SpecificEntropy"));
            }
            StreamSpecificationType::SpecificVolume => {
                writer = writer.with_attribute(("type", "SpecificVolume"));
            }
            StreamSpecificationType::MolarDensity => {
                writer = writer.with_attribute(("type", "MolarDensity"));
            }
            StreamSpecificationType::MassDensity => {
                writer = writer.with_attribute(("type", "MassDensity"));
            }
            StreamSpecificationType::MolarVaporFraction => {
                writer = writer.with_attribute(("type", "MolarVaporFraction"));
            }
            StreamSpecificationType::MassVaporFraction => {
                writer = writer.with_attribute(("type", "MassVaporFraction"));
            }
            StreamSpecificationType::MolarFraction(compound) => {
                writer = writer
                    .with_attribute(("type", "MolarFraction"))
                    .with_attribute(("compound", compound.as_str()));
            }
            StreamSpecificationType::MassFraction(compound) => {
                writer = writer
                    .with_attribute(("type", "MassFraction"))
                    .with_attribute(("compound", compound.as_str()));
            }
            StreamSpecificationType::MassFlowRate => {
                writer = writer.with_attribute(("type", "MassFlowRate"));
            }
            StreamSpecificationType::MolarFlowRate => {
                writer = writer.with_attribute(("type", "MolarFlowRate"));
            }
            StreamSpecificationType::MassCompoundFlowRate(compound) => {
                writer = writer
                    .with_attribute(("type", "MassCompoundFlowRate"))
                    .with_attribute(("compound", compound.as_str()));
            }
            StreamSpecificationType::MolarCompoundFlowRate(compound) => {
                writer = writer
                    .with_attribute(("type", "MolarCompoundFlowRate"))
                    .with_attribute(("compound", compound.as_str()));
            }
        }
        writer.write_text_content(quick_xml::events::BytesText::new(
            self.value.to_string().as_str(),
        ))?;
        Ok(())
    }

    /// Deserialises a [`StreamSpecification`] from a `<Specification>` XML node.
    ///
    /// Reads the `type` attribute to determine the variant, and the optional
    /// `compound` / `name` attributes for variants that carry an ID. The
    /// element text content is parsed as the numeric `value`.
    ///
    /// # Parameters
    /// - `xml`: An `XMLNode` cursor positioned at the `<Specification>` element.
    ///
    /// # Returns
    /// - `Ok(StreamSpecification)`: Successfully parsed specification.
    /// - `Err(e)`: On unexpected attributes, missing required attributes,
    ///   unknown type strings, or value parse errors.
    pub fn read_xml(mut xml: xml_reader_node::XMLNode) -> Result<Self, Box<dyn std::error::Error>> {
        let mut spec_type = None;
        let mut compound = None;
        let mut name = None;
        for (key, value) in xml.get_attributes() {
            match key.as_str() {
                "type" => {
                    spec_type = Some(value.clone());
                }
                "compound" => {
                    compound = Some(value.clone());
                }
                "name" => {
                    name = Some(value.clone());
                }
                _ => {
                    return Err(
                        format!("Unexpected attribute {} in StreamSpecification XML", key).into(),
                    );
                }
            }
        }
        let value = xml
            .get_text()?
            .parse::<f64>()
            .map_err(|e| format!("Error parsing StreamSpecification value: {}", e))?;
        match spec_type {
            None => Err("Missing type attribute in StreamSpecification XML".into()),
            Some(t) => {
                match t.as_ref() {
                    "ParameterValue" => {
                        let name = name.ok_or(
                            "Missing name attribute for ParameterValue StreamSpecification in XML",
                        )?;
                        Ok(StreamSpecification {
                            specification_type: StreamSpecificationType::ParameterValue(name),
                            value,
                        })
                    }
                    "Temperature" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::Temperature,
                        value,
                    }),
                    "Pressure" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::Pressure,
                        value,
                    }),
                    "MolarEnthalpy" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::MolarEnthalpy,
                        value,
                    }),
                    "MolarEntropy" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::MolarEntropy,
                        value,
                    }),
                    "MolarVolume" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::MolarVolume,
                        value,
                    }),
                    "SpecificEnthalpy" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::SpecificEnthalpy,
                        value,
                    }),
                    "SpecificEntropy" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::SpecificEntropy,
                        value,
                    }),
                    "SpecificVolume" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::SpecificVolume,
                        value,
                    }),
                    "MolarDensity" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::MolarDensity,
                        value,
                    }),
                    "MassDensity" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::MassDensity,
                        value,
                    }),
                    "MolarVaporFraction" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::MolarVaporFraction,
                        value,
                    }),
                    "MassVaporFraction" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::MassVaporFraction,
                        value,
                    }),
                    "MolarFraction" => {
                        let compound=compound.ok_or("Missing compound attribute for MolarFraction StreamSpecification in XML")?;
                        Ok(StreamSpecification {
                            specification_type: StreamSpecificationType::MolarFraction(compound),
                            value,
                        })
                    }
                    "MassFraction" => {
                        let compound=compound.ok_or("Missing compound attribute for MassFraction StreamSpecification in XML")?;
                        Ok(StreamSpecification {
                            specification_type: StreamSpecificationType::MassFraction(compound),
                            value,
                        })
                    }
                    "MassFlowRate" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::MassFlowRate,
                        value,
                    }),
                    "MolarFlowRate" => Ok(StreamSpecification {
                        specification_type: StreamSpecificationType::MolarFlowRate,
                        value,
                    }),
                    "MassCompoundFlowRate" => {
                        let compound=compound.ok_or("Missing compound attribute for MassCompoundFlowRate StreamSpecification in XML")?;
                        Ok(StreamSpecification {
                            specification_type: StreamSpecificationType::MassCompoundFlowRate(
                                compound,
                            ),
                            value,
                        })
                    }
                    unknown => {
                        Err(format!("Unknown StreamSpecification type {} in XML", unknown).into())
                    }
                }
            }
        }
    }
}
