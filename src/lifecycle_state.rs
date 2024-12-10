use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    #[serde(rename = "RUNNING")]
    Running,
    #[serde(rename = "ERRORED")]
    Errored,
}
