use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{Router, routing::get};
use eyre::WrapErr;
use tokio::net::TcpListener;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

use crate::api::api_router;
use crate::config::{Config, Ressources};
use crate::scheduler::Scheduler;

mod api;
mod auth;
mod config;
mod import;
mod models;
mod scheduler;

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let config = Config::load_file_from_env().wrap_err("could not load the config")?;
	setup_tracing();

	let ressources = Ressources::init(&config).wrap_err("could not init ressources")?;

	tracing::info!("Starting scheduler");
	let scheduler = Scheduler::setup();

	let app = Router::new()
		.route("/", get(async || "Hello, FeedR!"))
		// .nest("/web", web_router())
		.nest("/api", api_router(ressources.clone()))
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
	Registry::default()
		.with(
			EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| "info,pgpaste_server=debug".into()),
		)
		.with(
			tracing_subscriber::fmt::layer()
				.with_file(true)
				.with_line_number(true),
		)
		// TODO: add otlp layer
		.init();
}
