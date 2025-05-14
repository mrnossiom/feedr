use std::{env::var, ops, path::Path, sync::Arc};

use axum::extract::FromRequestParts;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use eyre::WrapErr;
use eyre::eyre;
use serde::Deserialize;
use url::Url;

use crate::{
	database::{PoolConnection, models::FeedId},
	fetcher::{FetchTask, Fetcher, FetcherHandle},
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

	pub session_secret: String,
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

impl FromRequestParts<Self> for RessourcesRef {
	type Rejection = ();
	async fn from_request_parts(
		_parts: &mut axum::http::request::Parts,
		state: &Self,
	) -> Result<Self, Self::Rejection> {
		Ok(state.clone())
	}
}

impl Ressources {
	pub fn init(config: &Config) -> eyre::Result<RessourcesRef> {
		let manager = ConnectionManager::<PgConnection>::new(&config.server.database_url);
		let db_pool = Pool::builder()
			.build(manager)
			.wrap_err("could not build database connection pool")?;

		tracing::info!("starting fetcher");
		let fetcher_handle = Fetcher::setup(db_pool.clone()).wrap_err("could not start fetcher")?;

		let ressources = Self {
			database_handle: db_pool,
			fetcher_handle,
		};

		ressources
			.run_migrations()
			.wrap_err("could not run migrations")?;

		Ok(RessourcesRef(Arc::new(ressources)))
	}

	fn run_migrations(&self) -> eyre::Result<()> {
		const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

		let mut conn = self.database_handle.get()?;

		conn.run_pending_migrations(MIGRATIONS)
			.map_err(|err| eyre!("{}", err))?;

		Ok(())
	}

	pub async fn fetch_url(&self, feed_id: FeedId, url: Url) -> eyre::Result<()> {
		self.fetcher_handle
			.fetch_feed(FetchTask { feed_id, url })
			.await
	}
}
