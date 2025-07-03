pub fn get_timestamp_now_ms() -> i64 {
    chrono::offset::Utc::now().timestamp_millis()
}
