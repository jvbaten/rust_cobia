use serde::Serialize;

#[derive(Serialize, Clone)]
/// Frontend-facing status payload for unit operation selection.
pub(crate) struct UnitStatus {
    /// Name of the currently selected unit operation, if any.
    pub active_unit: Option<String>,
    /// Current unit-related error message, if any.
    pub error: Option<String>,
    /// Name of the currently selected CAPE-OPEN unit provider, if any.
    pub provider: Option<String>,
    /// Available provider names for UI selection.
    pub provider_list: Option<Vec<String>>,
    /// Provider UUIDs aligned with [`UnitStatus::provider_list`].
    pub provider_uuid_list: Option<Vec<String>>,
}

impl Default for UnitStatus {
    /// Creates an empty unit status with no selected unit/provider and no
    /// discovered provider list.
    ///
    /// # Returns
    /// A [`UnitStatus`] with all fields initialised to `None`.
    fn default() -> Self {
        Self {
            active_unit: None,
            error: None,
            provider: None,
            provider_list: None,
            provider_uuid_list: None,
        }
    }
}
