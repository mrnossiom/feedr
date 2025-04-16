use std::borrow::Cow;

use diesel::{dsl, prelude::*, r2d2};
use models::UserId;
use serde::{Deserialize, Serialize};
use url::Url;

use self::models::{Feed, FeedId, UserFeedEntryMetaId, UserFeedId};

#[rustfmt::skip]
pub mod schema;
pub mod models;

pub type PoolConnection = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;
pub type PooledConnection = r2d2::PooledConnection<r2d2::ConnectionManager<SqliteConnection>>;

impl Feed<'_> {
	pub fn resolve_or_create(url: &Url, conn: &mut PooledConnection) -> QueryResult<FeedId> {
		conn.transaction(|conn| {
			use crate::database::schema::*;
			let id = feed::table
				.select(feed::id)
				.filter(feed::url.eq(url.as_str()))
				.get_result::<FeedId>(conn)
				.optional()?;

			id.map_or_else(
				|| {
					dsl::insert_into(feed::table)
						.values((feed::url.eq(url.as_str()), feed::status.eq("fetching")))
						.returning(feed::id)
						.get_result(conn)
				},
				Ok,
			)
		})
	}
}

/// A mix between user_feed and feed with user_feed(id) resolved
#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct ResolvedUserFeed<'a> {
	pub id: UserFeedId,

	pub url: Cow<'a, str>,
	pub status: String,

	pub title: Cow<'a, str>,
	pub description: Option<Cow<'a, str>>,
}

impl ResolvedUserFeed<'_> {
	pub fn resolve_all(user_id: UserId, conn: &mut PooledConnection) -> QueryResult<Vec<Self>> {
		use crate::database::schema::*;
		user_feed::table
			.inner_join(feed::table)
			.select((
				user_feed::id,
				feed::url,
				feed::status,
				user_feed::title,
				user_feed::description,
			))
			.filter(user_feed::user_id.eq(user_id))
			.load::<ResolvedUserFeed>(conn)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct ResolvedUserEntry<'a> {
	pub id: UserFeedEntryMetaId,

	pub title: Cow<'a, str>,
	pub content: Option<Cow<'a, str>>,

	pub read: i32,
	pub starred: i32,
}

impl ResolvedUserEntry<'_> {
	pub fn resolve_all(user_id: UserId, conn: &mut PooledConnection) -> QueryResult<Vec<Self>> {
		use crate::database::schema::*;
		user_feed_entry_meta::table
			.inner_join(feed_entry::table)
			.select((
				user_feed_entry_meta::id,
				feed_entry::title,
				feed_entry::content,
				user_feed_entry_meta::read,
				user_feed_entry_meta::starred,
			))
			.filter(user_feed_entry_meta::user_id.eq(user_id))
			.load::<ResolvedUserEntry>(conn)
	}
}
