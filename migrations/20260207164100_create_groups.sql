CREATE TABLE IF NOT EXISTS groups
(
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    group_id bigint NOT NULL,
    group_name varchar(128),
    screen_name text,
    is_closed integer,
    public_type varchar(128),
    photo_200 text,
    description text,
    members_count integer,
    PRIMARY KEY (user_id, group_id)
);

CREATE INDEX IF NOT EXISTS groups_user_id_idx ON groups(user_id);
