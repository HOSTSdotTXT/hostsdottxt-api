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
        UPDATE records SET name = $1, type = $2, content = $3, ttl = $4 WHERE id = $5 AND zone_id = $6 RETURNING *
    ";
    pub(crate) static ref DELETE_RECORD: &'static str = r"
        DELETE FROM records WHERE id = $1 AND zone_id = $2
    ";
    pub(crate) static ref GET_RECORDS: &'static str = r"
        SELECT id,zone_id,name,type,content,ttl,created_at,modified_at
            FROM records WHERE zone_id = $1
    ";
    pub(crate) static ref GET_USER_FROM_API_KEY: &'static str = r"
        SELECT users.id,email,password,display_name,users.created_at,modified_at,admin,enabled,totp_secret FROM api_keys
            JOIN users
                ON users.id = api_keys.owner_uuid
            WHERE api_keys.token_hash = $1 
                AND api_keys.expires_at > (now() AT TIME ZONE 'UTC');
    ";
    pub(crate) static ref GET_METRICS: &'static str = r#"
        SELECT
            percentile_cont(0.50) WITHIN GROUP (ORDER BY queries.duration_us) AS p50,
            percentile_cont(0.90) WITHIN GROUP (ORDER BY queries.duration_us) AS p90,
            percentile_cont(0.95) WITHIN GROUP (ORDER BY queries.duration_us) AS p95,
            percentile_cont(0.99) WITHIN GROUP (ORDER BY queries.duration_us) AS p99,
            avg(queries.duration_us)::FLOAT8 AS avg,
            count(queries.duration_us) as count
        FROM queries WHERE "time" > (NOW() AT TIME ZONE 'UTC' - '1 day'::INTERVAL);
    "#;
}
