use std::{collections::HashMap, sync::Arc, task::Context};

use axum::{
	extract::FromRequestParts,
	http::{Request, StatusCode, header, request::Parts},
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use parking_lot::Mutex;
use tower::{Layer, Service};

use crate::{config::Ressources, database::models::UserId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionSecret(String);

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
	pub login_kind: Option<LoginKind>,
}

impl<S: Send + Sync> FromRequestParts<S> for AuthSession {
	type Rejection = (StatusCode, &'static str);

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
		// TODO: ensure msg is backend only
		parts.extensions.get::<Self>().cloned().ok_or((
			StatusCode::INTERNAL_SERVER_ERROR,
			"this is not supposed to show up to the user",
		))
	}
}

#[derive(Debug, Clone)]
pub struct AuthnLayer {
	ressources: Ressources,
	session_map: Arc<Mutex<HashMap<SessionSecret, UserId>>>,
	allow_api_keys: bool,
}

impl AuthnLayer {
	pub fn new(ressources: Ressources) -> Self {
		Self {
			ressources,
			session_map: Arc::default(),
			allow_api_keys: false,
		}
	}

	pub fn new_with_api_keys(ressources: Ressources) -> Self {
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
			ressources: self.ressources.clone(),
			session_map: self.session_map.clone(),
			allow_api_keys: self.allow_api_keys,
		}
	}
}

#[derive(Debug, Clone)]
pub struct AuthnService<S> {
	service: S,
	ressources: Ressources,
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
				let mut conn = self.ressources.get_db_conn().ok()?;
				let user_id: i32 = api_key::table
					.select(api_key::user_id)
					.filter(api_key::secret.eq(&key.0))
					.get_result(&mut conn)
					.ok()?;
				Some(UserId::new(user_id))
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
			login_kind,
		};
		req.extensions_mut().insert(session);

		self.service.call(req)
	}
}
