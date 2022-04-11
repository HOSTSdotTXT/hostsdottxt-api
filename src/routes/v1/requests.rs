use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Signup {
    pub email: String,
    pub display_name: Option<String>,
    pub password: String,
}
