use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref SIGNUPS_ENABLED: bool =
        env::var("SIGNUPS_ENABLED").unwrap_or_else(|_| String::from("false")) == "true";
    pub static ref TOTP_ENABLED: bool =
        env::var("TOTP_ENABLED").unwrap_or_else(|_| String::from("false")) == "true";
}
