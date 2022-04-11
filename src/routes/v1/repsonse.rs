use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorMessage {
    pub error: String,
    pub reason: String,
}
