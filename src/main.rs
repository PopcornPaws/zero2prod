use sqlx::PgPool;
use zero2prod::configuration::get_config;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_config().expect("failed to read config");
    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("failed to connect to postgres");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = std::net::TcpListener::bind(&address).expect("failed to bind random port");
    zero2prod::run(listener, connection_pool)?.await
}
