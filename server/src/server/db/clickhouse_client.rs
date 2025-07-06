use clickhouse::Client;

pub fn get_clickhouse_client() -> Client {
    Client::default()
        // should include both protocol and port
        .with_url("http://localhost:8123")
        .with_user("sparganothis")
        .with_password("sparganothis")
        .with_database("sparganothis")
        .with_option("async_insert", "1")
        .with_option("wait_for_async_insert", "1")
}
