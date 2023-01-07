use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_config() -> Result<Config, config::ConfigError> {
    let mut configuration = config::Config::default();
    configuration.merge(config::File::with_name("configuration"))?;
    configuration.try_into()
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}
