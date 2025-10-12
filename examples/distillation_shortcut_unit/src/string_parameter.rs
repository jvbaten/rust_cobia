use crate::shared_unit_data::*;
use crate::distillation_shortcut_unit::DistillationShortcutUnit;
use cobia::*;

/// The StringParameter class implements a CAPE-OPEN 1.2 string parameter.
///
/// Parameters must implement the ICapeIdentification and ICapeParameter interfaces.
/// In addition, string parameters must implement the ICapeStringParameter interface.
///
/// This implementation can be used for both input and output parameters.
/// Input parameters can be set by the user, while output parameters are set by the unit operation.
///

#[cape_object_implementation(
		interfaces = {
			cape_open_1_2::ICapeIdentification,
			cape_open_1_2::ICapeParameter,
			cape_open_1_2::ICapeStringParameter,
		},
		new_arguments = {
			name,
			description,
			is_input,
			shared_unit_data,
			default_value,
			possible_values,
			exclusive,
		}
  )] 
pub struct StringParameter {
	/// The name of the parameter
	pub(crate) name: CapeStringImpl,
	/// The description of the parameter
	description: CapeStringImpl,
	/// Indicates whether this parameter is an input (true) or an output (false)
	is_input : bool,
	/// Shared data for the unit, containing unit-specific information
	shared_unit_data: SharedUnitDataRef, 
	/// The current value of the parameter
	pub(crate) value : CapeStringImpl,
	/// Default value of the parameter
	default_value : CapeStringImpl,
	/// List of allowed values for the parameter
	possible_values : Option<CapeArrayStringVec>,
	/// Exclusive implies only values in the list are allowed.
	exclusive : bool,
	/// Validation status of the parameter
	validation_status: cape_open_1_2::CapeValidationStatus,
}

impl StringParameter {

	pub fn new(name: CapeStringImpl, description: CapeStringImpl, is_input: bool, shared_unit_data: SharedUnitDataRef, default_value: CapeStringImpl, possible_values: Option<CapeArrayStringVec>, exclusive: bool) -> Self {
		Self {
			cobia_object_data:std::default::Default::default(), //initialization of this generated field is needed; can always be set to Default::default()
			name,
			description,
			is_input,
			shared_unit_data,
			value:default_value.clone(),
			default_value,
			possible_values,
			exclusive,
			validation_status:cape_open_1_2::CapeValidationStatus::CapeNotValidated,
		}
	}

	/// Replace the list of possible values with a new list.
	///
	/// # Arguments:
	/// * `possible_values` - An `Option<CapeArrayStringVec>` containing the new list of possible values.
	/// If `None`, the parameter will not have any possible values.	

	pub fn set_possible_values(&mut self, possible_values: Option<&CapeArrayStringVec>) {
		//check same
		match possible_values {
			Some(new_values) => 
				{match self.possible_values.as_ref() {
						Some(current_values) => if current_values == new_values { return; }, //no change, so return
						None => {} //current is None, continue
					}
					let mut new_options=CapeArrayStringVec::new();
					new_options.set(new_values).unwrap();
					self.possible_values = Some(new_options);
				},
			None => 
				{if self.possible_values.is_none() { return; } //no change, so return
				 self.possible_values=None;
				}
		}
		//invalidate the parameter, as the possible values have changed
		let mut shared=self.shared_unit_data.borrow_mut();
		shared.dirty=true;
		shared.validation_status=cape_open_1_2::CapeValidationStatus::CapeNotValidated;
		self.validation_status=cape_open_1_2::CapeValidationStatus::CapeNotValidated;
	}

	/// Get value by reference
	///
	/// # Returns:
	/// * A reference to the current value of the parameter as a `CapeStringImpl`.
	
	pub fn value(&self) -> &CapeStringImpl {
		&self.value
	}

}

impl std::fmt::Display for StringParameter {

	/// Format the StringParameter as a string for display purposes.
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
        write!(f,"Parameter '{}' of {} unit '{}'",self.name, DistillationShortcutUnit::NAME, self.shared_unit_data.borrow().name)
    }

}

impl cape_open_1_2::ICapeIdentification for StringParameter {

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
	/// This method is not allowed for this parameter implementation and will return an error.

	fn set_component_name(&mut self, _name: &CapeStringIn) -> Result<(), COBIAError> {
		Err(cobia::COBIAError::Code(cobia::COBIAERR_DENIED))
	}

	/// Set the description of the component.
	///
	/// This method is not allowed for this parameter implementation and will return an error.

	fn set_component_description(&mut self, _desc: &CapeStringIn) -> Result<(), COBIAError> {
		Err(cobia::COBIAError::Code(cobia::COBIAERR_DENIED))
	}
}

impl cape_open_1_2::ICapeParameter for StringParameter {

	/// Get the validation status of the parameter.
	///
	/// # Returns:
	/// * A `Result` containing the validation status, which is always `CapeValid` for this implementation.

    fn get_val_status(&mut self) -> Result<cape_open_1_2::CapeValidationStatus,COBIAError> {
		Ok(self.validation_status)
    }

	/// Get the mode of the parameter.
	///
	/// # Returns:
	/// * A `Result` containing the mode of the parameter, which is either `CapeInput` or `CapeOutput` based on the `is_input` field.

    fn get_mode(&mut self) -> Result<cape_open_1_2::CapeParamMode,COBIAError> {
        if self.is_input {
			Ok(cape_open_1_2::CapeParamMode::CapeInput)
		} else {
			Ok(cape_open_1_2::CapeParamMode::CapeOutput)
		}
    }

	/// Get the type of the parameter.
	///
	/// # Returns:
	/// * A `Result` containing the type of the parameter, which is always `CapeParameterString` for this implementation.

    fn get_type(&mut self) -> Result<cape_open_1_2::CapeParamType,COBIAError> {
        Ok(cape_open_1_2::CapeParamType::CapeParameterString)
    }

	/// Validate the parameter.
	///
	/// Call the value validation method to check if the parameter value is valid.
	/// 
	/// # Arguments:
	/// * `message` - A mutable reference to a `CapeStringOut` where any validation error messages will be set.
	///
	/// # Returns:
	/// * A `Result` containing a `CapeBoolean` indicating whether the parameter is valid or not. If the input value is not specified, it returns false and sets an error message.

    fn validate(&mut self,message:&mut CapeStringOut) -> Result<CapeBoolean,COBIAError> {
		match cape_open_1_2::ICapeStringParameter::validate(self,
			&CapeStringInFromProvider::from(&self.value).as_cape_string_in(), //this is how any implementation of CapeStringInProvider can be converted to a CapeStringIn
			message) {
			Ok(valid) => {
				self.validation_status=if valid!=0 {cape_open_1_2::CapeValidationStatus::CapeValid} else {cape_open_1_2::CapeValidationStatus::CapeInvalid};
				Ok(valid)
			},
			Err(e) => Err(e),
		}
	}

	/// Reset the parameter to its default value.
	///
	/// This method sets the value of the parameter back to its default value and marks the unit as dirty.
	/// It also resets the validation status to `CapeNotValidated`.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the value is reset to the default value.

    fn reset(&mut self) -> Result<(),COBIAError> {
		if self.default_value.is_empty() {
			return Err(cobia::COBIAError::Message("Default value is not available".into()));
		}
		if self.value!=self.default_value {
			self.value.set(&self.default_value); 
			let mut shared=self.shared_unit_data.borrow_mut();
			shared.dirty=true;
			shared.validation_status=cape_open_1_2::CapeValidationStatus::CapeNotValidated;
			self.validation_status=cape_open_1_2::CapeValidationStatus::CapeNotValidated;
		}
		Ok(())
	}
}

impl cape_open_1_2::ICapeStringParameter for StringParameter {

	/// Get the current value of the string parameter.
	///
	/// # Arguments:
	/// * `value` - A mutable reference to a `CapeStringOut` where the current value will be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the current value is set in `value`.

    fn get_value(&mut self,value:&mut CapeStringOut) -> Result<(),COBIAError> {
        value.set(&self.value)?;
		Ok(())
    }

	/// Set the value of the string parameter.
	///
	/// The current value cannot be set for an output parameter.
	///
	/// # Arguments:
	/// * `value` - A reference to a `CapeStringIn` containing the new value to be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the value is set and the unit is marked as dirty.

    fn set_value(&mut self,value:&CapeStringIn) -> Result<(),COBIAError> {
        if !self.is_input {
			return Err(cobia::COBIAError::Code(cobia::COBIAERR_DENIED));
		}
		if *value!=self.value {
			self.value.set(value); //set the value, which is a CapeStringImpl
			let mut shared=self.shared_unit_data.borrow_mut();
			shared.dirty=true;
			shared.validation_status=cape_open_1_2::CapeValidationStatus::CapeNotValidated;
			self.validation_status=cape_open_1_2::CapeValidationStatus::CapeNotValidated;
		}
		Ok(())
    }

	/// Get the default value of the string parameter.
	///
	/// # Arguments:
	/// * `default_value` - A mutable reference to a `CapeStringOut` where the default value will be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the default value is set in `default_value`.

    fn get_default_value(&mut self,default_value:&mut CapeStringOut) -> Result<(),COBIAError> {
        if self.default_value.is_empty() {
			return Err(cobia::COBIAError::Message("Default value is not available".into()));
		}
		default_value.set(&self.default_value)?; //set the default value
		Ok(())
    }

	/// Get the list of possible values for the string parameter.
	///
	/// # Arguments:
	/// * `option_names` - A mutable reference to a `CapeArrayStringOut` where the list of possible values will be set.
	///
	/// # Returns:
	/// * A `Result` indicating success or failure. If successful, the list of possible values is set in `option_names`.

    fn get_option_list(&mut self,option_names:&mut CapeArrayStringOut) -> Result<(),COBIAError> {
        match self.possible_values {
			Some(ref values) => {
				option_names.set(values)?; //set the possible values
				Ok(())
			},
			None => Err(cobia::COBIAError::Message("No options available".into())), //no possible values available
		}
    }

	/// Check whether the value is restricted to the list of possible values.
	///
	/// # Returns:
	/// * A `Result` containing a `CapeBoolean` indicating whether the value is restricted to the list of possible values.

    fn get_restricted_to_list(&mut self) -> Result<CapeBoolean,COBIAError> {
        Ok(self.exclusive as CapeBoolean)
    }

	/// Validate the value of the string parameter.
	///
	/// This method checks whether the specified value is suitable for the parameter.
	///
	/// # Arguments:
	/// * `value` - A reference to a `CapeStringIn` containing the value to be validated.
	/// * `message` - A mutable reference to a `CapeStringOut` where any validation error messages will be set.
	///
	/// # Returns:
	/// * A `Result` containing a `CapeBoolean` indicating whether the value is valid or not. If the input value is not specified, it returns false and sets an error message.

    fn validate(&mut self,value:&CapeStringIn,message:&mut CapeStringOut) -> Result<CapeBoolean,COBIAError> {
        if self.is_input && self.value.is_empty() {
			message.set_string("Value must be specified")?;
			Ok(false as CapeBoolean)
		} else if self.exclusive {
			let mut in_list=false;
			if let Some(possible_values) = &self.possible_values {
				for value in possible_values.iter() {
					if *value == self.value {
						in_list = true;
						break;
					}
				}
			}
			if !in_list {
				message.set_string(format!("Value \"{}\" is not in the list of allowed values",value))?;
			}
			Ok(in_list as CapeBoolean)
		} else {
			Ok(true as CapeBoolean)
		}
    }
}