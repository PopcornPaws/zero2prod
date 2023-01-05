#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener =
        std::net::TcpListener::bind("127.0.0.1:8000").expect("failed to bind random port");
    zero2prod::run(listener)?.await
}
