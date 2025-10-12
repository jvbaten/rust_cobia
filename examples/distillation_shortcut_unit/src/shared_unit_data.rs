use cobia::*;

/// The `SharedUnitData` struct holds common data for the distillation shortcut unit.
///
/// Each distillation shortcut unit contains a single instance of `SharedUnitData` that holds
/// information such as the unit's name and validation status. This data is shared across
/// all components of the unit, including ports, collections, and parameters.

pub(crate) struct SharedUnitData {
	/// The name of the distillation shortcut unit; used by various components for formatting the error source name.
	pub name: CapeStringImpl,
	/// The validation status of the distillation shortcut unit; used by various components to flag the unit as not validated.
	pub validation_status : cobia::cape_open_1_2::CapeValidationStatus,
	// The dirty flag for the unit operation. If the unit is dirty, it need saving
	pub dirty: bool,
}

/// All objects that need access to SharedUnitData are outlived by its owner, which 
/// is the DistillationShortcutUnit, so in principle we can pass a reference to 
/// a `RefCell<SharedUnitData>` to all objects that need it.
///
/// This complicates matters due to the life time specification of those objects. 
/// 
/// Instead in this example we prefer to pass a reference counted RefCell, so that
/// all objects can share the same instance of SharedUnitData, with shared ownership.
pub type SharedUnitDataRef = std::rc::Rc<std::cell::RefCell<SharedUnitData>>;

impl std::default::Default for SharedUnitData {
	/// Creates a new instance of `SharedUnitData` with default values.
	///
	/// Implemented to allow deriving Default on classes that reference this.
	fn default() -> Self {
		Self {
			name: std::default::Default::default(),
			validation_status: cobia::cape_open_1_2::CapeValidationStatus::CapeNotValidated,
			dirty:false,
		}
	}
}
