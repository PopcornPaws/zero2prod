use sqlx::{Connection, PgConnection};
use zero2prod::configuration::get_configuration;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("failed to read configuration");
    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("failed to connect to postgres");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = std::net::TcpListener::bind(&address).expect("failed to bind random port");
    zero2prod::run(listener, connection)?.await
}
