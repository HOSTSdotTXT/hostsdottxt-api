CREATE extension IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
  email varchar(255) NOT NULL UNIQUE,
  password varchar(255) NOT NULL,
  display_name varchar(255),
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  admin boolean NOT NULL DEFAULT false,
  enabled boolean NOT NULL DEFAULT true,
  totp_secret varchar(255)
);

CREATE TABLE zones (
  id varchar(255) NOT NULL UNIQUE PRIMARY KEY,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  modified_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  owner_uuid uuid NOT NULL,
  constraint owner_uuid_fk foreign key (owner_uuid) references users (id)
);

CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.modified_at = now() AT TIME ZONE 'UTC';
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_user_modtime BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE update_modified_column();
CREATE TRIGGER update_zone_modtime BEFORE UPDATE ON zones FOR EACH ROW EXECUTE PROCEDURE update_modified_column();
