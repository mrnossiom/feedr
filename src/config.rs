use std::{env::var, ops, path::Path, sync::Arc};

use axum::extract::FromRequestParts;
use diesel::{
	SqliteConnection,
	r2d2::{ConnectionManager, Pool, PooledConnection},
};
use eyre::WrapErr;
use serde::Deserialize;
use url::Url;

use crate::{
	database::{PoolConnection, models::FeedId},
	scheduler::{FetchTask, FetcherHandle},
};

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

#[derive(Debug)]
pub struct Ressources {
	pub database_handle: PoolConnection,
	pub fetcher_handle: FetcherHandle,
}

#[derive(Debug, Clone)]
pub struct RessourcesRef(Arc<Ressources>);

impl ops::Deref for RessourcesRef {
	type Target = Ressources;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl FromRequestParts<RessourcesRef> for RessourcesRef {
	type Rejection = ();
	async fn from_request_parts(
		_parts: &mut axum::http::request::Parts,
		state: &RessourcesRef,
	) -> Result<Self, Self::Rejection> {
		Ok(state.clone())
	}
}

impl Ressources {
	pub fn init(config: &Config, fetcher_handle: FetcherHandle) -> eyre::Result<RessourcesRef> {
		let manager = ConnectionManager::<SqliteConnection>::new(&config.server.database_url);
		let db_pool = Pool::builder()
			.build(manager)
			.wrap_err("could not build database connection pool")?;

		let ressources = Self {
			database_handle: db_pool,
			fetcher_handle,
		};

		Ok(RessourcesRef(Arc::new(ressources)))
	}

	pub fn get_db_conn(
		&self,
	) -> eyre::Result<PooledConnection<ConnectionManager<SqliteConnection>>> {
		self.database_handle
			.get()
			.wrap_err("could not obtain a connection handle")
	}

	pub fn fetch_url(&self, feed_id: FeedId, url: Url) -> impl Future<Output = eyre::Result<()>> {
		self.fetcher_handle.fetch_feed(FetchTask { feed_id, url })
	}
}
