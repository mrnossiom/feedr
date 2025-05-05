//! `FeedR`

use eyre::Context;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{Config, Ressources};
use crate::front::App;

mod config;
mod database;
mod fetcher;
mod front;
mod utils;

fn setup_tracing() {
	let env_filter =
		EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,feedr_server=debug".into());

	Registry::default()
		.with(env_filter)
		.with(tracing_subscriber::fmt::layer())
		// TODO: add otlp layer
		.init();
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	setup_tracing();

	let config = Config::load_file_from_env().wrap_err("could not load the config")?;
	let ressources = Ressources::init(&config).wrap_err("could not init ressources")?;
	let app = App::new(config, ressources);

	app.serve().await
}
