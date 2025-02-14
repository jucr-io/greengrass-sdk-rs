use serde::{Deserialize, Serialize};

/// The lifecycle state of a component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    /// The component is in the `RUNNING` state.
    #[serde(rename = "RUNNING")]
    Running,
    /// The component is in the `ERRORED` state.
    #[serde(rename = "ERRORED")]
    Errored,
}
