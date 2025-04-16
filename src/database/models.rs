use std::borrow::Cow;

use diesel::{
	backend, deserialize, expression::AsExpression, prelude::*, serialize, sql_types::Integer,
};
use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};

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
// pub struct UserId(pub(in crate::database) i32);
pub struct UserId(pub i32);

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user)]
pub struct User<'a> {
	pub id: UserId,

	pub username: Cow<'a, str>,

	pub d_auth_secret: Cow<'a, str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, DieselNewType, Deserialize, Serialize)]
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
