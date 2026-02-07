-- Add migration script here
CREATE EXTENSION IF NOT EXISTS citext;

ALTER TABLE users
    ALTER COLUMN email TYPE citext;