CREATE TABLE mspaint
(
    user_id BLOB,
    board_name String,
    time_created BigInt,
    time_modified BigInt,

    data_version BigInt,
    mspaint_data BLOB,
)
ENGINE = MergeTree ()
ORDER BY (user_id, board_name, time_created, time_modified)
COMMENT 'all binary states for mspaint feature';

CREATE TABLE game_states
(
    game_type String,
    user_id String,
    start_time BigInt,
    game_seed String,
    state_idx BigInt,

    data_version BigInt,
    last_action BLOB,
    state_data BLOB,
)
ENGINE = MergeTree ()
ORDER BY (game_type, user_id, start_time, game_seed, state_idx)
COMMENT 'all states in all games';


CREATE TABLE games
(
    game_type String,
    user_id String,
    start_time BigInt,
    game_seed String,

    data_version BigInt,
    match_info Nullable(BLOB)
)
ENGINE = MergeTree ()
ORDER BY (game_type, user_id, start_time, game_seed)
COMMENT 'list of all games (without states)';


CREATE TABLE matches
(
    game_type String,
    start_time  BigInt,
    user_ids Array(BLOB),
    game_seed String,
    match_id UUID,

    data_version BigInt,
    match_info BLOB,
)
ENGINE = MergeTree ()
ORDER BY (game_type, start_time, user_ids,  game_seed, match_id)
COMMENT 'list of multi-user matches';