//! `FeedR`

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{Router, routing::get};
use eyre::WrapErr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

use crate::api::api_router;
use crate::config::{Config, Ressources};
use crate::scheduler::Fetcher;

mod api;
mod auth;
mod config;
mod database;
mod import;
mod scheduler;

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let config = Config::load_file_from_env().wrap_err("could not load the config")?;
	setup_tracing();

	tracing::info!("Starting fetcher");
	let fetcher_handler = Fetcher::setup().wrap_err("could not start fetcher")?;

	let ressources =
		Ressources::init(&config, fetcher_handler).wrap_err("could not init ressources")?;

	let app = Router::new()
		.route("/", get(async || "Hello, FeedR!"))
		// .nest("/web", web_router())
		.nest("/api", api_router(&ressources))
		.layer(TraceLayer::new_for_http())
		.with_state(ressources);

	let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.server.port);
	let listener = TcpListener::bind(addr)
		.await
		.wrap_err_with(|| format!("could not bind to the specified interface: {addr:?}"))?;

	tracing::info!("Starting app router");
	axum::serve(listener, app)
		.await
		.wrap_err("could not serve app")?;

	Ok(())
}

fn setup_tracing() {
	let env_filter =
		EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,feedr_server=debug".into());

	Registry::default()
		.with(env_filter)
		.with(
			tracing_subscriber::fmt::layer()
				.with_file(true)
				.with_line_number(true),
		)
		// TODO: add otlp layer
		.init();
}
