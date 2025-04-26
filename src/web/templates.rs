use std::{borrow::Cow, collections::HashMap};

use askama::Template;

use crate::database::{ResolvedUserEntry, ResolvedUserFeed, models::User};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
	pub user: &'a User,

	pub user_feeds: HashMap<Cow<'a, str>, Vec<ResolvedUserFeed<'a>>>,
	pub user_entries: Vec<ResolvedUserEntry<'a>>,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileTemplate<'a> {
	pub user: &'a User,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {}
