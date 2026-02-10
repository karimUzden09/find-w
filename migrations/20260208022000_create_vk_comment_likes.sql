CREATE TABLE IF NOT EXISTS vk_comment_likes
(
    user_id uuid NOT NULL,
    vk_user_id bigint NOT NULL,
    group_id bigint NOT NULL,
    post_id bigint NOT NULL,
    comment_id bigint NOT NULL,
    found_date timestamptz NOT NULL,
    PRIMARY KEY (user_id, vk_user_id, group_id, post_id, comment_id),
    CONSTRAINT vk_comment_likes_vk_users_fk
        FOREIGN KEY (user_id, vk_user_id)
            REFERENCES vk_users (user_id, vk_user_id)
            ON DELETE CASCADE,
    CONSTRAINT vk_comment_likes_vk_comments_fk
        FOREIGN KEY (user_id, group_id, post_id, comment_id)
            REFERENCES vk_comments (user_id, group_id, post_id, comment_id)
            ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS vk_comment_likes_user_found_date_idx
    ON vk_comment_likes(user_id, found_date DESC);

CREATE INDEX IF NOT EXISTS vk_comment_likes_user_vk_user_found_date_idx
    ON vk_comment_likes(user_id, vk_user_id, found_date DESC);
