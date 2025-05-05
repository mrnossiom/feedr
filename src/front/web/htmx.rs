use std::fmt::Write as _;

use axum::{
	body::Body,
	response::{Html, IntoResponse, Response},
};

pub enum HxResponse {
	Redirect(HxRedirect),
	Notices(HxNotices),
}

impl IntoResponse for HxResponse {
	fn into_response(self) -> Response {
		match self {
			Self::Redirect(redir) => redir.into_response(),
			Self::Notices(notices) => notices.into_response(),
		}
	}
}

// ---

pub trait IntoHxResponse: IntoResponse {
	fn into_hx_response(self) -> HxResponse;
}

impl IntoHxResponse for HxResponse {
	fn into_hx_response(self) -> HxResponse {
		self
	}
}

// ---

pub struct HxRedirect {
	location: String,
}

impl HxRedirect {
	pub fn to(path: impl Into<String>) -> Self {
		Self {
			location: path.into(),
		}
	}
}

impl IntoResponse for HxRedirect {
	fn into_response(self) -> Response {
		Response::builder()
			.header("hx-location", self.location)
			.body(Body::empty())
			.expect("response is vaild")
	}
}

impl IntoHxResponse for HxRedirect {
	fn into_hx_response(self) -> HxResponse {
		HxResponse::Redirect(self)
	}
}

/// One or more OOB htmx responses
#[derive(Debug, Default)]
pub struct HxNotices {
	body: String,
}

impl HxNotices {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add(mut self, id: &str, notice: &str) -> Self {
		write!(
			self.body,
			r#"<span id="{id}" hx-swap-oob="innerHTML">{notice}</span>"#
		)
		.unwrap();
		self
	}
}

impl IntoResponse for HxNotices {
	fn into_response(self) -> Response {
		Html(self.body).into_response()
	}
}

impl IntoHxResponse for HxNotices {
	fn into_hx_response(self) -> HxResponse {
		HxResponse::Notices(self)
	}
}
