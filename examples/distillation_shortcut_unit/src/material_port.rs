use crate::shared_unit_data::*;
use crate::distillation_shortcut_unit::DistillationShortcutUnit;
use cobia::*;

/// The MaterialPort class implements a CAPE-OPEN 1.2 material port.
///
/// It connects to a valid CAPE-OPEN 1.2 compliant material object provided by the PME.


#[cape_object_implementation(
		interfaces = {
			cape_open_1_2::ICapeIdentification,
			cape_open_1_2::ICapeUnitPort,
		},
		create_arguments = {
			name,
			description,
			is_inlet,
			shared_unit_data
		}
  )] 
pub struct MaterialPort {
	/// The name of the port
	name: CapeStringImpl,
	/// The description of the port
	description: CapeStringImpl,
	/// The connected material object, if any
	connected_object: Option<cape_open_1_2::CapeThermoMaterial>,
	/// Indicates whether this port is an inlet (true) or an outlet (false)
	is_inlet : bool,
	/// Shared data for the unit, containing unit-specific information
	shared_unit_data: SharedUnitDataRef, 
}

impl MaterialPort {

	/// Get the connected material object, if any.
	///
	/// # Returns:
	/// * The connected material object, or `None` if the port is not connected to any material object.

	pub(crate) fn get_connected_material(&self) -> Option<cape_open_1_2::CapeThermoMaterial> {
		self.connected_object.clone()
	}

	/// Get the name of the port.
	///
	/// # Returns:
	/// * A reference to the name of the port as a `CapeStringImpl`.

	pub(crate) fn get_name(&self) -> &CapeStringImpl {
		&self.name
	}

}

impl std::fmt::Display for MaterialPort {

	/// Format the MaterialPort as a string for display purposes.
	///
	/// The std::fmt::Display interface is used when generating the 
	/// source name of the object that raises an error.
	///
	/// # Arguments:
	/// * `f` - A mutable reference to a `std::fmt::Formatter` where the formatted string will be written.
	///
	/// # Returns:
	/// * A `std::fmt::Result` indicating the success or failure of the formatting operation.

	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Port '{}' of {} unit '{}'",self.name, DistillationShortcutUnit::NAME, self.shared_unit_data.borrow().name)
    }

}

impl cape_open_1_2::ICapeIdentification for MaterialPort {

	/// Get the name of the component.
	///
	/// # Arguments:
	/// * `name` - A mutable reference to a `CapeStringOut` where the name will be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the name is set in `name`.

	fn get_component_name(&mut self,name:&mut CapeStringOut) -> Result<(), COBIAError> {
		name.set(&self.name)?;
		Ok(())
	}

	/// Get the description of the component.
	///
	/// # Arguments:
	/// * `description` - A mutable reference to a `CapeStringOut` where the description will be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the description is set in `description`.

	fn get_component_description(&mut self,description:&mut CapeStringOut) -> Result<(), COBIAError> {
		description.set(&self.description)?;
		Ok(())
	}

	/// Set the name of the component.
	///
	/// This method is not allowed for this port implementation and will return an error.

	fn set_component_name(&mut self, _name: &CapeStringIn) -> Result<(), COBIAError> {
		Err(cobia::COBIAError::Code(cobia::COBIAERR_DENIED))
	}

	/// Set the description of the component.
	///
	/// This method is not allowed for this port implementation and will return an error.

	fn set_component_description(&mut self, _desc: &CapeStringIn) -> Result<(), COBIAError> {
		Err(cobia::COBIAError::Code(cobia::COBIAERR_DENIED))
	}
}

impl cape_open_1_2::ICapeUnitPort for MaterialPort {

	/// Get the port type.
	///
	/// # Returns:
	/// * A `Result` containing the port type, which is always `CapeMaterial` for this implementation.

    fn get_port_type(&mut self) -> Result<cape_open_1_2::CapePortType,COBIAError> {
        Ok(cape_open_1_2::CapePortType::CapeMaterial)
    }

	/// Get the direction of the port.
	///
	/// # Returns:
	/// * A `Result` containing the port direction, which is either `CapeInlet` or `CapeOutlet`.

    fn get_direction(&mut self) -> Result<cape_open_1_2::CapePortDirection,COBIAError> {
        Ok(
			match self.is_inlet {
				true => cape_open_1_2::CapePortDirection::CapeInlet,
				false => cape_open_1_2::CapePortDirection::CapeOutlet,
			}
		)
    }

	/// Get the connected object of the port.
	///
	/// # Returns:
	/// * A `Result` containing the connected `CapeObject`, or an error if the port is not connected.

    fn get_connected_object(&mut self) -> Result<cobia::CapeObject,COBIAError> {
        match self.connected_object {
			Some(ref connected_object) => {
				match cobia::CapeObject::from_object(connected_object) {
					Ok(object) => {
						Ok(object)
					},
					Err(_) => {
						Err(COBIAError::Message("Object does not expose ICapeInterface".into()))
					}
				}
			},
			None => {
				Err(COBIAError::Code(COBIAERR_NOSUCHITEM))
			}
		}
    }

	/// Connect the port to a `CapeObject`.
	///
	/// This port will only accept connections to a valid `CapeThermoMaterial` object.
	///
	/// # Arguments:
	/// * `object_to_connect` - The `CapeObject` to connect to the port.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the port is connected to the material object.

    fn connect(&mut self,object_to_connect:cobia::CapeObject) -> Result<(),COBIAError> {
		let material_object = cape_open_1_2::CapeThermoMaterial::from_object(&object_to_connect);
		match material_object {
			Ok(material_object) => {
					self.connected_object=Some(material_object);
					self.shared_unit_data.borrow_mut().validation_status = cape_open_1_2::CapeValidationStatus::CapeNotValidated;
					Ok(())
				},
			Err(e) => Err(e),
		}
    }

	/// Disconnect the port from its connected object.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure; always succeeds.

    fn disconnect(&mut self) -> Result<(),COBIAError> {
        self.connected_object=None;
		self.shared_unit_data.borrow_mut().validation_status = cape_open_1_2::CapeValidationStatus::CapeNotValidated;
		Ok(())
    }
}
