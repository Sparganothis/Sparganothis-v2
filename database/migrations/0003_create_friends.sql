CREATE TABLE user_friends
(
    user_id BLOB,
    friend_id BLOB,

    added_on BigInt,
    data_version BigInt,
)
ENGINE = ReplacingMergeTree ()
ORDER BY (user_id, friend_id)
COMMENT 'user friend list';