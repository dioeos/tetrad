use super::error::Error;

use std::{net::SocketAddr, sync::OnceLock};

pub struct Config {
    database_url: String,
    bind_address: SocketAddr,
    instance_name: String,
    base_url: String,
}

const DEFAULT_DB_URL: &str = "sqlite://src/server/data/tetrad.sqlite3";
const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:8080";
const DEFAULT_INSTANCE_NAME: &str = "tetrad";
const DEFAULT_BASE_URL: &str = "http://localhost:8080";

pub fn use_config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();

    CONFIG.get_or_init(|| {
        Config::load_from_environment()
            .unwrap_or_else(|err| panic!("FATAL - WHILE LOADING CONF - Cause: {err:?}"))
    })
}

impl Config {
    pub fn new(
        database_url: impl Into<String>,
        bind_address: SocketAddr,
        instance_name: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            database_url: database_url.into(),
            bind_address,
            instance_name: instance_name.into(),
            base_url: base_url.into(),
        }
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn bind_address(&self) -> &SocketAddr {
        &self.bind_address
    }

    pub fn instance_name(&self) -> &str {
        &self.instance_name
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn load_from_environment() -> Result<Self, Error> {
        let database_url =
            std::env::var("TETRAD_DATABASE_URL").unwrap_or_else(|_| DEFAULT_DB_URL.to_owned());

        let bind_address = std::env::var("TETRAD_BIND_ADDRESS")
            .unwrap_or_else(|_| DEFAULT_BIND_ADDRESS.to_owned())
            .parse()
            .map_err(Error::InvalidBindAddress)?;

        let instance_name = std::env::var("TETRAD_INSTANCE_NAME")
            .unwrap_or_else(|_| DEFAULT_INSTANCE_NAME.to_owned());

        let base_url =
            std::env::var("TETRAD_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_owned());

        Ok(Self {
            database_url,
            bind_address,
            instance_name,
            base_url,
        })
    }
}
