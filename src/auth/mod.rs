use std::{collections::HashMap, sync::Arc, task::Context};

use axum::{
	extract::FromRequestParts,
	http::{Request, header, request::Parts},
};
use diesel::prelude::*;
use parking_lot::Mutex;
use tower::{Layer, Service};

use crate::{
	config::Ressources,
	database::{PoolConnection, models::UserId},
	error::{AuthError, RouteError},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionSecret(pub String);

#[derive(Debug, Clone)]
pub struct ApiKey(String);

#[derive(Debug, Clone)]
pub enum LoginKind {
	Session(SessionSecret),
	ApiKey(ApiKey),
}

#[derive(Debug, Clone)]
pub struct AuthSession {
	pub user_id: Option<UserId>,
	pub session_map: Arc<Mutex<HashMap<SessionSecret, UserId>>>,
}

impl AuthSession {
	pub fn user_id(&self) -> Result<UserId, AuthError> {
		self.user_id.ok_or(AuthError::NotAuthenticated)
	}

	pub fn user_id_or_redirect(&self) -> Result<UserId, AuthError> {
		self.user_id.ok_or(AuthError::ReturnToLogin)
	}
}

impl<S: Send + Sync> FromRequestParts<S> for AuthSession {
	type Rejection = RouteError;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
		let msg = "logic error: could not access `AuthSession` extension";
		parts
			.extensions
			.get::<Self>()
			.cloned()
			.ok_or(RouteError::Static(msg))
	}
}

#[derive(Debug, Clone)]
pub struct AuthnLayer {
	db_handle: PoolConnection,
	session_map: Arc<Mutex<HashMap<SessionSecret, UserId>>>,
	allow_api_keys: bool,
}

impl AuthnLayer {
	pub fn new(ressources: &Ressources) -> Self {
		let session_map = HashMap::new();

		Self {
			db_handle: ressources.database_handle.clone(),
			session_map: Arc::new(Mutex::new(session_map)),
			allow_api_keys: false,
		}
	}

	pub fn new_with_api_keys(ressources: &Ressources) -> Self {
		Self {
			allow_api_keys: true,
			..Self::new(ressources)
		}
	}
}

impl<S> Layer<S> for AuthnLayer {
	type Service = AuthnService<S>;

	fn layer(&self, service: S) -> Self::Service {
		AuthnService {
			service,
			db_handle: self.db_handle.clone(),
			session_map: self.session_map.clone(),
			allow_api_keys: self.allow_api_keys,
		}
	}
}

#[derive(Debug, Clone)]
pub struct AuthnService<S> {
	service: S,
	db_handle: PoolConnection,
	session_map: Arc<Mutex<HashMap<SessionSecret, UserId>>>,
	allow_api_keys: bool,
}

impl<S> AuthnService<S> {
	fn extract_session<ReqBody>(req: &Request<ReqBody>) -> Option<SessionSecret> {
		let cookie_header = req.headers().get(header::COOKIE)?;
		let (key, value) = cookie_header.to_str().unwrap().split_once('=')?;

		if key != "session" {
			return None;
		}

		Some(SessionSecret(value.to_owned()))
	}

	fn extract_api_key<ReqBody>(req: &Request<ReqBody>) -> Option<ApiKey> {
		let authz_header = req.headers().get(header::AUTHORIZATION)?;
		let api_key = authz_header.to_str().unwrap().strip_prefix("Bearer ")?;

		// invalid api key
		if !api_key.starts_with("fdr_v0_") {
			return None;
		}

		Some(ApiKey(api_key.to_owned()))
	}

	fn resolve_user(&self, login_kind: &LoginKind) -> Option<UserId> {
		match login_kind {
			LoginKind::Session(session) => self.session_map.lock().get(session).copied(),
			LoginKind::ApiKey(key) => {
				use crate::database::schema::*;
				let mut conn = self.db_handle.get().ok()?;
				let user_id = api_key::table
					.select(api_key::user_id)
					.filter(api_key::secret.eq(&key.0))
					.get_result::<UserId>(&mut conn)
					.ok()?;
				Some(user_id)
			}
		}
	}
}

impl<S: Service<Request<ReqBody>>, ReqBody> Service<Request<ReqBody>> for AuthnService<S> {
	type Response = S::Response;
	type Error = S::Error;
	type Future = S::Future;

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
		self.service.poll_ready(cx)
	}

	fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
		let login_kind = Self::extract_session(&req)
			.map(LoginKind::Session)
			.or_else(|| {
				self.allow_api_keys
					.then(|| Self::extract_api_key(&req).map(LoginKind::ApiKey))
					.flatten()
			});

		let user_id = login_kind
			.as_ref()
			.and_then(|login_kind| self.resolve_user(login_kind));

		let session = AuthSession {
			user_id,
			session_map: self.session_map.clone(),
		};
		req.extensions_mut().insert(session);

		self.service.call(req)
	}
}
