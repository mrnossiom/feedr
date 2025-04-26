use std::{borrow::Cow, fmt};

use diesel::prelude::*;
use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::database::schema::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType, Deserialize, Serialize)]
pub struct FeedId(i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = feed)]
pub struct Feed<'a> {
	pub id: FeedId,

	pub url: Cow<'a, str>,

	pub status: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType, Deserialize, Serialize)]
pub struct FeedEntryId(i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = feed_entry)]
pub struct FeedEntry<'a> {
	pub id: FeedEntryId,
	pub feed_id: FeedId,

	pub title: Cow<'a, str>,
	pub content: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType, Deserialize, Serialize)]
pub struct UserId(pub(in crate::database) i32);

impl fmt::Display for UserId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user)]
pub struct User {
	pub id: UserId,

	pub username: String,

	pub basic_secret: Option<String>,
	pub dauth_secret: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType, Deserialize, Serialize)]
pub struct UserFeedFolderId(i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user_feed_folder)]
pub struct UserFeedFolder<'a> {
	pub id: UserFeedFolderId,
	pub user_id: UserId,

	pub title: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType, Deserialize, Serialize)]
pub struct UserFeedId(i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user_feed)]
pub struct UserFeed<'a> {
	pub id: UserFeedId,
	pub user_id: UserId,
	pub feed_id: FeedId,
	pub folder_id: Option<UserFeedFolderId>,

	pub title: Cow<'a, str>,
	pub description: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_feed)]
pub struct NewUserFeed<'a> {
	pub user_id: UserId,
	pub feed_id: FeedId,
	pub folder_id: Option<UserFeedFolderId>,

	pub title: Cow<'a, str>,
	pub description: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType, Deserialize, Serialize)]
pub struct UserFeedEntryMetaId(i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user_feed_entry_meta)]
pub struct UserFeedEntryMeta {
	pub id: UserFeedEntryMetaId,
	pub user_id: UserId,
	pub feed_entry_id: FeedEntryId,

	pub read: i32,
	pub starred: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, DieselNewType, Deserialize, Serialize)]
pub struct SessionSecret(pub(crate) String);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable, Insertable)]
#[diesel(table_name = session)]
pub struct Session<'a> {
	pub id: SessionSecret,
	pub data: &'a [u8],
	pub expiry_date: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType, Deserialize, Serialize)]
pub struct ApiKeyId(i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = api_key)]
pub struct ApiKey<'a> {
	pub id: ApiKeyId,
	pub user_id: UserId,

	pub name: Cow<'a, str>,
	pub secret: Cow<'a, str>,
}
