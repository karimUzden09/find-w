-- Add migration script here
CREATE TABLE refresh_tokens (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    token_hash text NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,

    created_at timestamptz NOT NULL DEFAULT now(),
    revoked_at timestamptz NULL,
    replaced_by uuid NULL REFERENCES refresh_tokens(id)
);

CREATE INDEX refresh_tokens_user_id_idx ON refresh_tokens(user_id);
CREATE INDEX refresh_tokens_expires_at_idx ON refresh_tokens(expires_at);