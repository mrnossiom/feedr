use std::borrow::Cow;

use diesel::prelude::*;
use diesel_derive_newtype::DieselNewType;

use crate::database::schema::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct FeedId(i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = feed)]
pub struct Feed<'a> {
	pub id: FeedId,

	pub url: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct FeedEntryId(i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = feed_entry)]
pub struct FeedEntry<'a> {
	pub id: FeedEntryId,
	pub feed_id: FeedId,

	pub title: Cow<'a, str>,
	pub content: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct UserId(pub(in crate::database) i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user)]
pub struct User<'a> {
	pub id: UserId,

	pub username: Cow<'a, str>,

	pub d_auth_secret: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct UserFeedId(i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user_feed)]
pub struct UserFeed<'a> {
	pub id: UserFeedId,
	pub user_id: UserId,
	pub feed_id: FeedId,

	pub title: Cow<'a, str>,
	pub description: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = user_feed)]
pub struct NewUserFeed<'a> {
	pub user_id: UserId,
	pub feed_id: FeedId,

	pub title: Cow<'a, str>,
	pub description: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct UserFeedEntryId(i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user_feed_entry)]
pub struct UserFeedEntry {
	pub id: UserFeedEntryId,
	pub user_id: UserId,
	pub feed_entry_id: FeedEntryId,

	pub is_read: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType)]
pub struct ApiKeyId(i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = api_key)]
pub struct ApiKey<'a> {
	pub id: ApiKeyId,
	pub user_id: UserId,

	pub name: Cow<'a, str>,

	pub secret: Cow<'a, str>,
}
