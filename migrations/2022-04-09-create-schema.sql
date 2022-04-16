CREATE TABLE users (
  email varchar(255) NOT NULL UNIQUE PRIMARY KEY,
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
  owner_email varchar(255) NOT NULL,
  CONSTRAINT owner_email_fk FOREIGN KEY (owner_email) REFERENCES users (email) ON DELETE CASCADE
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
