use sqlx::postgres::PgPoolOptions;
use zero2prod::configuration::get_config;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_config().expect("failed to read config");
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());
    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = std::net::TcpListener::bind(&address).expect("failed to bind random port");
    zero2prod::startup::run(listener, connection_pool)?.await
}
