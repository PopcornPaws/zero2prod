use reqwest::StatusCode;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::configuration::{get_config, DatabaseConfig};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

use std::sync::Once;

static TRACING: Once = Once::new();

const TEST_EMAIL: &str = "email=ursula_le_guin%40gmail.com";
const TEST_NAME: &str = "name=le%20guin";
const SAVED_EMAIL: &str = "ursula_le_guin@gmail.com";
const SAVED_NAME: &str = "le guin";

struct TestApp {
    address: String,
    db_pool: PgPool,
}

#[allow(clippy::let_underscore_future)]
async fn spawn_app() -> TestApp {
    TRACING.call_once(|| {
        let subscriber_name = "test".to_string();
        let default_filter_level = "debug".to_string();
        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
            init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
            init_subscriber(subscriber);
        }
    });

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");
    let mut configuration = get_config().expect("failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&configuration.database).await;

    let server =
        zero2prod::startup::run(listener, db_pool.clone()).expect("failed to bind address");
    let _ = tokio::spawn(server);
    TestApp { address, db_pool }
}

pub async fn configure_database(config: &DatabaseConfig) -> PgPool {
    // create database
    let mut connection =
        PgConnection::connect(config.connection_string_without_db().expose_secret())
            .await
            .expect("failed to connect to postgres");
    connection
        .execute(format!("CREATE DATABASE \"{}\";", config.database_name).as_str())
        .await
        .expect("failed to create database");
    // migrate database
    let connection_pool = PgPool::connect(config.connection_string().expose_secret())
        .await
        .expect("failed to connect ot postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("failed to migrate database");
    // return the connection pool
    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    // arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    // act
    let response = client
        .get(&format!("{}/health_check", app.address))
        .send()
        .await
        .expect("failed to execute request");
    // assert
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.content_length(), Some(0));
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_input() {
    // arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    // act
    let body = format!("{TEST_NAME}&{TEST_EMAIL}");
    let response = client
        .post(&format!("{}/subscribe", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");
    // assert
    assert_eq!(response.status(), StatusCode::OK);

    let subscription = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("failed to fetch subscription");

    assert_eq!(subscription.email, SAVED_EMAIL);
    assert_eq!(subscription.name, SAVED_NAME);
}

#[tokio::test]
async fn subscribe_returns_400_for_missing_input() {
    // arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (TEST_NAME, "missing email"),
        (TEST_EMAIL, "missing name"),
        ("", "missing email and name"),
    ];
    let endpoint = format!("{}/subscribe", app.address);
    // act
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("failed to execute request");
        // assert
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "should have failed with message: {error_message}",
        );
    }
}
