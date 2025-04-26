use std::task::Context;

use axum::{
	extract::FromRequestParts,
	http::{Request, header, request::Parts},
};
use axum_login::AuthUser;
use diesel::{dsl, prelude::*, r2d2::PoolError, result::DatabaseErrorKind};
use password_auth::verify_password;
use serde::Deserialize;
use tokio::task;
use tower::{Layer, Service};
use tower_sessions::{
	ExpiredDeletion, SessionStore,
	session::{Id, Record},
	session_store,
};

use crate::{
	config::Ressources,
	database::{
		PoolConnection, PooledConnection,
		models::{Session, SessionSecret, User, UserId},
	},
	error::{AuthError, RouteError},
};

pub const SESSION_COOKIE_NAME: &str = "fdr-session";

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

	fn poll_ready(&mut self, cx: &mut Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
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

#[derive(Debug, Clone)]
pub struct SqliteStore {
	db_pool: PoolConnection,
}

impl SqliteStore {
	pub const fn new(db_pool: PoolConnection) -> Self {
		Self { db_pool }
	}

	fn conn(&self) -> session_store::Result<PooledConnection> {
		self.db_pool
			.get()
			.map_err(|err| session_store::Error::Backend(err.to_string()))
	}

	fn try_insert_with_conn(
		conn: &mut PooledConnection,
		record: &Record,
	) -> Result<bool, SessionStoreError> {
		use crate::database::schema::session;

		let data = rmp_serde::encode::to_vec(&record)?;

		let stmt = Session {
			id: SessionSecret(record.id.to_string()),
			data: &data,
			expiry_date: record.expiry_date,
		}
		.insert_into(session::table);

		match stmt.execute(conn) {
			Ok(1) => Ok(true),
			Ok(_)
			| Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => Ok(false),
			Err(err) => Err(err.into()),
		}
	}

	fn upsert_with_conn(
		conn: &mut PooledConnection,
		record: &Record,
	) -> Result<(), SessionStoreError> {
		use crate::database::schema::session;

		let data = rmp_serde::encode::to_vec(&record.data)?;

		let stmt = Session {
			id: SessionSecret(record.id.to_string()),
			data: &data,
			expiry_date: record.expiry_date,
		}
		.insert_into(session::table)
		.on_conflict(session::id)
		.do_update()
		.set((
			session::data.eq(&data),
			session::expiry_date.eq(record.expiry_date),
		));

		stmt.execute(conn)?;
		Ok(())
	}
}

#[async_trait::async_trait]
impl ExpiredDeletion for SqliteStore {
	async fn delete_expired(&self) -> session_store::Result<()> {
		use crate::database::schema::session;
		let mut conn = self.conn()?;

		dsl::delete(session::table)
			.filter(session::expiry_date.lt(dsl::now))
			.execute(&mut conn)
			.map_err(SessionStoreError::Diesel)?;

		Ok(())
	}
}

#[derive(Debug, thiserror::Error)]
pub enum SessionStoreError {
	#[error("rmp decode: {0}")]
	RmpDecode(#[from] rmp_serde::decode::Error),

	#[error("rmp encode: {0}")]
	RmpEncode(#[from] rmp_serde::encode::Error),

	#[error("pool: {0}")]
	DbPool(#[from] PoolError),

	#[error("diesel: {0}")]
	Diesel(#[from] diesel::result::Error),
}

impl From<SessionStoreError> for session_store::Error {
	fn from(value: SessionStoreError) -> Self {
		match value {
			SessionStoreError::RmpDecode(err) => Self::Decode(err.to_string()),
			SessionStoreError::RmpEncode(err) => Self::Encode(err.to_string()),
			SessionStoreError::DbPool(err) => Self::Backend(err.to_string()),
			SessionStoreError::Diesel(err) => Self::Backend(err.to_string()),
		}
	}
}

#[async_trait::async_trait]
impl SessionStore for SqliteStore {
	async fn create(&self, record: &mut Record) -> session_store::Result<()> {
		let mut conn = self.conn()?;
		conn.transaction::<_, SessionStoreError, _>(|conn| {
			while !Self::try_insert_with_conn(conn, record)? {
				// Generate a new ID
				record.id = Id::default();
			}

			Ok(())
		})?;

		Ok(())
	}

	async fn save(&self, record: &Record) -> session_store::Result<()> {
		let mut conn = self.conn()?;
		Self::upsert_with_conn(&mut conn, record)?;
		Ok(())
	}

	async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
		use crate::database::schema::session;
		let mut conn = self.conn()?;
		let stmt = session::table.select(session::data).filter(
			session::id
				.eq(session_id.to_string())
				.and(session::expiry_date.gt(dsl::now)),
		);

		let data = stmt
			.get_result::<Vec<u8>>(&mut conn)
			.optional()
			.map_err(SessionStoreError::Diesel)?;

		if let Some(data) = data {
			let data = rmp_serde::from_slice(&data).map_err(SessionStoreError::RmpDecode)?;
			Ok(Some(data))
		} else {
			Ok(None)
		}
	}

	async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
		use crate::database::schema::session;
		let mut conn = self.conn()?;
		dsl::delete(session::table)
			.filter(session::id.eq(session_id.to_string()))
			.execute(&mut conn)
			.map_err(SessionStoreError::Diesel)?;
		Ok(())
	}
}

#[derive(Clone)]
pub struct Backend {
	db_pool: PoolConnection,
}

impl Backend {
	pub const fn new(db_pool: PoolConnection) -> Self {
		Self { db_pool }
	}
}

pub type AuthSession = axum_login::AuthSession<Backend>;

impl AuthUser for User {
	type Id = UserId;
	fn id(&self) -> Self::Id {
		self.id
	}

	fn session_auth_hash(&self) -> &[u8] {
		self.basic_secret.as_ref().map_or(&[], |sl| sl.as_bytes())
	}
}

#[async_trait::async_trait]
impl axum_login::AuthnBackend for Backend {
	type User = User;
	type Credentials = LoginCredentials;
	type Error = AuthError;

	async fn authenticate(
		&self,
		creds: Self::Credentials,
	) -> Result<Option<Self::User>, Self::Error> {
		use crate::database::schema::user;

		let mut conn = self.db_pool.get()?;

		match creds {
			LoginCredentials::Basic { username, password } => {
				let user = user::table
					.select(user::all_columns)
					.filter(user::username.eq(username))
					.get_result::<User>(&mut conn)
					.optional()
					.unwrap();

				let Some(user) = user else {
					return Ok(None);
				};

				task::spawn_blocking(move || {
					let Some(secret) = &user.basic_secret else {
						return Ok(None);
					};

					let pass_ok = verify_password(password.as_bytes(), secret).is_ok();
					if pass_ok { Ok(Some(user)) } else { Ok(None) }
				})
				.await
				.unwrap()
			}
			LoginCredentials::DAuth { .. } => todo!(),
		}
	}

	async fn get_user(
		&self,
		user_id: &axum_login::UserId<Self>,
	) -> Result<Option<Self::User>, Self::Error> {
		use crate::database::schema::user;
		let mut conn = self.db_pool.get()?;
		let user = user::table
			.select(user::all_columns)
			.filter(user::id.eq(user_id))
			.get_result::<User>(&mut conn)
			.optional()
			.unwrap();

		Ok(user)
	}
}
