use axum::response::IntoResponse;
use diesel::r2d2::PoolError;
use reqwest::StatusCode;

use crate::auth::Backend;

pub type RouteResult<T> = Result<T, RouteError>;

#[derive(Debug, thiserror::Error)]
pub enum RouteError {
	#[error("static: {0}")]
	Static(&'static str),

	#[error("pool: {0}")]
	DbPool(#[from] PoolError),

	#[error("auth: {0}")]
	Auth(#[from] AuthError),

	#[error("template: {0}")]
	Template(#[from] askama::Error),

	#[error("other: {0}")]
	Other(#[from] eyre::Report),

	#[error("{0}")]
	User(&'static str),

	#[error("user opaque {0}: {1}")]
	UserOpaque(&'static str, eyre::Report),
}

impl From<axum_login::Error<Backend>> for RouteError {
	fn from(value: axum_login::Error<Backend>) -> Self {
		match value {
			axum_login::Error::Session(err) => Self::Auth(AuthError::Session(err)),
			axum_login::Error::Backend(err) => Self::Auth(err),
		}
	}
}

impl IntoResponse for RouteError {
	fn into_response(self) -> axum::response::Response {
		match self {
			err @ (Self::Static(_) | Self::DbPool(_) | Self::Template(_) | Self::Other(_)) => {
				tracing::error!(err = %err, "error at route boundary");
				StatusCode::INTERNAL_SERVER_ERROR.into_response()
			}
			Self::Auth(err) => err.into_response(),
			Self::User(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
			Self::UserOpaque(msg, err) => {
				tracing::error!(
					err = %err,
					"user opaque error at route boundary: {msg}"
				);
				(StatusCode::BAD_REQUEST, msg).into_response()
			}
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
	#[error("user is not authenticated")]
	NotAuthenticated,

	#[error("session: {0}")]
	Session(#[from] tower_sessions_core::session::Error),

	#[error("pool: {0}")]
	DbPool(#[from] PoolError),
}

impl IntoResponse for AuthError {
	fn into_response(self) -> axum::response::Response {
		match self {
			err @ Self::NotAuthenticated => {
				(StatusCode::UNAUTHORIZED, err.to_string()).into_response()
			}
			err @ (Self::DbPool(_) | Self::Session(_)) => {
				tracing::error!(err = %err, "error at auth boundary");
				StatusCode::INTERNAL_SERVER_ERROR.into_response()
			}
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum FetcherError {
	#[error("pool: {0}")]
	DbPool(#[from] PoolError),

	#[error("other: {0}")]
	Other(#[from] eyre::Report),
}
