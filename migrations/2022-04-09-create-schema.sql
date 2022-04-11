CREATE TABLE users (
  email varchar(255) NOT NULL UNIQUE PRIMARY KEY,
  password varchar(255) NOT NULL,
  display_name varchar(255),
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (now() AT TIME ZONE 'UTC'),
  admin boolean NOT NULL DEFAULT false,
  enabled boolean NOT NULL DEFAULT true,
  totp_secret varchar(255)
);
