use serde::Serialize;

/// Frontend-facing status payload for thermodynamic package selection.
#[derive(Serialize, Clone)]
pub(crate) struct ThermoStatus {
    /// Name of the currently selected thermodynamic package, if available.
    pub active_package: Option<String>,
    /// Current thermo-related error message, if any.
    pub error: Option<String>,
    /// Name of the currently selected CAPE-OPEN provider, if available.
    pub provider: Option<String>,
    /// Available provider names for UI selection.
    pub provider_list: Option<Vec<String>>,
    /// Provider UUIDs aligned with [`ThermoStatus::provider_list`].
    pub provider_uuid_list: Option<Vec<String>>,
    /// Available package names for the selected provider.
    pub package_list: Option<Vec<String>>,
}

impl Default for ThermoStatus {
    /// Creates an empty thermo status with no selected provider/package and no
    /// discovered provider/package lists.
    ///
    /// # Returns
    /// A [`ThermoStatus`] with all fields initialised to `None`.
    fn default() -> Self {
        Self {
            active_package: None,
            error: None,
            provider: None,
            provider_list: None,
            provider_uuid_list: None,
            package_list: None,
        }
    }
}
