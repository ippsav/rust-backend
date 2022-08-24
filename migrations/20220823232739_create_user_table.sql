CREATE TABLE users (
  id uuid, 
  PRIMARY KEY(id),
  username varchar(25) UNIQUE NOT NULL,
  email varchar(35) UNIQUE NOT NULL,
  password_hash varchar(40) NOT NULL,
  created_at timestamptz NOT NULL default now(),
  updated_at timestamptz NOT NULL default now()
);
