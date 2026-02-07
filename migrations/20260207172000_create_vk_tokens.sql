CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS vk_tokens
(
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash text NOT NULL,
    token_encrypted bytea NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (user_id, token_hash)
);

CREATE INDEX IF NOT EXISTS vk_tokens_user_id_created_at_idx
    ON vk_tokens(user_id, created_at DESC);
