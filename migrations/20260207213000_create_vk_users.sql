CREATE TABLE IF NOT EXISTS vk_users
(
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    vk_user_id bigint NOT NULL,
    sex smallint,
    first_name varchar(128),
    last_name varchar(128),
    city varchar(128),
    finded_date timestamptz NOT NULL,
    is_closed boolean,
    screen_name varchar(128),
    can_access_closed boolean,
    about text,
    status text,
    bdate varchar(20),
    photo text,
    PRIMARY KEY (user_id, vk_user_id)
);

CREATE INDEX IF NOT EXISTS vk_users_user_id_finded_date_idx
    ON vk_users(user_id, finded_date DESC);
