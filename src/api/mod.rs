use axum::Router;

use crate::{
	auth::AuthnLayer,
	config::{Ressources, RessourcesRef},
};

mod entries;
mod feeds;

pub fn router(ressources: &Ressources) -> Router<RessourcesRef> {
	let authn_layer = AuthnLayer::new_with_api_keys(ressources);

	Router::new()
		.nest("/v0", nightly_api_router())
		.layer(authn_layer)
}

pub fn nightly_api_router() -> Router<RessourcesRef> {
	Router::new()
		.nest("/user/feeds", feeds::router())
		.nest("/user/entries", entries::router())
}
