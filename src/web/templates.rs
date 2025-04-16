use std::borrow::Cow;

use askama::Template;

use crate::database::{ResolvedUserEntry, ResolvedUserFeed};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
	pub username: Cow<'a, str>,

	pub user_feeds: Vec<ResolvedUserFeed<'a>>,
	pub user_entries: Vec<ResolvedUserEntry<'a>>,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
	pub username: Option<Cow<'a, str>>,
	pub failed: bool,
}
