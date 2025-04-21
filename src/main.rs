//! `FeedR`

use std::{
	net::{IpAddr, Ipv4Addr, SocketAddr},
	sync::Arc,
};

use axum::{Router, http::HeaderName};
use eyre::WrapErr;
use reqwest::header;
use tokio::net::TcpListener;
use tower_http::{
	request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
	sensitive_headers::{SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer},
	trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{Config, Ressources};

mod api;
mod auth;
mod config;
mod database;
mod error;
mod fetcher;
mod import;
mod web;

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let config = Config::load_file_from_env().wrap_err("could not load the config")?;
	setup_tracing();

	let ressources = Ressources::init(&config).wrap_err("could not init ressources")?;

	let x_request_id = HeaderName::from_static("x-request-id");

	let headers: Arc<[_]> = Arc::new([header::AUTHORIZATION, header::COOKIE, header::SET_COOKIE]);

	let app = Router::new()
		.nest("/api", api::router(&ressources))
		.merge(web::router(&ressources))
		.layer(PropagateRequestIdLayer::new(x_request_id.clone()))
		.layer(SetSensitiveResponseHeadersLayer::from_shared(
			headers.clone(),
		))
		.layer(
			TraceLayer::new_for_http()
				.make_span_with(DefaultMakeSpan::new().include_headers(true))
				.on_response(DefaultOnResponse::new().include_headers(true)),
		)
		.layer(SetSensitiveRequestHeadersLayer::from_shared(headers))
		.layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid))
		.with_state(ressources);

	let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.server.port);
	let listener = TcpListener::bind(addr)
		.await
		.wrap_err_with(|| format!("could not bind to the specified interface: {addr:?}"))?;

	tracing::info!("starting app router");
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
		.with(tracing_subscriber::fmt::layer())
		// TODO: add otlp layer
		.init();
}
