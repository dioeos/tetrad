use std::net::SocketAddr;

use anyhow::Context;

pub struct Config {
    pub(crate) database_url: String,
    pub(crate) bind_address: SocketAddr,
    pub(crate) instance_name: String,
    pub(crate) base_url: String
}

impl Config {
    pub fn new(
        database_url: impl Into<String>,
        bind_address: SocketAddr,
        instance_name: impl Into<String>,
        base_url: impl Into<String>
    ) -> Self {
        Self {
            database_url: database_url.into(),
            bind_address,
            instance_name: instance_name.into(),
            base_url: base_url.into()
        }
    }
    pub fn from_environment() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://data/tetrad.sqlite3".to_owned());

        let bind_address = std::env::var("TETRAD_BIND_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_owned())
            .parse()
            .context("TETRAD_BIND_ADDRESS must be a socket address")?;

        let instance_name = std::env::var("TETRAD_INSTANCE_NAME")
            .unwrap_or_else(|_| "tetrad".to_owned());

        let base_url = std::env::var("TETRAD_BASE_URL")
            .unwrap_or_else(|_| "http:://localhost:8080".to_owned());

        Ok(Self {
            database_url,
            bind_address,
            instance_name,
            base_url
        })
    }
}
