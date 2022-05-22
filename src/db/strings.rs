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
    pub(crate) static ref CREATE_ZONE: &'static str = r"
        INSERT INTO zones(id,owner_uuid) VALUES ($1, $2) RETURNING *
    ";
    pub(crate) static ref GET_ZONES: &'static str = r"
        SELECT id,owner_uuid,created_at,modified_at
            FROM zones WHERE owner_uuid = $1
    ";
    pub(crate) static ref GET_ZONE: &'static str = r"
        SELECT id,owner_uuid,created_at,modified_at
            FROM zones WHERE id = $1
    ";
    pub(crate) static ref CREATE_RECORD: &'static str = r"
        INSERT INTO records(zone_id,name,type,content,ttl) VALUES ($1, $2, $3, $4, $5) RETURNING *
    ";
    pub(crate) static ref UPDATE_RECORD: &'static str = r"
        UPDATE records SET name = $1, type = $2, content = $3, ttl = $4 WHERE id = $5 RETURNING *
    ";
    pub(crate) static ref GET_RECORDS: &'static str = r"
        SELECT id,zone_id,name,type,content,ttl,created_at,modified_at
            FROM records WHERE zone_id = $1
    ";
}
