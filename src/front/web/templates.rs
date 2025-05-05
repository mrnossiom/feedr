use axum::response::{Html, IntoResponse, Response};

use crate::{
	database::{ResolvedUserEntry, ResolvedUserFeed, models::User},
	front::error::RouteError,
};

pub struct Template(Result<String, askama::Error>);

impl Template {
	pub fn render<T: askama::Template>(tpl: &T) -> Self {
		Self(tpl.render())
	}
}

impl IntoResponse for Template {
	fn into_response(self) -> Response {
		self.0
			.map(Html)
			.map_err(RouteError::Template)
			.into_response()
	}
}

#[derive(askama::Template)]
#[template(path = "not_found.html")]
pub struct NotFound<'a> {
	pub user: Option<&'a User>,
}

#[derive(askama::Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
	pub user: Option<&'a User>,

	pub user_feeds: Vec<(String, Vec<ResolvedUserFeed<'a>>)>,
	pub user_entries: Vec<ResolvedUserEntry<'a>>,
}

#[derive(askama::Template)]
#[template(path = "profile.html")]
pub struct Profile<'a> {
	pub user: Option<&'a User>,
}

#[derive(askama::Template)]
#[template(path = "login.html")]
pub struct Login<'a> {
	pub user: Option<&'a User>,
}
