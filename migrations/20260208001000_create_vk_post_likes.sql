CREATE TABLE IF NOT EXISTS vk_post_likes
(
    user_id uuid NOT NULL,
    vk_user_id bigint NOT NULL,
    group_id bigint NOT NULL,
    post_id bigint NOT NULL,
    found_date timestamptz NOT NULL,
    PRIMARY KEY (user_id, vk_user_id, group_id, post_id),
    CONSTRAINT vk_post_likes_vk_users_fk
        FOREIGN KEY (user_id, vk_user_id)
            REFERENCES vk_users (user_id, vk_user_id)
            ON DELETE CASCADE,
    CONSTRAINT vk_post_likes_vk_posts_fk
        FOREIGN KEY (user_id, group_id, post_id)
            REFERENCES vk_posts (user_id, group_id, post_id)
            ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS vk_post_likes_user_found_date_idx
    ON vk_post_likes(user_id, found_date DESC);

CREATE INDEX IF NOT EXISTS vk_post_likes_user_vk_user_found_date_idx
    ON vk_post_likes(user_id, vk_user_id, found_date DESC);
