use diesel::{
	SqliteConnection,
	r2d2::{ConnectionManager, Pool},
};
use eyre::WrapErr;
use facet::Facet;
use std::{env::var, path::Path, sync::Arc};

#[derive(Facet)]
pub struct Config {
	pub base_url: String,

	pub port: u16,

	pub database_url: String,
}

impl Config {
	pub fn load_file_from_env() -> eyre::Result<Self> {
		let config_path = var("FEEDR_SERVER_CONFIG").unwrap_or_else(|_| "./config.toml".into());

		let config_path = AsRef::<Path>::as_ref(&config_path)
			.canonicalize()
			.wrap_err("could not find the config file")?;

		let config_content =
			std::fs::read_to_string(config_path).wrap_err("could not read the config file")?;
		let config = facet_toml::from_str::<Self>(&config_content)
			.wrap_err("config file does not match the expect structure")?;

		Ok(config)
	}
}

#[derive(Debug)]
pub struct Ressources {
	pub db_pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl Ressources {
	pub(crate) fn init(config: &Config) -> eyre::Result<Arc<Self>> {
		let manager = ConnectionManager::<SqliteConnection>::new(&config.database_url);
		let db_pool = Pool::builder()
			.build(manager)
			.wrap_err("could not build database connection pool")?;

		let ressources = Self { db_pool };

		Ok(Arc::new(ressources))
	}
}
