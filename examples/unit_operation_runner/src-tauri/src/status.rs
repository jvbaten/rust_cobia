use serde::Serialize;

/// Status of the unit operation runner, to be sent to the frontend to update the status bar
#[derive(Serialize, Clone)]
#[serde(tag = "status", content = "message")]
pub(crate) enum Status {
    /// The runner is idle and ready to execute a simulation.
    Ready,
    /// The runner has successfully completed a simulation.
    Solved,
    /// The runner encountered an error during simulation. The message contains a human-readable description of the error.
    Error(String),
    /// The runner is expecting user input. The message contains a human-readable description of the required input.
    Incomplete(String),
}
