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

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub content: String,
    pub ttl: u32,
}
