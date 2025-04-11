use crate::config::Config;
use api::api_router;
use axum::{Router, routing::get};
use config::Ressources;
use eyre::WrapErr;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;

mod api;
mod auth;
mod config;

#[tokio::main]
async fn main() -> eyre::Result<()> {
	println!("Hello, FeedR!");

	let config = Config::load_file_from_env().wrap_err("could not load the config")?;
	let ressources = Ressources::init(&config).wrap_err("could not init ressources")?;

	let app = Router::new()
		.route("/", get(async || "Hello, FeedR!"))
		// .nest("/web", Router::new())
		.nest("/api", api_router(&ressources));

	let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);
	let listener = TcpListener::bind(addr)
		.await
		.wrap_err_with(|| format!("could not bind to the specified interface: {addr:?}"))?;

	axum::serve(listener, app)
		.await
		.wrap_err("could not serve app")?;

	Ok(())
}
