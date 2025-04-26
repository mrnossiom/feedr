//! `FeedR`

use app::App;
use eyre::Context;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{Config, Ressources};

mod api;
mod app;
mod auth;
mod config;
mod database;
mod error;
mod fetcher;
mod import;
mod web;

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
