use axum::{
	extract::FromRequestParts,
	http::{Request, header, request::Parts},
};
use diesel::prelude::*;
use serde::Deserialize;
use tower::{Layer, Service};

use crate::{
	config::Ressources,
	database::{PoolConnection, models::UserId},
	front::error::{AuthError, RouteError},
};

#[derive(Debug, Clone)]
pub struct ApiKey(String);

#[derive(Debug, Clone)]
pub struct ApiSession {
	pub user_id: Option<UserId>,
}

impl ApiSession {
	pub fn user_id(&self) -> Result<UserId, AuthError> {
		self.user_id.ok_or(AuthError::NotAuthenticated)
	}
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum LoginCredentials {
	#[serde(rename = "basic")]
	Basic { username: String, password: String },
	#[serde(rename = "dauth")]
	DAuth { instance: String },
}

impl<S: Send + Sync> FromRequestParts<S> for ApiSession {
	type Rejection = RouteError;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
		let msg = "logic error: could not access `ApiSession` extension";
		parts
			.extensions
			.get::<Self>()
			.cloned()
			.ok_or(RouteError::Static(msg))
	}
}

#[derive(Debug, Clone)]
pub struct ApiAuthnLayer {
	db_handle: PoolConnection,
}

impl ApiAuthnLayer {
	pub fn new(ressources: &Ressources) -> Self {
		Self {
			db_handle: ressources.database_handle.clone(),
		}
	}
}

impl<S> Layer<S> for ApiAuthnLayer {
	type Service = AuthnService<S>;

	fn layer(&self, service: S) -> Self::Service {
		AuthnService {
			service,
			db_handle: self.db_handle.clone(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct AuthnService<S> {
	service: S,
	db_handle: PoolConnection,
}

impl<S> AuthnService<S> {
	fn extract_api_key<ReqBody>(req: &Request<ReqBody>) -> Option<ApiKey> {
		let authz_header = req.headers().get(header::AUTHORIZATION)?;
		let api_key = authz_header.to_str().ok()?.strip_prefix("Bearer ")?;

		// invalid api key
		if !api_key.starts_with("fdr_v0_") {
			return None;
		}

		Some(ApiKey(api_key.to_owned()))
	}

	fn resolve_user(&self, api_key: &ApiKey) -> Option<UserId> {
		use crate::database::schema::*;
		let mut conn = self.db_handle.get().ok()?;
		let user_id = api_key::table
			.select(api_key::user_id)
			.filter(api_key::secret.eq(&api_key.0))
			.get_result::<UserId>(&mut conn)
			.ok()?;
		Some(user_id)
	}
}

impl<S: Service<Request<ReqBody>>, ReqBody> Service<Request<ReqBody>> for AuthnService<S> {
	type Response = S::Response;
	type Error = S::Error;
	type Future = S::Future;

	fn poll_ready(
		&mut self,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), Self::Error>> {
		self.service.poll_ready(cx)
	}

	fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
		let api_key = Self::extract_api_key(&req);

		let user_id = api_key
			.as_ref()
			.and_then(|api_key| self.resolve_user(api_key));

		let session = ApiSession { user_id };
		req.extensions_mut().insert(session);

		self.service.call(req)
	}
}
