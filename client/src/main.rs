use client::app::App;
use client::logger::{init_with_env_filter, tracing::Level};
const LOG_LEVEL: Level = Level::INFO;
const LOG_ENV_FILTER: &str =
    "info,iroh=error,iroh-gossip=error,iroh-relay=error";
// "info";
fn main() {
    init_with_env_filter(LOG_LEVEL, LOG_ENV_FILTER.to_string())
        .expect("logger failed to init");
    dioxus::launch(App);
}
