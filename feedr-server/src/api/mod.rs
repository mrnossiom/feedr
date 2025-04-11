use crate::{
	auth::{AuthSession, AuthnLayer},
	config::Ressources,
};
use axum::{Form, Router, extract::Multipart, http::StatusCode, routing::post};

pub fn api_router(ressources: &Ressources) -> Router {
	let authn_layer = AuthnLayer::new_with_api_keys(ressources.db_pool.clone());

	Router::new()
		.nest("/v0", nightly_api_router())
		.layer(authn_layer)
}

pub fn nightly_api_router() -> Router {
	Router::new().route("/import", post(import_handler))
}

async fn import_handler(auth: AuthSession, multipart: Multipart) -> StatusCode {
	let Some(user_id) = auth.user_id else {
		return StatusCode::UNAUTHORIZED;
	};

	StatusCode::IM_A_TEAPOT
}
