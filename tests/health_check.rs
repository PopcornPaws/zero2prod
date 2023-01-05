use reqwest::StatusCode;
use sqlx::{Connection, PgConnection};
use zero2prod::configuration::get_configuration;

const TEST_EMAIL: &str = "email=ursula_le_guin%40gmail.com";
const TEST_NAME: &str = "name=le%20guin";

fn spawn_app() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{port}")
}

#[tokio::test]
async fn health_check_works() {
    // arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    // act
    let response = client
        .get(&format!("{address}/health_check"))
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
    let address = spawn_app();
    let configuration = get_configuration().expect("failed to read configuration");
    let client = reqwest::Client::new();
    let mut connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("failed to connect to postgres");
    // act
    let body = format!("{TEST_NAME}&{TEST_EMAIL}");
    let response = client
        .post(&format!("{address}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");
    // assert
    assert_eq!(response.status(), StatusCode::OK);

    let subscription = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("failed to fetch subscription");

    assert_eq!(subscription.email, TEST_EMAIL.strip_prefix("email=").unwrap());
    assert_eq!(subscription.name, TEST_NAME.strip_prefix("name=").unwrap());
}

#[tokio::test]
async fn subscribe_returns_400_for_missing_input() {
    // arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        (TEST_NAME, "missing email"),
        (TEST_EMAIL, "missing name"),
        ("", "missing email and name"),
    ];
    let endpoint = format!("{address}/subscriptions");
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
