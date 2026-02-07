CREATE TABLE IF NOT EXISTS vk_posts
(
    user_id uuid NOT NULL,
    post_id bigint NOT NULL,
    group_id bigint NOT NULL,
    from_id bigint NOT NULL,
    created_date bigint NOT NULL,
    post_type varchar(25),
    post_text text,
    PRIMARY KEY (user_id, group_id, post_id),
    CONSTRAINT vk_posts_groups_fk
        FOREIGN KEY (user_id, group_id)
            REFERENCES groups (user_id, group_id)
            ON DELETE CASCADE,
    CONSTRAINT vk_posts_vk_users_fk
        FOREIGN KEY (user_id, from_id)
            REFERENCES vk_users (user_id, vk_user_id)
            ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS vk_posts_user_group_created_idx
    ON vk_posts(user_id, group_id, created_date DESC);

CREATE INDEX IF NOT EXISTS vk_posts_user_from_created_idx
    ON vk_posts(user_id, from_id, created_date DESC);
