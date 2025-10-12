use std::sync::LazyLock;
use cobia::*;

/// Enumeration of supported phase equilibrium types
pub(crate) enum PhaseEquilibriumType {
	/// Flash at specified temperature and pressure
	TemperaturePressure,
	/// Flash at specified pressure and enthalpy
	PressureEnthalpy,
	/// Flash at specified pressure and entropy
	PressureEntropy,
}

/// String constants needed for the phase equilibrium types
struct PhaseEquilibriumTypeStringConstants {
	/// the string literal 'unspecified'
	unspecified: CapeStringConstNoCase,
	/// the string literal 'overall'
	overall: CapeStringConstNoCase,
	/// the string literal 'temperature'
	temperature: CapeStringConstNoCase,
	/// the string literal 'pressure'
	pressure: CapeStringConstNoCase,
	/// the string literal 'enthalpy'
	enthalpy: CapeStringConstNoCase,
	/// the string literal 'entropy'
	entropy: CapeStringConstNoCase,
}

impl PhaseEquilibriumTypeStringConstants {
	/// Construction of the string constants
	fn new() -> Self {
		Self {
			unspecified: CapeStringConstNoCase::from_string("unspecified"),
			overall: CapeStringConstNoCase::from_string("overall"),
			temperature: CapeStringConstNoCase::from_string("temperature"),
			pressure: CapeStringConstNoCase::from_string("pressure"),
			enthalpy: CapeStringConstNoCase::from_string("enthalpy"),
			entropy: CapeStringConstNoCase::from_string("entropy"),
		}
	}
}

///String constants are constructed once, just-in-time, and are shared between all instances
static STRINGCONSTANTS: LazyLock<PhaseEquilibriumTypeStringConstants> = LazyLock::new(||{PhaseEquilibriumTypeStringConstants::new()});

impl PhaseEquilibriumType {


	/// Check phase equilibrium specifications, and return phase equilibrium type
	///
	/// This function is called from CalcEquilibrium as well as CheckEquilibriumSpec.
	/// 
	/// A phase equilibrium specification is a string of 3 or 4 items. The first item is the 
	/// CAPE-OPEN property name, the second item is the basis (which does not apply to 
	/// overall properties, as the Material Object can convert the basis given the overall
	/// composition, which is an input to the calculation), the third item is the phase
	/// on which the specification is made. The 4th item is optional, and is a compound ID,
	/// to which the specification applies. Typically a compound ID is not specified.
	///
	/// # Arguments
	/// * `specification1` - first equilibrium specification
	/// * `specification2` - second equilibrium specification
	/// * `solution_type` - second equilibrium specification
	///
	/// # Returns
	/// A `PhaseEquilibriumType` if the combination of specifications is supported, or else an error.
	pub(crate) fn new(specification1:&CapeArrayStringIn,specification2:&CapeArrayStringIn,solution_type:&CapeStringIn) -> Result<PhaseEquilibriumType,COBIAError> {
		let mut have_temperature = false;
		let mut have_pressure = false;
		let mut have_enthalpy = false;
		let mut have_entropy = false;
		let string_constants=&STRINGCONSTANTS;
		if !solution_type.is_empty() && string_constants.unspecified!=*solution_type {
			return Err(COBIAError::Message("Unsupported solution type".to_string()));
		}
		for spec in [specification1,specification2] {
			if spec.size()!=3 && spec.size()!=4 {
				return Err(COBIAError::Message("Unexpected number of values in flash specification".to_string()));
			}
			//spec contains property, basis, phase and optionally compound
			let property=spec.at(0).unwrap();
			if string_constants.temperature==property {
				have_temperature=true;
			} else if string_constants.pressure==property {
				have_pressure=true;
			} else if string_constants.enthalpy==property {
				have_enthalpy=true;
			} else if string_constants.entropy==property {
				have_entropy=true;
			} else {
				return Err(COBIAError::Message(format!("Unsupported property '{}'",property)));
			}
			//all properties are overall:
			if string_constants.overall!=spec.at(2).unwrap() {
				return Err(COBIAError::Message(format!("Unsupported phase specification '{}' for flash specification '{}'; only overall is supported",spec.at(2).unwrap(),spec.at(0).unwrap())));
			}
			//compound specs are not useful here
			if spec.size()==4 && !spec.at(3).unwrap().to_string().is_empty() {
				return Err(COBIAError::Message(format!("Unsupported compound specification for flash specification '{}'",spec.at(0).unwrap())));
			}
			//ignore basis - should be empty but is not relevant for overall properties
		}
		if have_temperature&&have_pressure {
			Ok(PhaseEquilibriumType::TemperaturePressure)
		} else if have_pressure&&have_enthalpy {
			Ok(PhaseEquilibriumType::PressureEnthalpy)
		} else if have_pressure&&have_entropy {
			Ok(PhaseEquilibriumType::PressureEntropy)
		} else {
			Err(COBIAError::Message("Unsupported combination of flash specification".to_string()))
		}
	}

}