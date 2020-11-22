# use the latest Rust stable version as base image
FROM rust:1.47
# switch working directory to `app` (created if doesn't exist)
WORKDIR app
# copy all files from working evironment to Docker image
COPY . .
# build the binary
RUN cargo build --release
# when `docker run is executed, launch the binary
ENTRYPOINT ["./target/release/zero2prod"]
