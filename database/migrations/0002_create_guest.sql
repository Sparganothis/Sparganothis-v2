CREATE TABLE guest_users
(
    user_id BLOB,

    nickname String,
    first_login BigInt,
    data_version BigInt,
)
ENGINE = MergeTree ()
ORDER BY (user_id, nickname, first_login)
COMMENT 'guest user list';


CREATE TABLE guest_user_login_events
(
    user_id BLOB,

    last_login BigInt,
)
ENGINE = MergeTree ()
ORDER BY (user_id, last_login)
COMMENT 'guest user event logins';
