use axum_login::AuthUser;
use diesel::prelude::*;
use eyre::WrapErr;
use password_auth::verify_password;
use tokio::task;

use crate::{
	database::{
		PoolConnection,
		models::{User, UserId},
	},
	front::error::AuthError,
};

use super::LoginCredentials;

pub type AuthSession = axum_login::AuthSession<Backend>;

#[derive(Clone)]
pub struct Backend {
	db_pool: PoolConnection,
}

impl Backend {
	pub const fn new(db_pool: PoolConnection) -> Self {
		Self { db_pool }
	}
}

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
					.wrap_err("could not retrieve user based on credentials")?;

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
				.wrap_err("could not check password")?
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
			.wrap_err("could not retrieve user")?;

		Ok(user)
	}
}
