CREATE TABLE IF NOT EXISTS user_settings
(
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    search_interval_minutes integer NOT NULL DEFAULT 60 CHECK (search_interval_minutes >= 30),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION ensure_user_settings_exists()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO user_settings (user_id)
    VALUES (NEW.id)
    ON CONFLICT (user_id) DO NOTHING;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS users_create_default_settings_trigger ON users;

CREATE TRIGGER users_create_default_settings_trigger
AFTER INSERT ON users
FOR EACH ROW
EXECUTE FUNCTION ensure_user_settings_exists();

INSERT INTO user_settings (user_id)
SELECT id
FROM users
ON CONFLICT (user_id) DO NOTHING;
