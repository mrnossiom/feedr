use std::{env::var, path::Path};

use diesel::{
	SqliteConnection,
	r2d2::{ConnectionManager, ManageConnection, Pool, PooledConnection},
};
use eyre::WrapErr;
use serde::Deserialize;

use crate::scheduler::FetcherHandle;

#[derive(Deserialize)]
pub struct Config {
	pub server: ServerConfig,
	pub web: WebConfig,
}

#[derive(Deserialize)]
pub struct ServerConfig {
	pub port: u16,
	pub database_url: String,
}

#[derive(Deserialize)]
pub struct WebConfig {
	pub base_url: String,
}

impl Config {
	pub fn load_file_from_env() -> eyre::Result<Self> {
		let config_path = var("FEEDR_SERVER_CONFIG").unwrap_or_else(|_| "./config.toml".into());

		let config_path = AsRef::<Path>::as_ref(&config_path)
			.canonicalize()
			.wrap_err("could not find the config file")?;

		let config_content =
			std::fs::read_to_string(config_path).wrap_err("could not read the config file")?;
		let config = toml::from_str::<Self>(&config_content)
			.wrap_err("config file does not match the expect structure")?;

		Ok(config)
	}
}

#[derive(Debug, Clone)]
pub struct Ressources {
	database_handle: Pool<ConnectionManager<SqliteConnection>>,
	fetcher_handle: FetcherHandle,
}

impl Ressources {
	pub fn init(config: &Config, fetcher_handle: FetcherHandle) -> eyre::Result<Self> {
		let manager = ConnectionManager::<SqliteConnection>::new(&config.server.database_url);
		let db_pool = Pool::builder()
			.build(manager)
			.wrap_err("could not build database connection pool")?;

		let ressources = Self {
			database_handle: db_pool,
			fetcher_handle,
		};

		Ok(ressources)
	}

	pub fn get_db_conn(
		&self,
	) -> eyre::Result<PooledConnection<ConnectionManager<SqliteConnection>>> {
		self.database_handle
			.get()
			.wrap_err("could not obtain a connection handle")
	}
}
