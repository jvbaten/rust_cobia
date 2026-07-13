use bitflags::bitflags;

bitflags! {

    /// Bitmask of pending UI update categories.
    ///
    /// Each flag corresponds to a distinct frontend event emitted by
    /// `send_status` in `lib.rs`. Flags are accumulated on the application
    /// state and cleared in one shot when the frontend is notified, so that
    /// multiple rapid backend changes result in a single batch of events.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct StatusNotification: u32 {
        /// No pending notifications.
        const None = 0;
        /// The thermodynamic package selection or configuration changed (`update-thermo` event).
        const ThermoChanged = 1;
        /// The unit operation selection or configuration changed (`update-unit` event).
        const UnitChanged = 2;
        /// One or more stream values changed (`update-streams` event).
        const StreamsChanged = 4;
        /// One or more unit-operation parameters changed (`update-parameters` event).
        const ParametersChanged = 8;
        /// The set of available reports changed (`update-reports` event).
        const ReportsChanged = 16;
        /// The overall runner status changed (`update-status` event).
        const StatusChanged = 32;
        /// The information-stream table changed (`update-info` event).
        const InfoChanged = 64;
        /// The backend log changed (`update-log` event).
        const LogChanged = 128;
    }
}
