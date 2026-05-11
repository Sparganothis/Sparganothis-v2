-- Migration: ClickHouse -> MariaDB 12.2 (InnoDB)
-- Source: database/migrations/0001_create_mspaint.sql
--         database/migrations/0002_create_guest.sql
--         database/migrations/0003_create_friends.sql

-- -------------------------------------------------------
-- mspaint  (was: MergeTree ORDER BY (user_id, board_name, time_created, time_modified))
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS mspaint (
    user_id       VARCHAR(64)    NOT NULL,
    board_name    VARCHAR(64)        NOT NULL,
    time_created  BIGINT      NOT NULL,
    time_modified BIGINT      NOT NULL,
    data_version  BIGINT      NOT NULL,
    mspaint_data  VARCHAR(2048)    NOT NULL,
    -- surrogate PK because BLOB/VARCHAR(64) columns cannot be part of a PK directly
    id            BIGINT      NOT NULL AUTO_INCREMENT,
    PRIMARY KEY (id),
    UNIQUE (user_id, board_name, time_created, time_modified)
) ENGINE = InnoDB
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_ci
  COMMENT = 'all binary states for mspaint feature';

-- -------------------------------------------------------
-- game_states  (was: MergeTree ORDER BY (game_type, user_id, start_time, game_seed, recv_time, score))
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS game_states (
    game_type    VARCHAR(64)        NOT NULL,
    user_id      VARCHAR(64)        NOT NULL,
    start_time   BIGINT      NOT NULL,
    game_seed    VARCHAR(64)        NOT NULL,
    recv_time    BIGINT      NOT NULL,
    score        BIGINT      NOT NULL,
    data_version BIGINT      NOT NULL,
    last_action  VARCHAR(64)    NOT NULL,
    state_data   VARCHAR(2048)    NOT NULL,
    id           BIGINT      NOT NULL AUTO_INCREMENT,
    PRIMARY KEY (id),
    UNIQUE  (game_type, user_id, start_time, game_seed, recv_time, score)
) ENGINE = InnoDB
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_ci
  COMMENT = 'all states in all games';

-- -------------------------------------------------------
-- games  (was: MergeTree ORDER BY (game_type, user_id, start_time, game_seed))
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS games (
    game_type    VARCHAR(64)        NOT NULL,
    user_id      VARCHAR(64)        NOT NULL,
    start_time   BIGINT      NOT NULL,
    game_seed    VARCHAR(64)        NOT NULL,
    data_version BIGINT      NOT NULL,
    match_info   VARCHAR(2048)    NULL,
    id           BIGINT      NOT NULL AUTO_INCREMENT,
    PRIMARY KEY (id),
    UNIQUE (game_type, user_id, start_time, game_seed)
) ENGINE = InnoDB
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_ci
  COMMENT = 'list of all games (without states)';

-- -------------------------------------------------------
-- matches  (was: MergeTree ORDER BY (game_type, start_time, user_ids, game_seed, match_id))
-- user_ids was Array(BLOB) in ClickHouse; stored as serialized VARCHAR(64) here.
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS matches (
    game_type    VARCHAR(64)        NOT NULL,
    start_time   BIGINT      NOT NULL,
    user_ids     VARCHAR(2048)    NOT NULL,
    game_seed    VARCHAR(64)        NOT NULL,
    match_id     VARCHAR(64)        NOT NULL,
    data_version BIGINT      NOT NULL,
    match_info   VARCHAR(2048)    NOT NULL,
    id           BIGINT      NOT NULL AUTO_INCREMENT,
    PRIMARY KEY (id),
    INDEX idx_matches_lookup (start_time)
) ENGINE = InnoDB
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_ci
  COMMENT = 'list of multi-user matches';

-- -------------------------------------------------------
-- guest_users  (was: MergeTree ORDER BY (user_id, nickname, first_login))
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS guest_users (
    user_id      VARCHAR(64)    NOT NULL,
    nickname     VARCHAR(64)        NOT NULL,
    first_login  BIGINT      NOT NULL,
    data_version BIGINT      NOT NULL,
    id           BIGINT      NOT NULL AUTO_INCREMENT,
    PRIMARY KEY (id),
    UNIQUE  (user_id, nickname, first_login)
) ENGINE = InnoDB
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_ci
  COMMENT = 'guest user list';

-- -------------------------------------------------------
-- guest_user_login_events  (was: MergeTree ORDER BY (user_id, last_login))
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS guest_user_login_events (
    user_id    VARCHAR(64)    NOT NULL,
    last_login BIGINT      NOT NULL,
    id         BIGINT      NOT NULL AUTO_INCREMENT,
    PRIMARY KEY (id),
    UNIQUE  (user_id, last_login)
) ENGINE = InnoDB
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_ci
  COMMENT = 'guest user event logins';

-- -------------------------------------------------------
-- user_friends  (was: ReplacingMergeTree ORDER BY (user_id, friend_id))
-- UNIQUE KEY enforces the deduplication semantics of ReplacingMergeTree.
-- Use VARCHAR(64) for the key columns so they can participate in a UNIQUE index
-- (BLOB/VARCHAR(64) columns require a prefix length, which loses uniqueness guarantees).
-- Store full binary value in separate _bin columns if needed.
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS user_friends (
    user_id      VARCHAR(64)    NOT NULL,
    friend_id    VARCHAR(64)    NOT NULL,
    added_on     BIGINT      NOT NULL,
    data_version BIGINT      NOT NULL,
    id           BIGINT      NOT NULL AUTO_INCREMENT,
    PRIMARY KEY (id),
    UNIQUE (user_id, friend_id)
) ENGINE = InnoDB
  CHARACTER SET utf8mb4
  COLLATE utf8mb4_unicode_ci
  COMMENT = 'user friend list';
