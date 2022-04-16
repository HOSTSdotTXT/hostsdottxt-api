use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Signup {
    pub email: String,
    pub display_name: Option<String>,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
    pub email: String,
    pub password: String,
    pub totp_code: Option<String>,
}
