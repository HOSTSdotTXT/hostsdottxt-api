use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref GET_USER: &'static str = r"
        SELECT id,email,password,display_name,created_at,modified_at,admin,enabled,totp_secret
            FROM users
            WHERE email = $1
    ";
    pub(crate) static ref GET_ALL_USERS: &'static str = r"
        SELECT id,email,password,display_name,created_at,modified_at,admin,enabled,totp_secret
            FROM users
    ";
    pub(crate) static ref CREATE_USER: &'static str = r"
        INSERT INTO users(email,password,display_name) VALUES ($1, $2, $3) RETURNING *
    ";
}
