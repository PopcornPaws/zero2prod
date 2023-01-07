use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::configuration::get_config;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".to_string(), "info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_config().expect("failed to read config");
    let connection_pool = PgPool::connect(config.database.connection_string().expose_secret())
        .await
        .expect("failed to connect to postgres");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = std::net::TcpListener::bind(&address).expect("failed to bind random port");
    zero2prod::startup::run(listener, connection_pool)?.await
}
