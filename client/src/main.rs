use client::app::App;
use dioxus::logger::tracing::Level;
fn main() {
    dioxus::logger::init(Level::INFO).expect("logger failed to init");
    dioxus::launch(App);
}
