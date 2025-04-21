use axum::response::{IntoResponse, Redirect};
use diesel::r2d2::PoolError;
use reqwest::StatusCode;

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

	#[error("user is not authenticated, returning to login")]
	ReturnToLogin,
}

impl IntoResponse for AuthError {
	fn into_response(self) -> axum::response::Response {
		match self {
			err @ Self::NotAuthenticated => {
				(StatusCode::UNAUTHORIZED, err.to_string()).into_response()
			}
			Self::ReturnToLogin => Redirect::to("/login").into_response(),
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
