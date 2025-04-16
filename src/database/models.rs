use std::borrow::Cow;

use diesel::prelude::Insertable;

use crate::database::schema::*;

#[derive(Insertable)]
#[diesel(table_name = feed)]
struct Feed<'a> {
	id: i32,

	url: Cow<'a, str>,
}

#[derive(Insertable)]
#[diesel(table_name = feed_entry)]
struct FeedEntry<'a> {
	id: i32,
	feed_id: i32,

	title: Cow<'a, str>,
	content: Cow<'a, str>,
}

#[derive(Insertable)]
#[diesel(table_name = user)]
struct User<'a> {
	id: i32,

	username: Cow<'a, str>,

	d_auth_secret: Cow<'a, str>,
}

#[derive(Insertable)]
#[diesel(table_name = user_feed)]
struct UserFeed<'a> {
	id: i32,
	user_id: i32,
	feed_id: i32,

	title: Cow<'a, str>,
	description: Cow<'a, str>,
}

#[derive(Insertable)]
#[diesel(table_name = user_feed_entry)]
struct UserFeedEntry {
	id: i32,
	user_id: i32,
	feed_entry_id: i32,

	is_read: i32,
}

#[derive(Insertable)]
#[diesel(table_name = api_key)]
struct ApiKey<'a> {
	id: i32,
	user_id: i32,

	name: Cow<'a, str>,

	secret: Cow<'a, str>,
}
