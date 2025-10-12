use strum::IntoEnumIterator;
use strum_macros::{EnumCount,EnumIter};
use std::sync::LazyLock;
use cobia::{CapeOpenMap,CapeStringConstProvider};

/// Singleton table of supported properties with lookup function
pub(crate) struct PropertyTables {
    /// Lookup table for single phase properties by identifier, case insensive.
	single_phase_property_map : CapeOpenMap<SinglePhaseProperty>,
}

/// Enumeration containing all supported single phase properties
#[derive(EnumIter,EnumCount)]
pub(crate) enum SinglePhaseProperty {
	/// Viscosity, Pa*s
	Viscosity,
	/// Temperature derivative of Viscosity, Pa*s/K
	ViscositydTemperature,
	/// Pressure derivative of Viscosity, Pa*s/Pa
	ViscositydPressure,
	/// Mole fraction derivative of viscosity, Pa*s
	ViscositydMoles,
	/// Mole number derivative of viscosity, Pa*s/mol
	ViscositydMolFraction,
	/// Thermal conductivity, W/m/K
	ThermalConductivity,
	/// Temperature derivative of thermal conductivity, W/m/K^2
	ThermalConductivitydTemperature,
	/// Pressure derivative of thermal conductivity, W/m/K/Pa
	ThermalConductivitydPressure,
	/// Mole fraction derivative of thermal conductivity, W/m/K
	ThermalConductivitydMoles,
	/// Mole number derivative of thermal conductivity, W/m/mol
	ThermalConductivitydMolFraction,
	/// Enthalpy, J/mol
	Enthalpy,
	/// Temperature derivative of enthalpy, J/mol/K
	EnthalpydTemperature,
	/// Pressure derivative of enthalpy, J/mol/Pa
	EnthalpydPressure,
	/// Mole fraction derivative of enthalpy, J/mol
	EnthalpydMoles,
	/// Mole number derivative of enthalpy, J/mol
	EnthalpydMolFraction,
	/// Entropy, J/mol/K
	Entropy,
	/// Temperature derivative of entropy, J/mol/K^2
	EntropydTemperature,
	/// Pressure derivative of entropy, J/mol/K/Pa
	EntropydPressure,
	/// Mole fraction derivative of entropy, J/mol/K
	EntropydMoles,
	/// Mole number derivative of entropy, J/mol/K
	EntropydMolFraction,
	/// Density, mol/m^3
	Density,
	/// Temperature derivative of density, mol/m^3/K
	DensitydTemperature,
	/// Pressure derivative of density, mol/m^3/Pa
	DensitydPressure,
	/// Mole fraction derivative of density, mol/m^3
	DensitydMoles,
	/// Mole number derivative of density, mol/m^3/mol
	DensitydMolFraction,
	/// Volume, m^3/mol
	Volume,
	/// Temperature derivative of volume, m^3/mol/K
	VolumedTemperature,
	/// Pressure derivative of volume, m^3/mol/Pa
	VolumedPressure,
	/// Mole fraction of volume, m^3/mol
	VolumedMoles,
	/// Mole number derivative of volume, m^3/mol
	VolumedMolFraction,
}

impl SinglePhaseProperty {

	/// Returns the name of the property as a string.
	/// 
	/// # Returns
	/// A string slice representing the name of the property.
	pub(crate) fn name(&self) -> &str {
		match self {
			SinglePhaseProperty::Viscosity => "viscosity",
			SinglePhaseProperty::ViscositydTemperature => "viscosity.Dtemperature",
			SinglePhaseProperty::ViscositydPressure => "viscosity.Dpressure",
			SinglePhaseProperty::ViscositydMoles => "viscosity.Dmoles",
			SinglePhaseProperty::ViscositydMolFraction => "viscosity.Dmolfraction",
			SinglePhaseProperty::ThermalConductivity => "thermalConductivity",
			SinglePhaseProperty::ThermalConductivitydTemperature => "thermalConductivity.Dtemperature",
			SinglePhaseProperty::ThermalConductivitydPressure => "thermalConductivity.Dpressure",
			SinglePhaseProperty::ThermalConductivitydMoles => "thermalConductivity.Dmoles",
			SinglePhaseProperty::ThermalConductivitydMolFraction => "thermalConductivity.Dmolfraction",
			SinglePhaseProperty::Enthalpy => "enthalpy",
			SinglePhaseProperty::EnthalpydTemperature => "enthalpy.Dtemperature",
			SinglePhaseProperty::EnthalpydPressure => "enthalpy.Dpressure",
			SinglePhaseProperty::EnthalpydMoles => "enthalpy.Dmoles",
			SinglePhaseProperty::EnthalpydMolFraction => "enthalpy.Dmolfraction",
			SinglePhaseProperty::Entropy=> "entropy",
			SinglePhaseProperty::EntropydTemperature => "entropy.Dtemperature",
			SinglePhaseProperty::EntropydPressure => "entropy.Dpressure",
			SinglePhaseProperty::EntropydMoles => "entropy.Dmoles",
			SinglePhaseProperty::EntropydMolFraction => "entropy.Dmolfraction",
			SinglePhaseProperty::Density=> "density",
			SinglePhaseProperty::DensitydTemperature => "density.Dtemperature",
			SinglePhaseProperty::DensitydPressure => "density.Dpressure",
			SinglePhaseProperty::DensitydMoles => "density.Dmoles",
			SinglePhaseProperty::DensitydMolFraction => "density.Dmolfraction",
			SinglePhaseProperty::Volume=> "volume",
			SinglePhaseProperty::VolumedTemperature => "volume.Dtemperature",
			SinglePhaseProperty::VolumedPressure => "volume.Dpressure",
			SinglePhaseProperty::VolumedMoles => "volume.Dmoles",
			SinglePhaseProperty::VolumedMolFraction => "volume.Dmolfraction",
		}
	}
}

impl PropertyTables {

	/// Construction of the singleton instance of the property tables.
	///
	/// Builds the property table.
	///
	/// # Returns
	/// A new instance of `PropertyTables`.
	fn new() -> PropertyTables {

		let mut single_phase_property_map = CapeOpenMap::new();
		for prop in SinglePhaseProperty::iter() {
			single_phase_property_map.insert(prop.name().into(),prop);
		}
		PropertyTables {
			single_phase_property_map,
		}
	}

	/// Single-phase property lookup by identifier.
	///
	/// # Arguments
	/// * `prop_name` - Property name - can be any class that implements `CapeStringConstProvider`.
	///
	/// # Returns
	/// * `Option<&SinglePhaseProperty>` - A reference to the property if found, otherwise `None`.
	pub fn get_single_phase_property<'a,'b,S:CapeStringConstProvider>(&'a self,prop_name:&'b S) -> Option<&'a SinglePhaseProperty> where 'b:'a {
		self.single_phase_property_map.get(prop_name)
	}

}

/// Global shared instance of the property tables, constructed on first use.
pub(crate) static PROPERTYTABLES: LazyLock<PropertyTables> = LazyLock::new(|
|{PropertyTables::new()});