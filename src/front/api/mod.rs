use axum::Router;

use crate::{
	config::{Ressources, RessourcesRef},
	front::auth::ApiAuthnLayer,
};

mod entries;
mod feeds;

pub fn router(ressources: &Ressources) -> Router<RessourcesRef> {
	let api_auth_layer = ApiAuthnLayer::new(ressources);

	Router::new()
		.nest("/v0", nightly_api_router())
		.layer(api_auth_layer)
}

pub fn nightly_api_router() -> Router<RessourcesRef> {
	Router::new()
		.nest("/user/feeds", feeds::router())
		.nest("/user/entries", entries::router())
}
