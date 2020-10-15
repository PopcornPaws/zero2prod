use std::net::TcpListener;

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success()); // status is within 200-299
    assert_eq!(Some(0), response.content_length());
}

// Launch our application in the background.
// No .await call, therefore no need for this function to be async.
fn spawn_app() -> String {
    // Set port number to 0 -> OS will automatically look for an available port
    // to bind the server to.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port!");
    // Retrieve random port number assigned by the OS.
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bind address!");
    // Launch the server as a background task.
    let _ = tokio::spawn(server);
    // Return application address.
    format!("http://127.0.0.1:{}", port)
}
