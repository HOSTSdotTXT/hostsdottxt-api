CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_prewarm";
ALTER SYSTEM SET shared_preload_libraries = 'pg_prewarm';

CREATE TABLE IF NOT EXISTS users (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  email varchar(255) NOT NULL UNIQUE,
  password varchar(255) NOT NULL,
  display_name varchar(255),
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  admin boolean NOT NULL DEFAULT false,
  enabled boolean NOT NULL DEFAULT true,
  totp_secret varchar(255),
  totp_confirmed boolean NOT NULL default false
);

CREATE TABLE IF NOT EXISTS zones (
  id varchar(255) NOT NULL UNIQUE PRIMARY KEY,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  owner_uuid uuid NOT NULL,
  constraint owner_uuid_fk foreign key (owner_uuid) references users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS records (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  zone_id varchar(255) NOT NULL,
  name varchar(255) NOT NULL,
  type varchar(16) NOT NULL,
  content TEXT NOT NULL,
  ttl INTEGER NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  constraint zone_id_fk foreign key (zone_id) references zones (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS api_keys (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  owner_uuid uuid NOT NULL,
  token_hash varchar(255) NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
  last_used TIMESTAMP WITH TIME ZONE,
  constraint owner_uuid_fk foreign key (owner_uuid) references users (id) ON DELETE CASCADE
);

CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.modified_at = now() AT TIME ZONE 'UTC';
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE OR REPLACE TRIGGER update_user_modtime BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE update_modified_column();
CREATE OR REPLACE TRIGGER update_zone_modtime BEFORE UPDATE ON zones FOR EACH ROW EXECUTE PROCEDURE update_modified_column();
CREATE OR REPLACE TRIGGER update_record_modtime BEFORE UPDATE ON records FOR EACH ROW EXECUTE PROCEDURE update_modified_column();
