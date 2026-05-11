


pub fn compute_elo(elo_score_percent: i32, our_elo: f64, opponent_elo: f64) -> (f64, String) {
    use skillratings::{
    elo::{elo, EloConfig, EloRating},
    Outcomes,
};

// Initialise a new player rating with a rating of 1000.
let player_one = EloRating {
    rating: our_elo,
};

// Or you can initialise it with your own values of course.
// Imagine these numbers being pulled from a database.
let player_two = EloRating {
    rating: opponent_elo,
};

// The outcome of the match is from the perspective of player one.
let outcome = match elo_score_percent {
    66..=i32::MAX => Outcomes::WIN,
    i32::MIN..=33 => Outcomes::LOSS,
    _ => Outcomes::DRAW,
};

// The config allows you to specify certain values in the Elo calculation.
// Here we modify the k-value to be 20.0, instead of the usual 32.0.
// To simplify massively: This means the ratings will not change as much.
let config = EloConfig { k: 20.0 };

// The elo function will calculate the new ratings for both players and return them.
let (new_player_one, _new_player_two) = elo(&player_one, &player_two, &outcome, &config);

(new_player_one.rating, format!("{:?}", outcome))
}