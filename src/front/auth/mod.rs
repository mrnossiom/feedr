use axum::{
	extract::FromRequestParts,
	http::request::Parts,
	response::{IntoResponse, Response},
};
use eyre::eyre;

use crate::{database::models::User, front::error::RouteError};

mod api;
mod backend;
mod store;

pub use self::api::{ApiAuthnLayer, ApiKey, ApiSession, AuthnService, LoginCredentials};
pub use self::backend::{AuthSession, Backend};
pub use self::store::{SqliteStore, SqliteStoreError};

pub struct UserSession(pub User);

impl<S> FromRequestParts<S> for UserSession
where
	S: Send + Sync,
{
	type Rejection = Response;
	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		let user = AuthSession::from_request_parts(parts, state)
			.await
			.map(|user| user.user.map(Self))
			.map_err(IntoResponse::into_response)?;

		user.ok_or_else(|| {
			RouteError::Other(eyre!("current route is not protected")).into_response()
		})
	}
}

pub fn is_safe_relative_path(path: &str) -> bool {
	path.starts_with('/')
        && !path.starts_with("//")
        && !path.contains("://")
        && !path.contains("..") // avoid directory traversal
        && !path.contains('\0') // avoid null bytes
}
