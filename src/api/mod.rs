use axum::Router;

use crate::{auth::AuthnLayer, config::Ressources};

mod feeds;

pub fn api_router(ressources: &Ressources) -> Router<Ressources> {
	let authn_layer = AuthnLayer::new_with_api_keys(ressources.clone());

	Router::new()
		.nest("/v0", nightly_api_router())
		.layer(authn_layer)
}

pub fn nightly_api_router() -> Router<Ressources> {
	Router::new().nest("/feeds", feeds::api_router())
}
