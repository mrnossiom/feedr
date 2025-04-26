use std::borrow::Cow;

use diesel::{dsl, prelude::*, r2d2};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use url::Url;

use self::models::{Feed, FeedId, UserFeedEntryMetaId, UserFeedId};
use self::models::{UserFeedFolder, UserFeedFolderId, UserId};

pub mod models;
pub mod schema;

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

/// A mix between `user_feed` and feed with `user_feed(id)` resolved
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

	pub fn resolve_all_by_folders(
		user_id: UserId,
		conn: &mut PooledConnection,
	) -> QueryResult<Vec<(String, Vec<Self>)>> {
		use crate::database::schema::*;
		let feeds = user_feed::table
			.inner_join(feed::table)
			.left_join(user_feed_folder::table)
			.order_by((user_feed_folder::title, user_feed::title))
			.select((
				user_feed_folder::title.nullable(),
				(
					user_feed::id,
					feed::url,
					feed::status,
					user_feed::title,
					user_feed::description,
				),
			))
			.filter(user_feed::user_id.eq(user_id))
			.load::<(Option<Cow<'_, str>>, ResolvedUserFeed)>(conn)?;

		let feed_folders = feeds
			.into_iter()
			.chunk_by(|(folder, _)| folder.as_deref().unwrap_or("Default").to_string())
			.into_iter()
			.map(|(folder, feeds)| {
				(
					folder,
					feeds
						.into_iter()
						.map(|(_, feeds)| feeds)
						.collect::<Vec<_>>(),
				)
			})
			.collect::<Vec<_>>();

		Ok(feed_folders)
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

impl UserFeedFolder<'_> {
	pub fn resolve_or_create(
		user_id: UserId,
		title: &str,
		conn: &mut PooledConnection,
	) -> QueryResult<UserFeedFolderId> {
		conn.transaction(|conn| {
			use crate::database::schema::*;
			let id = user_feed_folder::table
				.filter(
					user_feed_folder::title
						.eq(title)
						.and(user_feed_folder::user_id.eq(user_id)),
				)
				.select(user_feed_folder::id)
				.get_result::<UserFeedFolderId>(conn)
				.optional()?;

			id.map_or_else(
				|| {
					dsl::insert_into(user_feed_folder::table)
						.values((
							user_feed_folder::title.eq(title),
							user_feed_folder::user_id.eq(user_id),
						))
						.returning(user_feed_folder::id)
						.get_result(conn)
				},
				Ok,
			)
		})
	}
}
