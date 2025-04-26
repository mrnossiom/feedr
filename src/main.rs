//! `FeedR`

use std::{
	net::{IpAddr, Ipv4Addr, SocketAddr},
	sync::Arc,
};

use axum::{Router, http::HeaderName};
use axum_login::AuthManagerLayerBuilder;
use base64::{Engine, prelude::BASE64_STANDARD};
use eyre::WrapErr;
use reqwest::header;
use time::Duration;
use tokio::net::TcpListener;
use tower_http::{
	request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
	sensitive_headers::{SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer},
	trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer, cookie::Key};
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

use crate::auth::{Backend, SqliteStore};
use crate::config::{Config, Ressources, RessourcesRef};

mod api;
mod auth;
mod config;
mod database;
mod error;
mod fetcher;
mod import;
mod web;

struct App {
	config: Config,
	ressources: RessourcesRef,
}

impl App {
	async fn serve(self) -> eyre::Result<()> {
		let session_store = SqliteStore::new(self.ressources.database_handle.clone());

		let deletion_task = tokio::task::spawn(
			session_store
				.clone()
				.continuously_delete_expired(tokio::time::Duration::from_secs(60)),
		);

		let session_key = Key::from(
			&BASE64_STANDARD
				.decode(self.config.server.session_secret.as_bytes())
				.wrap_err("could not decode session secret in base64")?,
		);
		let session_layer = SessionManagerLayer::new(session_store)
			.with_expiry(Expiry::OnInactivity(Duration::days(1)))
			.with_secure(false)
			.with_signed(session_key);

		let app = Router::new()
			.merge(web::router())
			.nest("/api", api::router(&self.ressources));

		let x_request_id = HeaderName::from_static("x-request-id");
		let headers: Arc<[_]> =
			Arc::new([header::AUTHORIZATION, header::COOKIE, header::SET_COOKIE]);

		let session_backend = Backend::new(self.ressources.database_handle.clone());
		let session_auth_layer =
			AuthManagerLayerBuilder::new(session_backend, session_layer).build();

		let layered_app = app
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
			.layer(session_auth_layer)
			.with_state(self.ressources);

		let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), self.config.server.port);
		let listener = TcpListener::bind(addr)
			.await
			.wrap_err_with(|| format!("could not bind to the specified interface: {addr:?}"))?;

		tracing::info!("starting app router");
		axum::serve(listener, layered_app)
			.await
			.wrap_err("could not serve app")?;

		deletion_task.await??;

		Ok(())
	}
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let config = Config::load_file_from_env().wrap_err("could not load the config")?;
	setup_tracing();

	let ressources = Ressources::init(&config).wrap_err("could not init ressources")?;

	let app = App { config, ressources };

	app.serve().await?;

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
